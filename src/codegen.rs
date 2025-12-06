use crate::ast::{Program, Stmt, Expr, BinOp};
use std::fmt::Write;
use std::collections::HashMap;

pub fn codegen_to_asm(prog: &Program) -> String {
    let mut out = String::new();
    writeln!(&mut out, "    .intel_syntax noprefix").unwrap();
    writeln!(&mut out, "    .section .rodata").unwrap();

    let mut strs: Vec<String> = Vec::new();
    let mut int_consts: HashMap<String,i64> = HashMap::new();
    let mut str_consts: HashMap<String,String> = HashMap::new();

    // first pass: collect literal lets
    for item in &prog.items {
        if let Stmt::Let { name, value, .. } = item {
            match value {
                Expr::Int(v) => { int_consts.insert(name.clone(), *v); }
                Expr::Str(s) => { str_consts.insert(name.clone(), s.clone()); if !strs.contains(s) { strs.push(s.clone()); } }
                _ => {}
            }
        }
    }

    // iterative constant folding for let bindings
    let mut changed = true;
    while changed {
        changed = false;
        for item in &prog.items {
            if let Stmt::Let { name, value, .. } = item {
                if int_consts.contains_key(name) { continue; }
                if let Some(v) = eval_const_expr_with_ctx(value, &int_consts) {
                    int_consts.insert(name.clone(), v);
                    changed = true;
                }
            }
        }
    }

    collect_strings_with_ctx(prog, &mut strs, &int_consts);
    for (i, s) in strs.iter().enumerate() {
        writeln!(&mut out, "Lmsg{}:", i).unwrap();
        let esc = s.replace('"', "\\\"");
        writeln!(&mut out, "    .ascii \"{}\"", esc).unwrap();
    }

    writeln!(&mut out, "    .section .bss").unwrap();
    writeln!(&mut out, "input_buffer: .space 256").unwrap();

    // allocate space for let variables
    let mut let_names: Vec<String> = Vec::new();
    for item in &prog.items {
        if let Stmt::Let { name, .. } = item {
            if !let_names.contains(name) {
                let_names.push(name.clone());
            }
        }
    }
    // detect let variables that are initialized by `input()` -> need string buffers
    let mut let_string_names: Vec<String> = Vec::new();
    for item in &prog.items {
        if let Stmt::Let { name, value, .. } = item {
            if let Expr::Call { name: fnname, args: _ } = value {
                if fnname == "input" {
                    let_string_names.push(name.clone());
                }
            }
        }
    }
    for name in &let_names {
        writeln!(&mut out, "{}: .quad 0", name).unwrap();
    }
    // allocate buffers and length fields for string lets
    for name in &let_string_names {
        writeln!(&mut out, "{}_buf: .space 256", name).unwrap();
        writeln!(&mut out, "{}_len: .quad 0", name).unwrap();
    }

    writeln!(&mut out, "    .section .text").unwrap();
    writeln!(&mut out, "    .global _start").unwrap();
    writeln!(&mut out, "_start:").unwrap();

    let mut label_counter = 0;
    let mut code = String::new();
    codegen_stmts(&prog.items, &mut code, &strs, &int_consts, &str_consts, &let_names, &let_string_names, &mut label_counter);
    out.push_str(&code);

    writeln!(&mut out, "    mov rax, 60").unwrap();
    writeln!(&mut out, "    xor rdi, rdi").unwrap();
    writeln!(&mut out, "    syscall").unwrap();

    out
}

