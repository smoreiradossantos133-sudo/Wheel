// LLVM backend for Wheel (optional). Build with `--features llvm`.
#[cfg(feature = "llvm")]
pub mod llvm {
    use crate::ast::{Program, Stmt, Expr, BinOp};
    use inkwell::context::Context;
    use inkwell::targets::{Target, InitializationConfig, FileType, RelocMode, CodeModel};
    use inkwell::OptimizationLevel;
    use inkwell::values::{PointerValue, BasicValueEnum};
    use inkwell::types::BasicTypeEnum;
    use inkwell::AddressSpace;
    use std::path::Path;
    use std::collections::HashMap;
    use anyhow::Result;

    pub fn compile_with_llvm(prog: &Program, out_path: &Path) -> Result<()> {
        Target::initialize_all(&InitializationConfig::default());

        let context = Context::create();
        let module = context.create_module("wheel_module");
        let builder = context.create_builder();

        let i32_t = context.i32_type();
        let i64_t = context.i64_type();
        let i8_t = context.i8_type();
        let i8ptr_t = i8_t.ptr_type(AddressSpace::default());

        // declare printf: i32 (i8*, ...)
        let printf_ty = i32_t.fn_type(&[i8ptr_t.into()], true);
        let printf = module.add_function("printf", printf_ty, None);

        // create format strings
        let fmt_i64_arr = context.const_string(b"%ld\n", true);
        let fmt_i64 = module.add_global(fmt_i64_arr.get_type(), None, "_fmt_ld");
        fmt_i64.set_initializer(&fmt_i64_arr);
        fmt_i64.set_constant(true);

        let fmt_str_arr = context.const_string(b"%s\n", true);
        let fmt_str = module.add_global(fmt_str_arr.get_type(), None, "_fmt_s");
        fmt_str.set_initializer(&fmt_str_arr);
        fmt_str.set_constant(true);

        // create main(argc, argv)
        let main_fn_ty = i32_t.fn_type(&[i32_t.into(), i8ptr_t.ptr_type(AddressSpace::default()).into()], false);
        let main_fn = module.add_function("main", main_fn_ty, None);
        let entry = context.append_basic_block(main_fn, "entry");
        builder.position_at_end(entry);

        // allocate local variables for `let` bindings at entry
        let mut locals: HashMap<String, PointerValue> = HashMap::new();

        // sanitize helper for global names
        fn sanitize_name(s: &str) -> String {
            s.chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect()
        }

        // helper to codegen expressions, returns BasicValueEnum
        let gen_expr = |e: &Expr| -> BasicValueEnum {
            let i64_t = context.i64_type();
            match e {
                Expr::Int(v) => i64_t.const_int(*v as u64, true).into(),
                Expr::Ident(name) => {
                    if let Some(ptr) = locals.get(name) {
                        builder.build_load(*ptr, &format!("load_{}", name))
                    } else {
                        i64_t.const_int(0, false).into()
                    }
                }
                Expr::BinaryOp { op, left, right } => {
                    let l = gen_expr(left).into_int_value();
                    let r = gen_expr(right).into_int_value();
                    let res = match op {
                        BinOp::Add => builder.build_int_add(l, r, "addtmp"),
                        BinOp::Sub => builder.build_int_sub(l, r, "subtmp"),
                        BinOp::Mul => builder.build_int_mul(l, r, "multmp"),
                        BinOp::Div => builder.build_int_signed_div(l, r, "divtmp"),
                        BinOp::Lt => builder.build_int_compare(inkwell::IntPredicate::SLT, l, r, "lttmp"),
                        BinOp::Gt => builder.build_int_compare(inkwell::IntPredicate::SGT, l, r, "gttmp"),
                        BinOp::LtEq => builder.build_int_compare(inkwell::IntPredicate::SLE, l, r, "lteqtmp"),
                        BinOp::GtEq => builder.build_int_compare(inkwell::IntPredicate::SGE, l, r, "gteqtmp"),
                        BinOp::EqEq => builder.build_int_compare(inkwell::IntPredicate::EQ, l, r, "eqtmp"),
                        BinOp::NotEq => builder.build_int_compare(inkwell::IntPredicate::NE, l, r, "netmp"),
                    };
                    // comparisons are i1; extend to i64 for uniformity
                    if matches!(op, BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq | BinOp::EqEq | BinOp::NotEq) {
                        builder.build_int_z_extend(res, i64_t, "bool_to_i64").into()
                    } else {
                        res.into()
                    }
                }
                Expr::Str(s) => {
                    // create a global constant with null terminator
                    let mut bytes = s.as_bytes().to_vec();
                    bytes.push(0);
                    let arr = context.const_string(&bytes, true);
                    let gv = module.add_global(arr.get_type(), None, &format!("str_{}", sanitize_name(s)));
                    gv.set_initializer(&arr);
                    gv.set_constant(true);
                    let ptr = gv.as_pointer_value();
                    ptr.into()
                }
                Expr::Call { name, args } => {
                    // not supported in this simple backend; return 0
                    i64_t.const_int(0, false).into()
                }
            }
        };

        // First pass: create allocas for let bindings
        for item in &prog.items {
            if let Stmt::Let { name, value: _ } = item {
                let alloca = builder.build_alloca(i64_t, name);
                locals.insert(name.clone(), alloca);
            }
        }

        // Generate code for statements sequentially
        for item in &prog.items {
            match item {
                Stmt::Let { name, value } => {
                    let val = match value {
                        Expr::Int(_) | Expr::BinaryOp { .. } | Expr::Ident(_) => gen_expr(value, &context, &module, &builder, &locals).into_int_value(),
                        Expr::Str(s) => {
                            // create global string and store pointer as integer (not ideal)
                            let mut bytes = s.as_bytes().to_vec();
                            bytes.push(0);
                            let arr = context.const_string(&bytes, true);
                            let gv = module.add_global(arr.get_type(), None, &format!("str_{}", sanitize_name(s)));
                            gv.set_initializer(&arr);
                            gv.set_constant(true);
                            let ptr = gv.as_pointer_value();
                            // cast pointer to i64
                            let ptr_i64 = builder.build_ptr_to_int(ptr, i64_t, "ptrtoi");
                            builder.build_store(*locals.get(name).unwrap(), ptr_i64);
                            continue;
                        }
                        _ => i64_t.const_int(0, false).into_int_value(),
                    };
                    builder.build_store(*locals.get(name).unwrap(), val);
                }
                Stmt::Assign { name, value } => {
                    let val = gen_expr(value, &context, &module, &builder, &locals).into_int_value();
                    if let Some(ptr) = locals.get(name) {
                        builder.build_store(*ptr, val);
                    }
                }
                Stmt::Expr(Expr::Call { name, args }) if name == "print" && args.len() == 1 => {
                    // handle print for int or string
                    match &args[0] {
                        Expr::Int(v) => {
                            // use printf("%ld\n", v)
                            let fmt_ptr = unsafe { module.get_global("_fmt_ld").unwrap().as_pointer_value() };
                            let val = gen_expr(&args[0], &context, &module, &builder, &locals);
                            builder.build_call(printf, &[fmt_ptr.into(), val.into()], "call_printf");
                        }
                        Expr::Ident(id) => {
                            // assume integer stored in locals
                            if let Some(ptr) = locals.get(id) {
                                let v = builder.build_load(*ptr, &format!("load_{}", id));
                                let fmt_ptr = unsafe { module.get_global("_fmt_ld").unwrap().as_pointer_value() };
                                builder.build_call(printf, &[fmt_ptr.into(), v.into()], "call_printf");
                            }
                        }
                        Expr::Str(s) => {
                            let mut bytes = s.as_bytes().to_vec();
                            bytes.push(0);
                            let arr = context.const_string(&bytes, true);
                            let gv = module.add_global(arr.get_type(), None, &format!("str_{}", sanitize_name(s)));
                            gv.set_initializer(&arr);
                            gv.set_constant(true);
                            let ptr = gv.as_pointer_value();
                            let fmt_ptr = unsafe { module.get_global("_fmt_s").unwrap().as_pointer_value() };
                            builder.build_call(printf, &[fmt_ptr.into(), ptr.into()], "call_printf");
                        }
                        _ => {}
                    }
                }
                Stmt::If { cond, then_body, else_body } => {
                    // evaluate condition
                    let cond_val = gen_expr(cond, &context, &module, &builder, &locals).into_int_value();
                    let zero = i64_t.const_int(0, false);
                    let cond_bool = builder.build_int_compare(inkwell::IntPredicate::NE, cond_val, zero, "ifcond");

                    let then_bb = context.append_basic_block(main_fn, "then");
                    let else_bb = context.append_basic_block(main_fn, "else");
                    let merge_bb = context.append_basic_block(main_fn, "ifend");
                    builder.build_conditional_branch(cond_bool, then_bb, else_bb);

                    // then
                    builder.position_at_end(then_bb);
                    for s in then_body { /* very small recursion: only allow simple stmts */
                        match s {
                            Stmt::Expr(Expr::Call { name, args }) if name=="print" && args.len()==1 => {
                                // reuse code above by building a tiny nested generator: only simple forms
                                if let Expr::Int(v) = &args[0] {
                                    let fmt_ptr = unsafe { module.get_global("_fmt_ld").unwrap().as_pointer_value() };
                                    builder.build_call(printf, &[fmt_ptr.into(), i64_t.const_int(*v as u64, false).into()], "call_printf");
                                }
                            }
                            _ => {}
                        }
                    }
                    builder.build_unconditional_branch(merge_bb);

                    // else
                    builder.position_at_end(else_bb);
                    if let Some(eb) = else_body {
                        for s in eb {
                            match s {
                                Stmt::Expr(Expr::Call { name, args }) if name=="print" && args.len()==1 => {
                                    if let Expr::Int(v) = &args[0] {
                                        let fmt_ptr = unsafe { module.get_global("_fmt_ld").unwrap().as_pointer_value() };
                                        builder.build_call(printf, &[fmt_ptr.into(), i64_t.const_int(*v as u64, false).into()], "call_printf");
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    builder.build_unconditional_branch(merge_bb);

                    builder.position_at_end(merge_bb);
                }
                Stmt::While { cond, body } => {
                    // simple while implementation
                    let loop_bb = context.append_basic_block(main_fn, "loop");
                    let body_bb = context.append_basic_block(main_fn, "loopbody");
                    let after_bb = context.append_basic_block(main_fn, "loopafter");
                    builder.build_unconditional_branch(loop_bb);
                    builder.position_at_end(loop_bb);
                    let cond_val = gen_expr(cond, &context, &module, &builder, &locals).into_int_value();
                    let zero = i64_t.const_int(0, false);
                    let cond_bool = builder.build_int_compare(inkwell::IntPredicate::NE, cond_val, zero, "whilecond");
                    builder.build_conditional_branch(cond_bool, body_bb, after_bb);
                    builder.position_at_end(body_bb);
                    for s in body {
                        // only support print(int) or simple assign
                        match s {
                            Stmt::Expr(Expr::Call { name, args }) if name=="print" && args.len()==1 => {
                                if let Expr::Int(v) = &args[0] {
                                    let fmt_ptr = unsafe { module.get_global("_fmt_ld").unwrap().as_pointer_value() };
                                    builder.build_call(printf, &[fmt_ptr.into(), i64_t.const_int(*v as u64, false).into()], "call_printf");
                                }
                            }
                            Stmt::Assign { name, value } => {
                                let val = gen_expr(value, &context, &module, &builder, &locals).into_int_value();
                                if let Some(ptr) = locals.get(name) {
                                    builder.build_store(*ptr, val);
                                }
                            }
                            _ => {}
                        }
                    }
                    builder.build_unconditional_branch(loop_bb);
                    builder.position_at_end(after_bb);
                }
                _ => {}
            }
        }

        // return 0
        builder.build_return(Some(&i32_t.const_int(0, false)));

        // write object file
        let triple = inkwell::targets::TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).map_err(|e| anyhow::anyhow!("target lookup failed: {:?}", e))?;
        let tm = target.create_target_machine(&triple, "generic", "", OptimizationLevel::Default, inkwell::targets::RelocMode::Default, inkwell::targets::CodeModel::Default).ok_or_else(|| anyhow::anyhow!("failed to create target machine"))?;

        let obj_path = out_path.with_extension("o");
        tm.write_to_file(&module, FileType::Object, &obj_path).map_err(|e| anyhow::anyhow!("write object failed: {:?}", e))?;

        Ok(())
    }
}

#[cfg(not(feature = "llvm"))]
pub mod llvm {
    use crate::ast::Program;
    use std::path::Path;
    use anyhow::Result;
    pub fn compile_with_llvm(_prog: &Program, _out: &Path) -> Result<()> {
        Err(anyhow::anyhow!("LLVM backend not enabled. Build with --features llvm"))
    }
}
