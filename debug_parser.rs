fn main() {
    let src = r#"
func main() {
    print("Hello");
}
"#;
    
    let mut parser = wheelc::parser::Parser::new(src);
    let prog = parser.parse_program();
    
    println!("Program items: {}", prog.items.len());
    for (i, item) in prog.items.iter().enumerate() {
        println!("Item {}: {:?}", i, std::any::type_name_of_val(item));
    }
}