fn codegen_stmts(items: &[Stmt], out: &mut String, strs: &Vec<String>, int_consts: &HashMap<String,i64>, str_consts: &HashMap<String,String>, let_names: &Vec<String>, let_string_names: &Vec<String>, label_counter: &mut usize) {
    for item in items {
        match item {
            Stmt::Import { .. } => {}
            Stmt::Let { name, value, .. } => {
                // initialize variable at runtime if needed
                // special-case `let name = input();` -> read into name_buf and store length
                if let Expr::Call { name: fnname, args: _ } = value {
                    if fnname == "input" {
                        // syscall: read(0, name_buf, 255)
                        writeln!(out, "    mov rax, 0").unwrap();
                        writeln!(out, "    mov rdi, 0").unwrap();
                        writeln!(out, "    lea rsi, [rip + {}_buf]", name).unwrap();
                        writeln!(out, "    mov rdx, 255").unwrap();
                        writeln!(out, "    syscall").unwrap();
                        // rax = bytes read; store to name_len
                        writeln!(out, "    mov qword ptr [rip + {}_len], rax", name).unwrap();
                        continue;
                    }
                }
                if let Some(v) = eval_const_expr_with_ctx(value, int_consts) {
                    writeln!(out, "    mov rax, {}", v).unwrap();
                    writeln!(out, "    mov qword ptr [rip + {}], rax", name).unwrap();
                } else {
                    gen_expr(value, out, strs, int_consts, str_consts);
                    writeln!(out, "    mov qword ptr [rip + {}], rax", name).unwrap();
                }
            }
            Stmt::Assign { name, value } => {
                gen_expr(value, out, strs, int_consts, str_consts);
                writeln!(out, "    mov qword ptr [rip + {}], rax", name).unwrap();
            }
            Stmt::If { cond, then_body, else_body } => {
                let else_label = format!("Lelse_{}", label_counter);
                let end_label = format!("Lend_{}", label_counter);
                *label_counter += 1;

                // Only constant-fold if the condition does not reference runtime
                // `let` variables. If it references runtime vars we must emit a
                // dynamic check.
                if let Some(val) = eval_const_expr_with_ctx(cond, int_consts) {
                    if !expr_uses_let(cond, let_names) {
                        if val == 0 {
                            if else_body.is_some() { writeln!(out, "    jmp {}", else_label).unwrap(); }
                            else { writeln!(out, "    jmp {}", end_label).unwrap(); }
                        }
                    } else {
                        gen_expr(cond, out, strs, int_consts, str_consts);
                        if else_body.is_some() {
                            writeln!(out, "    cmp rax, 0").unwrap();
                            writeln!(out, "    je {}", else_label).unwrap();
                        } else {
                            writeln!(out, "    cmp rax, 0").unwrap();
                            writeln!(out, "    je {}", end_label).unwrap();
                        }
                    }
                } else {
                    gen_expr(cond, out, strs, int_consts, str_consts);
                    if else_body.is_some() {
                        writeln!(out, "    cmp rax, 0").unwrap();
                        writeln!(out, "    je {}", else_label).unwrap();
                    } else {
                        writeln!(out, "    cmp rax, 0").unwrap();
                        writeln!(out, "    je {}", end_label).unwrap();
                    }
                }

                codegen_stmts(then_body, out, strs, int_consts, str_consts, let_names, let_string_names, label_counter);

                if else_body.is_some() {
                    writeln!(out, "    jmp {}", end_label).unwrap();
                }

                if let Some(eb) = else_body {
                    writeln!(out, "{}:", else_label).unwrap();
                    codegen_stmts(eb, out, strs, int_consts, str_consts, let_names, let_string_names, label_counter);
                }

                writeln!(out, "{}:", end_label).unwrap();
            }
            Stmt::While { cond, body } => {
                let loop_label = format!("Lloop_{}", label_counter);
                let exit_label = format!("Lexit_{}", label_counter);
                *label_counter += 1;

                writeln!(out, "{}:", loop_label).unwrap();

                // Constant-fold only when the condition does not reference runtime
                // variables allocated in `let_names`.
                if let Some(val) = eval_const_expr_with_ctx(cond, int_consts) {
                    if !expr_uses_let(cond, let_names) {
                        if val == 0 { writeln!(out, "    jmp {}", exit_label).unwrap(); }
                    } else {
                        gen_expr(cond, out, strs, int_consts, str_consts);
                        writeln!(out, "    cmp rax, 0").unwrap();
                        writeln!(out, "    je {}", exit_label).unwrap();
                    }
                } else {
                    gen_expr(cond, out, strs, int_consts, str_consts);
                    writeln!(out, "    cmp rax, 0").unwrap();
                    writeln!(out, "    je {}", exit_label).unwrap();
                }

                codegen_stmts(body, out, strs, int_consts, str_consts, let_names, let_string_names, label_counter);
                writeln!(out, "    jmp {}", loop_label).unwrap();
                writeln!(out, "{}:", exit_label).unwrap();
            }
            Stmt::Expr(Expr::Call { name, args }) if name=="print" && args.len()==1 => {
                writeln!(out, "    mov rax, 1").unwrap();
                writeln!(out, "    mov rdi, 1").unwrap();

                match &args[0] {
                    Expr::Str(s) => {
                        if let Some(idx) = strs.iter().position(|x| x==s) {
                            writeln!(out, "    lea rsi, [rip + Lmsg{}]", idx).unwrap();
                            writeln!(out, "    mov rdx, {}", s.as_bytes().len()).unwrap();
                        } else {
                            writeln!(out, "    mov rsi, 0").unwrap();
                            writeln!(out, "    mov rdx, 0").unwrap();
                        }
                    }
                    Expr::Ident(id) => {
                        // If this identifier is a runtime variable (declared with let),
                        // it may be an integer or a string buffer (from input()).
                        if let_string_names.contains(id) {
                            // dynamic string: load pointer and length and write
                            writeln!(out, "    mov rax, 1").unwrap();
                            writeln!(out, "    mov rdi, 1").unwrap();
                            writeln!(out, "    lea rsi, [rip + {}_buf]", id).unwrap();
                            writeln!(out, "    mov rdx, qword ptr [rip + {}_len]", id).unwrap();
                        } else if let_names.contains(id) {
                            // integer variable: evaluate and print simple single-digit
                            gen_expr(&Expr::Ident(id.clone()), out, strs, int_consts, str_consts);
                            writeln!(out, "    lea rsi, [rip + input_buffer]").unwrap();
                            writeln!(out, "    mov rbx, rax").unwrap();
                            writeln!(out, "    add rbx, '0'").unwrap();
                            writeln!(out, "    mov byte ptr [rsi], bl").unwrap();
                            writeln!(out, "    mov rdx, 1").unwrap();
                        } else if let Some(sv) = str_consts.get(id) {
                            if let Some(idx) = strs.iter().position(|x| x==sv) {
                                writeln!(out, "    lea rsi, [rip + Lmsg{}]", idx).unwrap();
                                writeln!(out, "    mov rdx, {}", sv.as_bytes().len()).unwrap();
                            } else {
                                writeln!(out, "    mov rsi, 0").unwrap();
                                writeln!(out, "    mov rdx, 0").unwrap();
                            }
                        } else if let Some(iv) = int_consts.get(id) {
                            let s = iv.to_string();
                            if let Some(idx) = strs.iter().position(|x| x==&s) {
                                writeln!(out, "    lea rsi, [rip + Lmsg{}]", idx).unwrap();
                                writeln!(out, "    mov rdx, {}", s.as_bytes().len()).unwrap();
                            } else {
                                writeln!(out, "    mov rsi, 0").unwrap();
                                writeln!(out, "    mov rdx, 0").unwrap();
                            }
                        } else {
                            writeln!(out, "    mov rsi, 0").unwrap();
                            writeln!(out, "    mov rdx, 0").unwrap();
                        }
                    }
                    Expr::Int(v) => {
                        let s = v.to_string();
                        if let Some(idx) = strs.iter().position(|x| x==&s) {
                            writeln!(out, "    lea rsi, [rip + Lmsg{}]", idx).unwrap();
                            writeln!(out, "    mov rdx, {}", s.as_bytes().len()).unwrap();
                        } else {
                            writeln!(out, "    mov rsi, 0").unwrap();
                            writeln!(out, "    mov rdx, 0").unwrap();
                        }
                    }
                    Expr::BinaryOp{..} => {
                        if let Some(val) = eval_const_expr_with_ctx(&args[0], int_consts) {
                            let s = val.to_string();
                            if let Some(idx) = strs.iter().position(|x| x==&s) {
                                writeln!(out, "    lea rsi, [rip + Lmsg{}]", idx).unwrap();
                                writeln!(out, "    mov rdx, {}", s.as_bytes().len()).unwrap();
                            } else {
                                writeln!(out, "    mov rsi, 0").unwrap();
                                writeln!(out, "    mov rdx, 0").unwrap();
                            }
                        } else {
                            // dynamic expression: evaluate and print simple single-digit
                            // positive integers by converting to a single ASCII digit.
                            gen_expr(&args[0], out, strs, int_consts, str_consts);
                            writeln!(out, "    lea rsi, [rip + input_buffer]").unwrap();
                            writeln!(out, "    mov rbx, rax").unwrap();
                            writeln!(out, "    add rbx, '0'").unwrap();
                            writeln!(out, "    mov byte ptr [rsi], bl").unwrap();
                            writeln!(out, "    mov rdx, 1").unwrap();
                        }
                    }
                    _ => { writeln!(out, "    mov rsi, 0").unwrap(); writeln!(out, "    mov rdx, 0").unwrap(); }
                }

                writeln!(out, "    mov rax, 1").unwrap();
                writeln!(out, "    mov rdi, 1").unwrap();
                writeln!(out, "    syscall").unwrap();
            }
            _ => {}
        }
    }
}

