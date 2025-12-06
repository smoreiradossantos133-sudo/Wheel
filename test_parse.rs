use std::fs;

mod lexer;
mod parser;
mod ast;

use parser::Parser;

fn main() {
    let src = fs::read_to_string("examples/hello.wheel").unwrap();
    let mut p = Parser::new(&src);
    let prog = p.parse_program();
    
    println!("Program has {} items", prog.items.len());
    for (i, item) in prog.items.iter().enumerate() {
        println!("Item {}: {:?}", i, item);
    }
}
