// LLVM backend for Wheel (optional). Build with `--features llvm`.
#[cfg(feature = "llvm")]
pub mod llvm {
    use crate::ast::{Program, Stmt, Expr, BinOp};
    use inkwell::context::Context;
    use inkwell::targets::{Target, InitializationConfig, FileType};
    use inkwell::OptimizationLevel;
    use inkwell::values::{PointerValue, BasicValueEnum};
    use inkwell::AddressSpace;
    use std::path::Path;
    use std::collections::HashMap;
    use anyhow::Result;

    pub fn compile_with_llvm(prog: &Program, out_path: &Path) -> Result<Vec<String>> {
        compile_with_llvm_target(prog, out_path, "x86_64-unknown-linux-gnu")
    }

    pub fn compile_with_llvm_target(prog: &Program, out_path: &Path, target_triple: &str) -> Result<Vec<String>> {
        // dump AST for debugging
        let _ = std::fs::write("/workspaces/Wheel/tmp.ast", format!("{:#?}", prog));

    // initialize all targets for cross-compilation support
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

        // declare scanf: i32 (i8*, ...)
        let scanf_ty = i32_t.fn_type(&[i8ptr_t.into()], true);
        let scanf = module.add_function("scanf", scanf_ty, None);

        // declare malloc: i8* (i64)
        let malloc = match module.get_function("malloc") {
            Some(f) => f,
            None => {
                let malloc_ty = i8ptr_t.fn_type(&[i64_t.into()], false);
                module.add_function("malloc", malloc_ty, None)
            }
        };

        // declare atoi: i32 (i8*)
        let atoi = match module.get_function("atoi") {
            Some(f) => f,
            None => {
                let atoi_ty = i32_t.fn_type(&[i8ptr_t.into()], false);
                module.add_function("atoi", atoi_ty, None)
            }
        };

        // declare strcmp: i32 (i8*, i8*)
        let strcmp = match module.get_function("strcmp") {
            Some(f) => f,
            None => {
                let strcmp_ty = i32_t.fn_type(&[i8ptr_t.into(), i8ptr_t.into()], false);
                module.add_function("strcmp", strcmp_ty, None)
            }
        };

        // create format strings
        let fmt_i64_arr = context.const_string(b"%ld\n", true);
        let fmt_i64 = module.add_global(fmt_i64_arr.get_type(), None, "_fmt_ld");
        fmt_i64.set_initializer(&fmt_i64_arr);
        fmt_i64.set_constant(true);

        let fmt_str_arr = context.const_string(b"%s\n", true);
        let fmt_str = module.add_global(fmt_str_arr.get_type(), None, "_fmt_s");
        fmt_str.set_initializer(&fmt_str_arr);
        fmt_str.set_constant(true);

        let fmt_scan_arr = context.const_string(b"%255s\0", true);
        let fmt_scan = module.add_global(fmt_scan_arr.get_type(), None, "_fmt_scan");
        fmt_scan.set_initializer(&fmt_scan_arr);
        fmt_scan.set_constant(true);

        // global input buffer (256 bytes)
        let input_buf_ty = context.i8_type().array_type(256);
        let input_buf = module.add_global(input_buf_ty, None, "_input_buf");
        input_buf.set_initializer(&input_buf_ty.const_zero());
        input_buf.set_constant(false);

        // create main(argc, argv)
        let main_fn_ty = i32_t.fn_type(&[i32_t.into(), i8ptr_t.ptr_type(AddressSpace::default()).into()], false);
        let main_fn = module.add_function("main", main_fn_ty, None);
        let entry = context.append_basic_block(main_fn, "entry");
        builder.position_at_end(entry);

        // allocate local variables for `let` bindings at entry
        let mut locals: HashMap<String, PointerValue> = HashMap::new();

        // container to collect extra link args (object files, shared libs)
        let mut extra_link_args: Vec<String> = Vec::new();

        // First pass: create allocas for let bindings and record initial values
        let mut initial_vals: HashMap<String, Expr> = HashMap::new();
        for item in &prog.items {
                if let Stmt::Let { name, value, .. } = item {
                let alloca = builder.build_alloca(i64_t, name);
                locals.insert(name.clone(), alloca);
                initial_vals.insert(name.clone(), value.clone());
            }
        }

        // Second pass: Generate function definitions
        let mut user_main_fn: Option<inkwell::values::FunctionValue> = None;
        for item in &prog.items {
            if let Stmt::Func { name, params, body } = item {
                eprintln!("Generating function: {}", name);
                
                // If function is named "main", create it as "user_main" first
                let actual_func_name = if name == "main" { "user_main" } else { name };
                
                let func_type = i64_t.fn_type(&vec![i64_t.into(); params.len()], false);
                let func = module.add_function(actual_func_name, func_type, None);
                
                if name == "main" {
                    user_main_fn = Some(func);
                }
                
                // Create entry block for the function
                let func_entry = context.append_basic_block(func, "entry");
                builder.position_at_end(func_entry);
                
                // Create allocas for parameters
                let mut func_locals: HashMap<String, PointerValue> = HashMap::new();
                for (i, param_name) in params.iter().enumerate() {
                    if let Some(param) = func.get_nth_param(i as u32) {
                        let alloca = builder.build_alloca(i64_t, param_name);
                        builder.build_store(alloca, param.into_int_value());
                        func_locals.insert(param_name.clone(), alloca);
                    }
                }
                
                // Generate function body
                let mut func_initial_vals: HashMap<String, Expr> = HashMap::new();
                for stmt in body {
                    codegen_stmt(stmt, &context, &module, &builder, &printf, &mut func_locals, &func, i64_t, i32_t, &mut func_initial_vals, &mut extra_link_args);
                }
                
                // Default return 0 if not explicitly returned
                builder.build_return(Some(&i64_t.const_int(0, false)));
            }
        }

        // Position back to main entry for remaining code
        builder.position_at_end(entry);

        // Third pass: Generate code for statements (non-function) sequentially
        for item in &prog.items {
            if !matches!(item, Stmt::Func { .. }) {
                codegen_stmt(item, &context, &module, &builder, &printf, &mut locals, &main_fn, i64_t, i32_t, &mut initial_vals, &mut extra_link_args);
            }
        }

        // If there's a user-defined main, call it
        if let Some(user_main) = user_main_fn {
            builder.build_call(user_main, &[], "call_user_main");
        }

        // return 0
        builder.build_return(Some(&i64_t.const_int(0, false)));

        // write object file with specified target triple
        let triple = inkwell::targets::TargetTriple::create(target_triple);
        let target = Target::from_triple(&triple).map_err(|e| anyhow::anyhow!("target lookup failed for {}: {:?}", target_triple, e))?;
        let tm = target.create_target_machine(&triple, "generic", "", OptimizationLevel::Default, inkwell::targets::RelocMode::Default, inkwell::targets::CodeModel::Default).ok_or_else(|| anyhow::anyhow!("failed to create target machine for {}", target_triple))?;

        // dump IR for debugging
        let _ = std::fs::write("/workspaces/Wheel/tmp.ll", module.print_to_string().to_string());
        let obj_path = out_path.with_extension("o");
        tm.write_to_file(&module, FileType::Object, &obj_path).map_err(|e| anyhow::anyhow!("write object failed: {:?}", e))?;

        Ok(extra_link_args)
    }