fn gen_expr(e: &Expr, out: &mut String, strs: &Vec<String>, int_consts: &HashMap<String,i64>, str_consts: &HashMap<String,String>) {
    match e {
        Expr::Int(v) => {
            writeln!(out, "    mov rax, {}", v).unwrap();
        }
        Expr::Ident(name) => {
            writeln!(out, "    mov rax, qword ptr [rip + {}]", name).unwrap();
        }
        Expr::ArrayAccess { .. } => {
            writeln!(out, "    mov rax, 0").unwrap(); // placeholder for array access
        }
        Expr::ArrayLiteral(_) => {
            writeln!(out, "    mov rax, 0").unwrap(); // placeholder for array literal
        }
        Expr::BinaryOp { op, left, right } => {
            gen_expr(left, out, strs, int_consts, str_consts);
            writeln!(out, "    push rax").unwrap();
            gen_expr(right, out, strs, int_consts, str_consts);
            writeln!(out, "    mov rbx, rax").unwrap();
            writeln!(out, "    pop rax").unwrap();
            match op {
                BinOp::Add => { writeln!(out, "    add rax, rbx").unwrap(); }
                BinOp::Sub => { writeln!(out, "    sub rax, rbx").unwrap(); }
                BinOp::Mul => { writeln!(out, "    imul rax, rbx").unwrap(); }
                BinOp::Div => {
                    writeln!(out, "    cqo").unwrap();
                    writeln!(out, "    idiv rbx").unwrap();
                }
                BinOp::Lt => {
                    writeln!(out, "    cmp rax, rbx").unwrap();
                    writeln!(out, "    setl al").unwrap();
                    writeln!(out, "    movzx rax, al").unwrap();
                }
                BinOp::Gt => {
                    writeln!(out, "    cmp rax, rbx").unwrap();
                    writeln!(out, "    setg al").unwrap();
                    writeln!(out, "    movzx rax, al").unwrap();
                }
                BinOp::LtEq => {
                    writeln!(out, "    cmp rax, rbx").unwrap();
                    writeln!(out, "    setle al").unwrap();
                    writeln!(out, "    movzx rax, al").unwrap();
                }
                BinOp::GtEq => {
                    writeln!(out, "    cmp rax, rbx").unwrap();
                    writeln!(out, "    setge al").unwrap();
                    writeln!(out, "    movzx rax, al").unwrap();
                }
                BinOp::EqEq => {
                    writeln!(out, "    cmp rax, rbx").unwrap();
                    writeln!(out, "    sete al").unwrap();
                    writeln!(out, "    movzx rax, al").unwrap();
                }
                BinOp::NotEq => {
                    writeln!(out, "    cmp rax, rbx").unwrap();
                    writeln!(out, "    setne al").unwrap();
                    writeln!(out, "    movzx rax, al").unwrap();
                }
            }
        }
        Expr::Str(s) => {
            if let Some(idx) = strs.iter().position(|x| x==s) {
                writeln!(out, "    lea rax, [rip + Lmsg{}]", idx).unwrap();
            } else {
                writeln!(out, "    mov rax, 0").unwrap();
            }
        }
        Expr::Call { name, args } => {
            // Support a simple `input()` call as expression: read up to 255 bytes
            // into `input_buffer` and return the pointer (address) in `rax`.
            if name == "input" {
                // syscall: read(0, input_buffer, 255)
                writeln!(out, "    mov rax, 0").unwrap();
                writeln!(out, "    mov rdi, 0").unwrap();
                writeln!(out, "    lea rsi, [rip + input_buffer]").unwrap();
                writeln!(out, "    mov rdx, 255").unwrap();
                writeln!(out, "    syscall").unwrap();
                // keep bytes read in rbx, return buffer pointer in rax
                writeln!(out, "    mov rbx, rax").unwrap();
                writeln!(out, "    lea rax, [rip + input_buffer]").unwrap();
                writeln!(out, "    mov rax, rax").unwrap();
                writeln!(out, "    mov rax, rax").unwrap();
                // Note: rbx holds length, rax holds pointer
                return;
            }
            // Other calls: no runtime support yet, return 0
            for _a in args { }
            writeln!(out, "    mov rax, 0").unwrap();
        }
    }
}

