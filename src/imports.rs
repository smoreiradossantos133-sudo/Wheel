use std::collections::HashSet;
use std::path::Path;
use crate::ast::{Program, Stmt};
use std::fs;

pub fn process_imports(prog: &mut Program, base_dir: &Path, processed: &mut HashSet<String>) -> anyhow::Result<()> {
    let mut imported_stmts = Vec::new();
    let mut remaining_stmts = Vec::new();

    for stmt in prog.items.drain(..) {
        if let Stmt::Import { path } = &stmt {
            if !processed.contains(path) {
                processed.insert(path.clone());
                let import_path = if path.ends_with(".wheel") {
                    base_dir.join(path)
                } else {
                    base_dir.join(format!("{}.wheel", path))
                };

                if import_path.exists() {
                    let import_src = fs::read_to_string(&import_path)?;
                    let mut import_parser = crate::parser::Parser::new(&import_src);
                    let mut imported_prog = import_parser.parse_program();
                    process_imports(&mut imported_prog, import_path.parent().unwrap_or(base_dir), processed)?;
                    imported_stmts.extend(imported_prog.items);
                }
            }
        } else {
            remaining_stmts.push(stmt);
        }
    }

    // First include imported statements, then original
    prog.items = imported_stmts;
    prog.items.extend(remaining_stmts);
    Ok(())
}