    fn codegen_stmt<'ctx>(
        stmt: &Stmt,
        context: &'ctx Context,
        module: &inkwell::module::Module<'ctx>,
        builder: &inkwell::builder::Builder<'ctx>,
        printf: &inkwell::values::FunctionValue<'ctx>,
        locals: &mut HashMap<String, PointerValue<'ctx>>,
        main_fn: &inkwell::values::FunctionValue<'ctx>,
        i64_t: inkwell::types::IntType<'ctx>,
        i32_t: inkwell::types::IntType<'ctx>,
        initial_vals: &mut HashMap<String, Expr>,
        extra_link_args: &mut Vec<String>,
    ) {
        match stmt {
            Stmt::Use { lib } => {
                // Handle library imports. Try to resolve local objects/shared libs
                eprintln!("Using library: {}", lib);
                let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let candidates = [
                    cwd.join(format!("{}.o", lib)),
                    cwd.join(format!("lib{}.o", lib)),
                    cwd.join(format!("{}.so", lib)),
                    cwd.join(format!("lib{}.so", lib)),
                    cwd.join(format!("{}", lib)),
                ];
                for cand in &candidates {
                    if cand.exists() {
                        // Add absolute path to link args
                        if let Ok(abs) = cand.canonicalize() {
                            extra_link_args.push(abs.to_string_lossy().to_string());
                            eprintln!("Will link with: {}", abs.display());
                        } else {
                            extra_link_args.push(cand.to_string_lossy().to_string());
                            eprintln!("Will link with: {}", cand.display());
                        }
                        // stop at first match
                        break;
                    }
                }
            }
            Stmt::Import { path } => {
                // Handle file imports
                eprintln!("Importing from: {}", path);
            }
            Stmt::Let { name, value, .. } => {
                    // support reversed form `let input() = Var;` where parser produced
                    // name == "input()" and value == Ident(var). In that case treat
                    // as `let Var = input();` by creating an alloca for Var if missing
                    // and storing the result of `input()` into it.
                    if name.contains('(') {
                        if let Expr::Ident(varname) = value {
                            if !locals.contains_key(varname) {
                                let alloca = builder.build_alloca(i64_t, varname);
                                locals.insert(varname.clone(), alloca);
                            }
                            // record that varname is initialized from input()
                            initial_vals.insert(varname.clone(), Expr::Call { name: "input".to_string(), args: Vec::new() });
                            let input_expr = Expr::Call { name: "input".to_string(), args: Vec::new() };
                            let ival = gen_expr(&input_expr, context, module, builder, locals, i64_t, initial_vals).into_int_value();
                            if let Some(ptr) = locals.get(varname) {
                                builder.build_store(*ptr, ival);
                            }
                            return;
                        }
                    }
                    let val = match value {
                            Expr::Int(_) | Expr::BinaryOp { .. } | Expr::Ident(_) | Expr::Call { .. } => {
                                gen_expr(value, context, module, builder, locals, i64_t, initial_vals).into_int_value()
                            }
                    Expr::Str(s) => {
                        // create global string and store pointer as integer
                        let mut bytes = s.as_bytes().to_vec();
                        bytes.push(0);
                        let arr = context.const_string(&bytes, true);
                        let gv = module.add_global(arr.get_type(), None, &format!("str_{}", sanitize_name(s)));
                        gv.set_initializer(&arr);
                        gv.set_constant(true);
                        let ptr = gv.as_pointer_value();
                        let ptr_i64 = builder.build_ptr_to_int(ptr, i64_t, "ptrtoi");
                        if let Some(ptr_loc) = locals.get(name) {
                            builder.build_store(*ptr_loc, ptr_i64);
                        } else {
                            let alloca = builder.build_alloca(i64_t, name);
                            locals.insert(name.clone(), alloca);
                            builder.build_store(alloca, ptr_i64);
                        }
                        return;
                    }
                        _ => i64_t.const_int(0, false),
                };
                if let Some(ptr) = locals.get(name) {
                    builder.build_store(*ptr, val);
                } else {
                    // create alloca on demand for lets inside blocks and store
                    let alloca = builder.build_alloca(i64_t, name);
                    locals.insert(name.clone(), alloca);
                    // record initial value expression so later code knows this name came from input/str
                    initial_vals.insert(name.clone(), value.clone());
                    builder.build_store(alloca, val);
                }
            }
            Stmt::Assign { name, value } => {
                let val = gen_expr(value, context, module, builder, locals, i64_t, initial_vals).into_int_value();
                if let Some(ptr) = locals.get(name) {
                    builder.build_store(*ptr, val);
                }
            }
            Stmt::Expr(Expr::Call { name, args }) if name == "print" && args.len() == 1 => {
                // handle print for strings or any expression that evaluates to an integer
                match &args[0] {
                    Expr::Str(s) => {
                        // print string literal
                        let mut bytes = s.as_bytes().to_vec();
                        bytes.push(0);
                        let arr = context.const_string(&bytes, true);
                        let gv = module.add_global(arr.get_type(), None, &format!("str_{}", sanitize_name(s)));
                        gv.set_initializer(&arr);
                        gv.set_constant(true);
                        let ptr = gv.as_pointer_value();
                        let fmt_g = module.get_global("_fmt_s").unwrap().as_pointer_value();
                        let fmt_ptr = builder.build_bitcast(fmt_g, context.i8_type().ptr_type(AddressSpace::default()), "fmt_s_cast").into_pointer_value();
                        builder.build_call(*printf, &[fmt_ptr.into(), ptr.into()], "call_printf");
                    }
                    _ => {
                        // For any other expression (int literal, binary op, ident, call), generate the expression and print as integer
                        let val = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals).into_int_value();
                        let fmt_g = module.get_global("_fmt_ld").unwrap().as_pointer_value();
                        let fmt_ptr = builder.build_bitcast(fmt_g, context.i8_type().ptr_type(AddressSpace::default()), "fmt_ld_cast").into_pointer_value();
                        builder.build_call(*printf, &[fmt_ptr.into(), val.into()], "call_printf");
                    }
                }
            }
            Stmt::If { cond, then_body, else_body } => {
                // evaluate condition
                let cond_val = gen_expr(cond, context, module, builder, locals, i64_t, initial_vals).into_int_value();
                let zero = i64_t.const_int(0, false);
                let cond_bool = builder.build_int_compare(inkwell::IntPredicate::NE, cond_val, zero, "ifcond");

                let then_bb = context.append_basic_block(*main_fn, "then");
                let else_bb = context.append_basic_block(*main_fn, "else");
                let merge_bb = context.append_basic_block(*main_fn, "ifend");
                builder.build_conditional_branch(cond_bool, then_bb, else_bb);

                // then
                builder.position_at_end(then_bb);
                for s in then_body {
                    codegen_stmt(s, context, module, builder, printf, locals, main_fn, i64_t, i32_t, initial_vals, extra_link_args);
                }
                builder.build_unconditional_branch(merge_bb);

                // else
                builder.position_at_end(else_bb);
                if let Some(eb) = else_body {
                    for s in eb {
                        codegen_stmt(s, context, module, builder, printf, locals, main_fn, i64_t, i32_t, initial_vals, extra_link_args);
                    }
                }
                builder.build_unconditional_branch(merge_bb);

                builder.position_at_end(merge_bb);
            }
            Stmt::While { cond, body } => {
                // simple while implementation
                let loop_bb = context.append_basic_block(*main_fn, "loop");
                let body_bb = context.append_basic_block(*main_fn, "loopbody");
                let after_bb = context.append_basic_block(*main_fn, "loopafter");
                builder.build_unconditional_branch(loop_bb);
                builder.position_at_end(loop_bb);
                let cond_val = gen_expr(cond, context, module, builder, locals, i64_t, initial_vals).into_int_value();
                let zero = i64_t.const_int(0, false);
                let cond_bool = builder.build_int_compare(inkwell::IntPredicate::NE, cond_val, zero, "whilecond");
                builder.build_conditional_branch(cond_bool, body_bb, after_bb);
                builder.position_at_end(body_bb);
                for s in body {
                    codegen_stmt(s, context, module, builder, printf, locals, main_fn, i64_t, i32_t, initial_vals, extra_link_args);
                }
                builder.build_unconditional_branch(loop_bb);
                builder.position_at_end(after_bb);
            }
            Stmt::ForRange { var, start, end, body } => {
                use inkwell::IntPredicate;
                // allocate loop variable in entry block
                let loop_var = builder.build_alloca(i64_t, &format!("for_{}", var));

                // initialize with start
                let start_val = gen_expr(start, context, module, builder, locals, i64_t, initial_vals).into_int_value();
                builder.build_store(loop_var, start_val);

                // create blocks
                let check_bb = context.append_basic_block(*main_fn, &format!("for_check_{}", var));
                let body_bb = context.append_basic_block(*main_fn, &format!("for_body_{}", var));
                let after_bb = context.append_basic_block(*main_fn, &format!("for_after_{}", var));

                builder.build_unconditional_branch(check_bb);

                // check condition
                builder.position_at_end(check_bb);
                let cur = builder.build_load(i64_t, loop_var, &format!("load_{}", var)).into_int_value();
                let end_val = gen_expr(end, context, module, builder, locals, i64_t, initial_vals).into_int_value();
                let cond = builder.build_int_compare(IntPredicate::SLT, cur, end_val, "for_cond");
                builder.build_conditional_branch(cond, body_bb, after_bb);

                // body
                builder.position_at_end(body_bb);
                let mut body_locals = locals.clone();
                body_locals.insert(var.clone(), loop_var);
                for s in body {
                    codegen_stmt(s, context, module, builder, printf, &mut body_locals, main_fn, i64_t, i32_t, initial_vals, extra_link_args);
                }

                // increment
                let cur2 = builder.build_load(i64_t, loop_var, &format!("loadinc_{}", var)).into_int_value();
                let one = i64_t.const_int(1, false);
                let next = builder.build_int_add(cur2, one, &format!("inc_{}", var));
                builder.build_store(loop_var, next);
                builder.build_unconditional_branch(check_bb);

                builder.position_at_end(after_bb);
            }
            Stmt::ArrayAssign { .. } => {
                // placeholder for array assignment
                eprintln!("Array assignment not yet fully implemented");
            }
            Stmt::StructDef { .. } => {
                // placeholder for struct definition
                eprintln!("Struct definition not yet fully implemented");
            }
            _ => {}
        }
    }

    fn gen_expr<'ctx>(
        e: &Expr,
        context: &'ctx Context,
        module: &inkwell::module::Module<'ctx>,
        builder: &inkwell::builder::Builder<'ctx>,
        locals: &HashMap<String, PointerValue<'ctx>>,
        i64_t: inkwell::types::IntType<'ctx>,
        initial_vals: &HashMap<String, Expr>,
    ) -> BasicValueEnum<'ctx> {
        match e {
            Expr::Int(v) => i64_t.const_int(*v as u64, true).into(),
            Expr::Ident(name) => {
                if let Some(ptr) = locals.get(name) {
                        builder.build_load(i64_t, *ptr, &format!("load_{}", name))
                } else {
                    i64_t.const_int(0, false).into()
                }
            }
            Expr::ArrayAccess { .. } => {
                i64_t.const_int(0, false).into() // placeholder for array access
            }
            Expr::ArrayLiteral(_) => {
                i64_t.const_int(0, false).into() // placeholder for array literal
            }
            Expr::BinaryOp { op, left, right } => {
                // handle string equality by using strcmp when either side is a string literal or input
                use inkwell::IntPredicate;
                if matches!(op, BinOp::EqEq | BinOp::NotEq) {
                    let is_string_like = |ex: &Expr| -> bool {
                        match ex {
                            Expr::Str(_) => true,
                            // input() returns integer (via atoi conversion), not string-like
                            Expr::Ident(id) => {
                                if let Some(v) = initial_vals.get(id) {
                                    match v {
                                        Expr::Str(_) => true,
                                        // input() is converted to int, don't treat as string
                                        Expr::Call { name, .. } if name == "input" => false,
                                        _ => false,
                                    }
                                } else { false }
                            }
                            _ => false,
                        }
                    };
                    let left_is_str = is_string_like(&*left);
                    let right_is_str = is_string_like(&*right);
                    if left_is_str || right_is_str {
                        // Converte ambos os lados para inteiro com `atoi` e compara os inteiros.
                        let lval = gen_expr(&*left, context, module, builder, locals, i64_t, initial_vals).into_int_value();
                        let rval = gen_expr(&*right, context, module, builder, locals, i64_t, initial_vals).into_int_value();
                        let i8ptr = context.i8_type().ptr_type(AddressSpace::default());
                        let lptr = builder.build_int_to_ptr(lval, i8ptr, "lptr");
                        let rptr = builder.build_int_to_ptr(rval, i8ptr, "rptr");
                        let atoi_fn = module.get_function("atoi").unwrap_or_else(|| {
                            let atoi_ty = context.i32_type().fn_type(&[context.i8_type().ptr_type(AddressSpace::default()).into()], false);
                            module.add_function("atoi", atoi_ty, None)
                        });
                        let lcall = builder.build_call(atoi_fn, &[lptr.into()], "call_atoi_l");
                        let rcall = builder.build_call(atoi_fn, &[rptr.into()], "call_atoi_r");
                        let li = lcall.try_as_basic_value().left().unwrap().into_int_value();
                        let ri = rcall.try_as_basic_value().left().unwrap().into_int_value();
                        let cmpi = match op {
                            BinOp::EqEq => builder.build_int_compare(IntPredicate::EQ, li, ri, "eqi"),
                            BinOp::NotEq => builder.build_int_compare(IntPredicate::NE, li, ri, "nei"),
                            _ => builder.build_int_compare(IntPredicate::EQ, li, ri, "eqi"),
                        };
                        let ext = builder.build_int_z_extend(cmpi, i64_t, "bool_to_i64");
                        return ext.into();
                    }
                }

                // compute operands
                let mut l = gen_expr(left, context, module, builder, locals, i64_t, initial_vals).into_int_value();
                let mut r = gen_expr(right, context, module, builder, locals, i64_t, initial_vals).into_int_value();

                // if arithmetic operation and operands are string-like, convert with atoi
                let is_arith = matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div);
                if is_arith {
                    let left_is_str = {
                        match &**left {
                            Expr::Str(_) => true,
                            // input() now returns int, not string-like
                            Expr::Call { name, .. } if name=="input" => false,
                            Expr::Ident(id) => {
                                if let Some(v) = initial_vals.get(id) {
                                    match v {
                                        Expr::Str(_) => true,
                                        // input() returns int, don't apply atoi
                                        Expr::Call { name, .. } if name == "input" => false,
                                        _ => false,
                                    }
                                } else { false }
                            }
                            _ => false,
                        }
                    };
                    let right_is_str = {
                        match &**right {
                            Expr::Str(_) => true,
                            // input() now returns int, not string-like
                            Expr::Call { name, .. } if name=="input" => false,
                            Expr::Ident(id) => {
                                if let Some(v) = initial_vals.get(id) {
                                    match v {
                                        Expr::Str(_) => true,
                                        // input() returns int, don't apply atoi
                                        Expr::Call { name, .. } if name == "input" => false,
                                        _ => false,
                                    }
                                } else { false }
                            }
                            _ => false,
                        }
                    };
                    let i8ptr = context.i8_type().ptr_type(AddressSpace::default());
                    let atoi_fn = module.get_function("atoi").unwrap_or_else(|| {
                        let atoi_ty = context.i32_type().fn_type(&[context.i8_type().ptr_type(AddressSpace::default()).into()], false);
                        module.add_function("atoi", atoi_ty, None)
                    });
                    if left_is_str {
                        let lptr = builder.build_int_to_ptr(l, i8ptr, "lptr_for_atoi");
                        let call = builder.build_call(atoi_fn, &[lptr.into()], "call_atoi");
                        let ai = call.try_as_basic_value().left().unwrap().into_int_value();
                        l = builder.build_int_z_extend(ai, i64_t, "atoi_to_i64");
                    }
                    if right_is_str {
                        let rptr = builder.build_int_to_ptr(r, i8ptr, "rptr_for_atoi");
                        let call = builder.build_call(atoi_fn, &[rptr.into()], "call_atoi");
                        let ai = call.try_as_basic_value().left().unwrap().into_int_value();
                        r = builder.build_int_z_extend(ai, i64_t, "atoi_to_i64");
                    }
                }
                let res: inkwell::values::IntValue = match op {
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
                // represent strings as i64 pointer values to keep a uniform integer model
                let ptr_i64 = builder.build_ptr_to_int(ptr, i64_t, "strptrtoi");
                ptr_i64.into()
            }
            Expr::Call { name, args } => {
                match name.as_str() {
                    // Built-in input() function - returns integer (auto-converted via atoi)
                    "input" if args.len() == 0 => {
                        let i32_t = context.i32_type();
                        let i8_t = context.i8_type();
                        let gv = module.get_global("_input_buf").expect("_input_buf global missing").as_pointer_value();
                        let i8ptr_t_local = i8_t.ptr_type(AddressSpace::default());
                        let buf_ptr = builder.build_pointer_cast(gv, i8ptr_t_local, "buf_ptr");
                        let scanf_fn = module.get_function("scanf").expect("scanf should be declared");
                        let fmt_g = module.get_global("_fmt_scan").unwrap().as_pointer_value();
                        let fmt_ptr = builder.build_bitcast(fmt_g, i8ptr_t_local, "fmt_scan_cast").into_pointer_value();
                        builder.build_call(scanf_fn, &[fmt_ptr.into(), buf_ptr.into()], "call_scanf");
                        
                        // Convert the input string to integer using atoi
                        let atoi_fn = module.get_function("atoi").unwrap_or_else(|| {
                            let atoi_ty = i32_t.fn_type(&[i8ptr_t_local.into()], false);
                            module.add_function("atoi", atoi_ty, None)
                        });
                        let atoi_result = builder.build_call(atoi_fn, &[buf_ptr.into()], "call_atoi_input");
                        let atoi_val = atoi_result.try_as_basic_value().left().unwrap().into_int_value();
                        // Extend i32 result to i64
                        builder.build_int_z_extend(atoi_val, i64_t, "atoi_to_i64").into()
                    }
                    
                    // SDL Library functions
                    "sdl_init" if args.len() == 0 => {
                        let fn_ty = i64_t.fn_type(&[], false);
                        let func = match module.get_function("sdl_init") {
                            Some(f) => f,
                            None => module.add_function("sdl_init", fn_ty, None),
                        };
                        builder.build_call(func, &[], "call_sdl_init").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "sdl_create_window" if args.len() == 3 => {
                        let fn_ty = i64_t.fn_type(&[i64_t.into(), i64_t.into(), i64_t.into()], false);
                        let func = match module.get_function("sdl_create_window") {
                            Some(f) => f,
                            None => module.add_function("sdl_create_window", fn_ty, None),
                        };
                        let w = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        let h = gen_expr(&args[1], context, module, builder, locals, i64_t, initial_vals);
                        let t = gen_expr(&args[2], context, module, builder, locals, i64_t, initial_vals);
                        builder.build_call(func, &[w.into(), h.into(), t.into()], "call_sdl_create_window").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "sdl_draw_pixel" if args.len() == 5 => {
                        let fn_ty = i64_t.fn_type(&[i64_t.into(), i64_t.into(), i64_t.into(), i64_t.into(), i64_t.into()], false);
                        let func = match module.get_function("sdl_draw_pixel") {
                            Some(f) => f,
                            None => module.add_function("sdl_draw_pixel", fn_ty, None),
                        };
                        let x = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        let y = gen_expr(&args[1], context, module, builder, locals, i64_t, initial_vals);
                        let r = gen_expr(&args[2], context, module, builder, locals, i64_t, initial_vals);
                        let g = gen_expr(&args[3], context, module, builder, locals, i64_t, initial_vals);
                        let b = gen_expr(&args[4], context, module, builder, locals, i64_t, initial_vals);
                        builder.build_call(func, &[x.into(), y.into(), r.into(), g.into(), b.into()], "call_sdl_draw_pixel").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "sdl_draw_rect" if args.len() == 7 => {
                        let fn_ty = i64_t.fn_type(&[i64_t.into(), i64_t.into(), i64_t.into(), i64_t.into(), i64_t.into(), i64_t.into(), i64_t.into()], false);
                        let func = match module.get_function("sdl_draw_rect") {
                            Some(f) => f,
                            None => module.add_function("sdl_draw_rect", fn_ty, None),
                        };
                        let x = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        let y = gen_expr(&args[1], context, module, builder, locals, i64_t, initial_vals);
                        let w = gen_expr(&args[2], context, module, builder, locals, i64_t, initial_vals);
                        let h = gen_expr(&args[3], context, module, builder, locals, i64_t, initial_vals);
                        let r = gen_expr(&args[4], context, module, builder, locals, i64_t, initial_vals);
                        let g = gen_expr(&args[5], context, module, builder, locals, i64_t, initial_vals);
                        let b = gen_expr(&args[6], context, module, builder, locals, i64_t, initial_vals);
                        builder.build_call(func, &[x.into(), y.into(), w.into(), h.into(), r.into(), g.into(), b.into()], "call_sdl_draw_rect").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "sdl_clear" if args.len() == 3 => {
                        let fn_ty = i64_t.fn_type(&[i64_t.into(), i64_t.into(), i64_t.into()], false);
                        let func = match module.get_function("sdl_clear") {
                            Some(f) => f,
                            None => module.add_function("sdl_clear", fn_ty, None),
                        };
                        let r = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        let g = gen_expr(&args[1], context, module, builder, locals, i64_t, initial_vals);
                        let b = gen_expr(&args[2], context, module, builder, locals, i64_t, initial_vals);
                        builder.build_call(func, &[r.into(), g.into(), b.into()], "call_sdl_clear").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "sdl_present" if args.len() == 0 => {
                        let fn_ty = i64_t.fn_type(&[], false);
                        let func = match module.get_function("sdl_present") {
                            Some(f) => f,
                            None => module.add_function("sdl_present", fn_ty, None),
                        };
                        builder.build_call(func, &[], "call_sdl_present").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "sdl_poll_event" if args.len() == 0 => {
                        let fn_ty = i64_t.fn_type(&[], false);
                        let func = match module.get_function("sdl_poll_event") {
                            Some(f) => f,
                            None => module.add_function("sdl_poll_event", fn_ty, None),
                        };
                        builder.build_call(func, &[], "call_sdl_poll_event").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "sdl_delay" if args.len() == 1 => {
                        let fn_ty = i64_t.fn_type(&[i64_t.into()], false);
                        let func = match module.get_function("sdl_delay") {
                            Some(f) => f,
                            None => module.add_function("sdl_delay", fn_ty, None),
                        };
                        let ms = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        builder.build_call(func, &[ms.into()], "call_sdl_delay").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "sdl_destroy_window" if args.len() == 0 => {
                        let fn_ty = i64_t.fn_type(&[], false);
                        let func = match module.get_function("sdl_destroy_window") {
                            Some(f) => f,
                            None => module.add_function("sdl_destroy_window", fn_ty, None),
                        };
                        builder.build_call(func, &[], "call_sdl_destroy_window").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "sdl_quit" if args.len() == 0 => {
                        let fn_ty = i64_t.fn_type(&[], false);
                        let func = match module.get_function("sdl_quit") {
                            Some(f) => f,
                            None => module.add_function("sdl_quit", fn_ty, None),
                        };
                        builder.build_call(func, &[], "call_sdl_quit").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    
                    // Hardware I/O functions
                    "port_read_byte" if args.len() == 1 => {
                        let fn_ty = i64_t.fn_type(&[i64_t.into()], false);
                        let func = match module.get_function("port_read_byte") {
                            Some(f) => f,
                            None => module.add_function("port_read_byte", fn_ty, None),
                        };
                        let port = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        builder.build_call(func, &[port.into()], "call_port_read_byte").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "port_write_byte" if args.len() == 2 => {
                        let fn_ty = i64_t.fn_type(&[i64_t.into(), i64_t.into()], false);
                        let func = match module.get_function("port_write_byte") {
                            Some(f) => f,
                            None => module.add_function("port_write_byte", fn_ty, None),
                        };
                        let port = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        let val = gen_expr(&args[1], context, module, builder, locals, i64_t, initial_vals);
                        builder.build_call(func, &[port.into(), val.into()], "call_port_write_byte").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "getpid" if args.len() == 0 => {
                        let fn_ty = i64_t.fn_type(&[], false);
                        let func = match module.get_function("wheel_getpid") {
                            Some(f) => f,
                            None => module.add_function("wheel_getpid", fn_ty, None),
                        };
                        builder.build_call(func, &[], "call_getpid").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "sleep" if args.len() == 1 => {
                        let fn_ty = i64_t.fn_type(&[i64_t.into()], false);
                        let func = match module.get_function("wheel_sleep") {
                            Some(f) => f,
                            None => module.add_function("wheel_sleep", fn_ty, None),
                        };
                        let secs = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        builder.build_call(func, &[secs.into()], "call_sleep").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "time_now" if args.len() == 0 => {
                        let fn_ty = i64_t.fn_type(&[], false);
                        let func = match module.get_function("wheel_time_now") {
                            Some(f) => f,
                            None => module.add_function("wheel_time_now", fn_ty, None),
                        };
                        builder.build_call(func, &[], "call_time_now").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    
                    // Luck library functions - random number generation
                    "luck_random" if args.len() == 1 => {
                        let fn_ty = i64_t.fn_type(&[i64_t.into()], false);
                        let func = match module.get_function("luck_random") {
                            Some(f) => f,
                            None => module.add_function("luck_random", fn_ty, None),
                        };
                        let max = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        builder.build_call(func, &[max.into()], "call_luck_random").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "luck_random_range" if args.len() == 2 => {
                        let fn_ty = i64_t.fn_type(&[i64_t.into(), i64_t.into()], false);
                        let func = match module.get_function("luck_random_range") {
                            Some(f) => f,
                            None => module.add_function("luck_random_range", fn_ty, None),
                        };
                        let min = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        let max = gen_expr(&args[1], context, module, builder, locals, i64_t, initial_vals);
                        builder.build_call(func, &[min.into(), max.into()], "call_luck_random_range").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    
                    // Memory management functions
                    "mem_alloc" if args.len() == 1 => {
                        let i8_ptr = context.i8_type().ptr_type(inkwell::AddressSpace::default());
                        let fn_ty = i8_ptr.fn_type(&[i64_t.into()], false);
                        let func = module.get_function("mem_alloc").unwrap_or_else(|| module.add_function("mem_alloc", fn_ty, None));
                        let size = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        let ptr_result = builder.build_call(func, &[size.into()], "call_mem_alloc").try_as_basic_value().left().unwrap();
                        if let inkwell::values::BasicValueEnum::PointerValue(ptr) = ptr_result {
                            builder.build_ptr_to_int(ptr, i64_t, "ptr_to_i64").into()
                        } else {
                            i64_t.const_int(0, false).into()
                        }
                    }
                    "mem_free" if args.len() == 1 => {
                        let i8_ptr = context.i8_type().ptr_type(inkwell::AddressSpace::default());
                        let fn_ty = context.void_type().fn_type(&[i8_ptr.into()], false);
                        let func = module.get_function("mem_free").unwrap_or_else(|| module.add_function("mem_free", fn_ty, None));
                        let ptr = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        builder.build_call(func, &[ptr.into()], "call_mem_free");
                        i64_t.const_int(0, false).into()
                    }
                    "mem_get_used" if args.is_empty() => {
                        let fn_ty = i64_t.fn_type(&[], false);
                        let func = module.get_function("mem_get_used").unwrap_or_else(|| module.add_function("mem_get_used", fn_ty, None));
                        builder.build_call(func, &[], "call_mem_get_used").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "mem_get_free" if args.is_empty() => {
                        let fn_ty = i64_t.fn_type(&[], false);
                        let func = module.get_function("mem_get_free").unwrap_or_else(|| module.add_function("mem_get_free", fn_ty, None));
                        builder.build_call(func, &[], "call_mem_get_free").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    
                    // Hardware I/O functions
                    "io_read_port" if args.len() == 1 => {
                        let fn_ty = i64_t.fn_type(&[i64_t.into()], false);
                        let func = module.get_function("io_read_port").unwrap_or_else(|| module.add_function("io_read_port", fn_ty, None));
                        let port = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        builder.build_call(func, &[port.into()], "call_io_read_port").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "io_write_port" if args.len() == 2 => {
                        let fn_ty = context.void_type().fn_type(&[i64_t.into(), i64_t.into()], false);
                        let func = module.get_function("io_write_port").unwrap_or_else(|| module.add_function("io_write_port", fn_ty, None));
                        let port = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        let value = gen_expr(&args[1], context, module, builder, locals, i64_t, initial_vals);
                        builder.build_call(func, &[port.into(), value.into()], "call_io_write_port");
                        i64_t.const_int(0, false).into()
                    }
                    "io_enable_interrupts" if args.is_empty() => {
                        let fn_ty = context.void_type().fn_type(&[], false);
                        let func = module.get_function("io_enable_interrupts").unwrap_or_else(|| module.add_function("io_enable_interrupts", fn_ty, None));
                        builder.build_call(func, &[], "call_io_enable_interrupts");
                        i64_t.const_int(0, false).into()
                    }
                    "io_disable_interrupts" if args.is_empty() => {
                        let fn_ty = context.void_type().fn_type(&[], false);
                        let func = module.get_function("io_disable_interrupts").unwrap_or_else(|| module.add_function("io_disable_interrupts", fn_ty, None));
                        builder.build_call(func, &[], "call_io_disable_interrupts");
                        i64_t.const_int(0, false).into()
                    }
                    "io_halt" if args.is_empty() => {
                        let fn_ty = context.void_type().fn_type(&[], false);
                        let func = module.get_function("io_halt").unwrap_or_else(|| module.add_function("io_halt", fn_ty, None));
                        builder.build_call(func, &[], "call_io_halt");
                        i64_t.const_int(0, false).into()
                    }
                    
                    // Filesystem functions
                    "fs_open" if args.len() == 1 => {
                        let i8_ptr = context.i8_type().ptr_type(inkwell::AddressSpace::default());
                        let fn_ty = i64_t.fn_type(&[i8_ptr.into()], false);
                        let func = module.get_function("fs_open").unwrap_or_else(|| module.add_function("fs_open", fn_ty, None));
                        let device = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        builder.build_call(func, &[device.into()], "call_fs_open").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "fs_close" if args.len() == 1 => {
                        let fn_ty = context.void_type().fn_type(&[i64_t.into()], false);
                        let func = module.get_function("fs_close").unwrap_or_else(|| module.add_function("fs_close", fn_ty, None));
                        let handle = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        builder.build_call(func, &[handle.into()], "call_fs_close");
                        i64_t.const_int(0, false).into()
                    }
                    "fs_read_block" if args.len() == 3 => {
                        let i8_ptr = context.i8_type().ptr_type(inkwell::AddressSpace::default());
                        let fn_ty = i64_t.fn_type(&[i64_t.into(), i64_t.into(), i8_ptr.into()], false);
                        let func = module.get_function("fs_read_block").unwrap_or_else(|| module.add_function("fs_read_block", fn_ty, None));
                        let handle = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        let block = gen_expr(&args[1], context, module, builder, locals, i64_t, initial_vals);
                        let buffer = gen_expr(&args[2], context, module, builder, locals, i64_t, initial_vals);
                        builder.build_call(func, &[handle.into(), block.into(), buffer.into()], "call_fs_read_block").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "fs_write_block" if args.len() == 3 => {
                        let i8_ptr = context.i8_type().ptr_type(inkwell::AddressSpace::default());
                        let fn_ty = i64_t.fn_type(&[i64_t.into(), i64_t.into(), i8_ptr.into()], false);
                        let func = module.get_function("fs_write_block").unwrap_or_else(|| module.add_function("fs_write_block", fn_ty, None));
                        let handle = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        let block = gen_expr(&args[1], context, module, builder, locals, i64_t, initial_vals);
                        let buffer = gen_expr(&args[2], context, module, builder, locals, i64_t, initial_vals);
                        builder.build_call(func, &[handle.into(), block.into(), buffer.into()], "call_fs_write_block").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    
                    // Process management functions
                    "process_init" if args.is_empty() => {
                        let fn_ty = context.void_type().fn_type(&[], false);
                        let func = module.get_function("process_init").unwrap_or_else(|| module.add_function("process_init", fn_ty, None));
                        builder.build_call(func, &[], "call_process_init");
                        i64_t.const_int(0, false).into()
                    }
                    "process_create" if args.len() == 1 => {
                        let i8_ptr = context.i8_type().ptr_type(inkwell::AddressSpace::default());
                        let fn_ty = i64_t.fn_type(&[i8_ptr.into()], false);
                        let func = module.get_function("process_create").unwrap_or_else(|| module.add_function("process_create", fn_ty, None));
                        let command = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        builder.build_call(func, &[command.into()], "call_process_create").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "process_wait" if args.len() == 1 => {
                        let fn_ty = i64_t.fn_type(&[i64_t.into()], false);
                        let func = module.get_function("process_wait").unwrap_or_else(|| module.add_function("process_wait", fn_ty, None));
                        let pid = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        builder.build_call(func, &[pid.into()], "call_process_wait").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "process_is_running" if args.len() == 1 => {
                        let fn_ty = i64_t.fn_type(&[i64_t.into()], false);
                        let func = module.get_function("process_is_running").unwrap_or_else(|| module.add_function("process_is_running", fn_ty, None));
                        let pid = gen_expr(&args[0], context, module, builder, locals, i64_t, initial_vals);
                        builder.build_call(func, &[pid.into()], "call_process_is_running").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    "process_yield" if args.is_empty() => {
                        let fn_ty = context.void_type().fn_type(&[], false);
                        let func = module.get_function("process_yield").unwrap_or_else(|| module.add_function("process_yield", fn_ty, None));
                        builder.build_call(func, &[], "call_process_yield");
                        i64_t.const_int(0, false).into()
                    }
                    "process_get_current_pid" if args.is_empty() => {
                        let fn_ty = i64_t.fn_type(&[], false);
                        let func = module.get_function("process_get_current_pid").unwrap_or_else(|| module.add_function("process_get_current_pid", fn_ty, None));
                        builder.build_call(func, &[], "call_process_get_current_pid").try_as_basic_value().left().unwrap_or(i64_t.const_int(0, false).into())
                    }
                    
                    // Default: unknown function
                    _ => i64_t.const_int(0, false).into()
                }
            }
        }
    }

    fn sanitize_name(s: &str) -> String {
        s.chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect()
    }
}

#[cfg(not(feature = "llvm"))]
pub mod llvm {
    use crate::ast::Program;
    use std::path::Path;
    use anyhow::Result;
    pub fn compile_with_llvm(_prog: &Program, _out: &Path) -> Result<Vec<String>> {
        Err(anyhow::anyhow!("LLVM backend not enabled. Build with --features llvm"))
    }
    pub fn compile_with_llvm_target(_prog: &Program, _out: &Path, _target: &str) -> Result<Vec<String>> {
        Err(anyhow::anyhow!("LLVM backend not enabled. Build with --features llvm"))
    }
}