fn eval_const_expr(e: &Expr) -> Option<i64> {
    match e {
        Expr::Int(v) => Some(*v),
        Expr::BinaryOp { op, left, right } => {
            let l = eval_const_expr(left)?;
            let r = eval_const_expr(right)?;
            match op {
                BinOp::Add => Some(l + r),
                BinOp::Sub => Some(l - r),
                BinOp::Mul => Some(l * r),
                BinOp::Div => if r!=0 { Some(l / r) } else { None },
                BinOp::Lt => Some(if l < r { 1 } else { 0 }),
                BinOp::Gt => Some(if l > r { 1 } else { 0 }),
                BinOp::LtEq => Some(if l <= r { 1 } else { 0 }),
                BinOp::GtEq => Some(if l >= r { 1 } else { 0 }),
                BinOp::EqEq => Some(if l == r { 1 } else { 0 }),
                BinOp::NotEq => Some(if l != r { 1 } else { 0 }),
            }
        }
        _ => None
    }
}

fn eval_const_expr_with_ctx(e: &Expr, ctx: &HashMap<String,i64>) -> Option<i64> {
    match e {
        Expr::Int(v) => Some(*v),
        Expr::Ident(name) => ctx.get(name).copied(),
        Expr::BinaryOp { op, left, right } => {
            let l = eval_const_expr_with_ctx(left, ctx)?;
            let r = eval_const_expr_with_ctx(right, ctx)?;
            match op {
                BinOp::Add => Some(l + r),
                BinOp::Sub => Some(l - r),
                BinOp::Mul => Some(l * r),
                BinOp::Div => if r!=0 { Some(l / r) } else { None },
                BinOp::Lt => Some(if l < r { 1 } else { 0 }),
                BinOp::Gt => Some(if l > r { 1 } else { 0 }),
                BinOp::LtEq => Some(if l <= r { 1 } else { 0 }),
                BinOp::GtEq => Some(if l >= r { 1 } else { 0 }),
                BinOp::EqEq => Some(if l == r { 1 } else { 0 }),
                BinOp::NotEq => Some(if l != r { 1 } else { 0 }),
            }
        }
        _ => None
    }
}

fn collect_strings_with_ctx(prog: &Program, out: &mut Vec<String>, ctx: &HashMap<String,i64>) {
    fn collect_stmts(stmts: &[Stmt], out: &mut Vec<String>, ctx: &HashMap<String,i64>) {
        for item in stmts {
            match item {
                Stmt::Let { name: _, value, .. } => {
                    if let Expr::Str(s) = value {
                        if !out.contains(s) {
                            out.push(s.clone());
                        }
                    }
                }
                Stmt::Expr(Expr::Call { name, args }) => {
                    if name == "print" && args.len()==1 {
                        match &args[0] {
                            Expr::Str(s) => {
                                if !out.contains(s) { out.push(s.clone()); }
                            }
                            Expr::Int(v) => {
                                let s = v.to_string();
                                if !out.contains(&s) { out.push(s); }
                            }
                            Expr::BinaryOp{..} => {
                                if let Some(v) = eval_const_expr_with_ctx(&args[0], ctx) {
                                    let s = v.to_string();
                                    if !out.contains(&s) { out.push(s); }
                                }
                            }
                            Expr::Ident(id) => {
                                if let Some(iv) = ctx.get(id) {
                                    let s = iv.to_string();
                                    if !out.contains(&s) { out.push(s); }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Stmt::If { cond: _, then_body, else_body } => {
                    collect_stmts(then_body, out, ctx);
                    if let Some(eb) = else_body {
                        collect_stmts(eb, out, ctx);
                    }
                }
                Stmt::While { cond: _, body } => {
                    collect_stmts(body, out, ctx);
                }
                _ => {}
            }
        }
    }
    collect_stmts(&prog.items, out, ctx);
}

fn expr_uses_let(e: &Expr, let_names: &Vec<String>) -> bool {
    match e {
        Expr::Ident(name) => let_names.contains(name),
        Expr::BinaryOp { left, right, .. } => expr_uses_let(left, let_names) || expr_uses_let(right, let_names),
        Expr::Call { args, .. } => args.iter().any(|a| expr_uses_let(a, let_names)),
        _ => false
    }
}

pub fn codegen_to_machine_code(prog: &Program) -> (Vec<u8>, Vec<u8>) {
    (vec![], vec![])
}

