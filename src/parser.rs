use crate::lexer::{Lexer, Token};
use crate::ast::{Expr, BinOp, Stmt, Program, Type};

pub struct Parser<'a> {
    lex: Lexer<'a>,
    lookahead: Token,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Self {
        let mut lx = Lexer::new(src);
        let la = lx.next_token();
        Self { lex: lx, lookahead: la }
    }

    fn bump(&mut self) {
        self.lookahead = self.lex.next_token();
    }

    fn parse_type(&mut self) -> Option<Type> {
        match &self.lookahead {
            Token::Ident(name) => {
                let base_name = name.clone();
                self.bump();
                
                // Check for array type (e.g., int[10])
                if self.lookahead == Token::LBracket {
                    self.bump();
                    if let Token::Int(size) = &self.lookahead {
                        let array_size = *size as usize;
                        self.bump();
                        if self.lookahead == Token::RBracket {
                            self.bump();
                            let base_type = match base_name.as_str() {
                                "int" => Box::new(Type::Int),
                                "string" | "str" => Box::new(Type::Str),
                                _ => Box::new(Type::Struct(base_name)),
                            };
                            return Some(Type::Array { base: base_type, size: array_size });
                        }
                    }
                    return None;
                }
                
                // Simple type
                match base_name.as_str() {
                    "int" => Some(Type::Int),
                    "string" | "str" => Some(Type::Str),
                    _ => Some(Type::Struct(base_name)),
                }
            }
            _ => None,
        }
    }

    pub fn parse_program(&mut self) -> Program {
        let mut items = Vec::new();
        while self.lookahead != Token::EOF {
            if let Some(s) = self.parse_stmt() {
                items.push(s);
            } else {
                self.bump();
            }
        }
        Program { items }
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        match &self.lookahead {
            Token::Struct => {
                self.bump();
                if let Token::Ident(name) = &self.lookahead {
                    let struct_name = name.clone();
                    self.bump();
                    if self.lookahead == Token::LBrace {
                        self.bump();
                        let mut fields = Vec::new();
                        while self.lookahead != Token::RBrace && self.lookahead != Token::EOF {
                            if let Token::Ident(field_name) = &self.lookahead {
                                let fname = field_name.clone();
                                self.bump();
                                if self.lookahead == Token::Colon {
                                    self.bump();
                                    if let Some(ftype) = self.parse_type() {
                                        fields.push((fname, ftype));
                                        if self.lookahead == Token::Comma {
                                            self.bump();
                                        }
                                    }
                                }
                            } else {
                                self.bump();
                            }
                        }
                        if self.lookahead == Token::RBrace {
                            self.bump();
                        }
                        if self.lookahead == Token::Semicolon {
                            self.bump();
                        }
                        return Some(Stmt::StructDef { name: struct_name, fields });
                    }
                }
                None
            }
            Token::If => {
                self.bump();
                let cond = self.parse_expr()?;
                // Opcional "then"
                if self.lookahead == Token::Then {
                    self.bump();
                }
                if self.lookahead == Token::LBrace {
                    self.bump();
                    let mut then_body = Vec::new();
                    while self.lookahead != Token::RBrace && self.lookahead != Token::EOF {
                        if let Some(s) = self.parse_stmt() {
                            then_body.push(s);
                        } else {
                            self.bump();
                        }
                    }
                    if self.lookahead == Token::RBrace {
                        self.bump();
                    }
                    let else_body = if self.lookahead == Token::Else {
                        self.bump();
                        // Support `else if` (elif) by allowing `else` followed by `if`.
                        if self.lookahead == Token::If {
                            // Delegate to parse_stmt to parse the nested if.
                            if let Some(s) = self.parse_stmt() {
                                Some(vec![s])
                            } else { None }
                        } else if self.lookahead == Token::LBrace {
                            self.bump();
                            let mut eb = Vec::new();
                            while self.lookahead != Token::RBrace && self.lookahead != Token::EOF {
                                if let Some(s) = self.parse_stmt() {
                                    eb.push(s);
                                } else {
                                    self.bump();
                                }
                            }
                            if self.lookahead == Token::RBrace {
                                self.bump();
                            }
                            Some(eb)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    return Some(Stmt::If { cond, then_body, else_body });
                }
                None
            }
            Token::While => {
                self.bump();
                let cond = self.parse_expr()?;
                if self.lookahead == Token::LBrace {
                    self.bump();
                    let mut body = Vec::new();
                    while self.lookahead != Token::RBrace && self.lookahead != Token::EOF {
                        if let Some(s) = self.parse_stmt() {
                            body.push(s);
                        } else {
                            self.bump();
                        }
                    }
                    if self.lookahead == Token::RBrace {
                        self.bump();
                    }
                    return Some(Stmt::While { cond, body });
                }
                None
            }
            Token::For => {
                self.bump();
                // Expect: for var in range(end) or for var in range(start, end)
                if let Token::Ident(var_name) = &self.lookahead {
                    let var = var_name.clone();
                    self.bump();
                    if self.lookahead == Token::In {
                        self.bump();
                        if self.lookahead == Token::Range {
                            self.bump();
                            if self.lookahead == Token::LParen {
                                self.bump();
                                let first_arg = self.parse_expr()?;
                                let (start, end) = if self.lookahead == Token::Comma {
                                    self.bump();
                                    let second_arg = self.parse_expr()?;
                                    (first_arg, second_arg)
                                } else {
                                    // range(n) means 0..n
                                    (Expr::Int(0), first_arg)
                                };
                                if self.lookahead == Token::RParen { self.bump(); }
                                if self.lookahead == Token::LBrace {
                                    self.bump();
                                    let mut body = Vec::new();
                                    while self.lookahead != Token::RBrace && self.lookahead != Token::EOF {
                                        if let Some(s) = self.parse_stmt() {
                                            body.push(s);
                                        } else { self.bump(); }
                                    }
                                    if self.lookahead == Token::RBrace { self.bump(); }
                                    return Some(Stmt::ForRange { var, start, end, body });
                                }
                            }
                        }
                    }
                }
                None
            }
            Token::Import => {
                self.bump();
                // Esperamos: import "lib_name" ou import "path/to/lib"
                if let Token::Str(path) = &self.lookahead {
                    let p = path.clone();
                    self.bump();
                    if self.lookahead == Token::Semicolon {
                        self.bump();
                    }
                    return Some(Stmt::Import { path: p });
                }
                None
            }
            Token::Use => {
                self.bump();
                // Esperamos: use #library_name
                if self.lookahead == Token::Hash {
                    self.bump();
                    if let Token::Ident(libname) = &self.lookahead {
                        let lib = libname.clone();
                        self.bump();
                        if self.lookahead == Token::Semicolon {
                            self.bump();
                        }
                        return Some(Stmt::Use { lib });
                    }
                }
                None
            }
            Token::Let => {
                self.bump();
                if let Token::Ident(name) = &self.lookahead {
                    let n = name.clone();
                    self.bump();
                    
                    // Check for type annotation (e.g., let x: int = 5)
                    let ty = if self.lookahead == Token::Colon {
                        self.bump();
                        self.parse_type()
                    } else {
                        None
                    };
                    
                    if self.lookahead == Token::Eq {
                        self.bump();
                        if let Some(expr) = self.parse_expr() {
                            if self.lookahead == Token::Semicolon {
                                self.bump();
                            }
                            return Some(Stmt::Let { name: n, ty, value: expr });
                        }
                    }
                }
                None
            }
            Token::Func => {
                self.bump();
                if let Token::Ident(name) = &self.lookahead {
                    let n = name.clone();
                    self.bump();
                    if self.lookahead == Token::LParen {
                        self.bump();
                        let mut params = Vec::new();
                        while self.lookahead != Token::RParen && self.lookahead != Token::EOF {
                            if let Token::Ident(p) = &self.lookahead {
                                params.push(p.clone());
                                self.bump();
                            }
                            if self.lookahead == Token::Comma {
                                self.bump();
                            }
                        }
                        if self.lookahead == Token::RParen {
                            self.bump();
                        }
                        if self.lookahead == Token::LBrace {
                            self.bump();
                            let mut body = Vec::new();
                            while self.lookahead != Token::RBrace && self.lookahead != Token::EOF {
                                if let Some(s) = self.parse_stmt() {
                                    body.push(s);
                                } else {
                                    self.bump();
                                }
                            }
                            if self.lookahead == Token::RBrace {
                                self.bump();
                            }
                            return Some(Stmt::Func { name: n, params, body });
                        }
                    }
                }
                None
            }
            Token::Return => {
                self.bump();
                let e = self.parse_expr();
                if self.lookahead == Token::Semicolon {
                    self.bump();
                }
                return Some(Stmt::Return(e));
            }
            Token::Print => {
                self.bump();
                if self.lookahead == Token::LParen {
                    self.bump();
                    let expr = self.parse_expr().unwrap_or(Expr::Int(0));
                    if self.lookahead == Token::RParen {
                        self.bump();
                    }
                    if self.lookahead == Token::Semicolon {
                        self.bump();
                    }
                    return Some(Stmt::Expr(Expr::Call { name: "print".to_string(), args: vec![expr] }));
                }
                None
            }
            Token::Set => {
                self.bump();
                if let Token::Ident(name) = &self.lookahead {
                    let n = name.clone();
                    self.bump();
                    if self.lookahead == Token::Eq {
                        self.bump();
                        if let Some(expr) = self.parse_expr() {
                            if self.lookahead == Token::Semicolon { self.bump(); }
                            return Some(Stmt::Assign { name: n, value: expr });
                        }
                    }
                }
                None
            }
            _ => {
                // Check if this is an assignment (ident = expr) by trying to parse it
                if let Token::Ident(_name) = &self.lookahead {
                    // We need to peek ahead to check for '=' without consuming tokens
                    // For now, just try parsing as a statement that might be an assignment
                    // This will be handled in the fallthrough
                    if let Some(e) = self.parse_expr() {
                        // After parsing the expression, check if next token is '='
                        // If the expression was just an Ident and next token is '=', treat as assignment
                        if let Expr::Ident(name) = &e {
                            if self.lookahead == Token::Eq {
                                self.bump();
                                if let Some(expr) = self.parse_expr() {
                                    if self.lookahead == Token::Semicolon {
                                        self.bump();
                                    }
                                    return Some(Stmt::Assign { name: name.clone(), value: expr });
                                }
                            }
                        }
                        
                        if self.lookahead == Token::Semicolon {
                            self.bump();
                        }
                        return Some(Stmt::Expr(e));
                    }
                }
                
                if let Some(e) = self.parse_expr() {
                    if self.lookahead == Token::Semicolon {
                        self.bump();
                    }
                    return Some(Stmt::Expr(e));
                }
                None
            }
        }
    }

    fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_binary()
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        match &self.lookahead {
            Token::Int(v) => {
                let val = *v;
                self.bump();
                Some(Expr::Int(val))
            }
            Token::Str(s) => {
                let v = s.clone();
                self.bump();
                Some(Expr::Str(v))
            }
            Token::LBracket => {
                // Array literal: [1, 2, 3]
                self.bump();
                let mut elements = Vec::new();
                while self.lookahead != Token::RBracket && self.lookahead != Token::EOF {
                    if let Some(e) = self.parse_expr() {
                        elements.push(e);
                    }
                    if self.lookahead == Token::Comma {
                        self.bump();
                    }
                }
                if self.lookahead == Token::RBracket {
                    self.bump();
                }
                Some(Expr::ArrayLiteral(elements))
            }
            Token::Ident(name) => {
                let n = name.clone();
                self.bump();
                if self.lookahead == Token::LParen {
                    self.bump();
                    let mut args = Vec::new();
                    while self.lookahead != Token::RParen && self.lookahead != Token::EOF {
                        if let Some(a) = self.parse_expr() {
                            args.push(a);
                        }
                        if self.lookahead == Token::Comma {
                            self.bump();
                        }
                    }
                    if self.lookahead == Token::RParen {
                        self.bump();
                    }
                    return Some(Expr::Call { name: n, args });
                }
                Some(Expr::Ident(n))
            }
            Token::LParen => {
                self.bump();
                let e = self.parse_expr();
                if self.lookahead == Token::RParen {
                    self.bump();
                }
                e
            }
            _ => None,
        }
    }

    fn parse_postfix(&mut self) -> Option<Expr> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.lookahead == Token::LBracket {
                self.bump();
                let index = self.parse_expr()?;
                if self.lookahead == Token::RBracket {
                    self.bump();
                }
                expr = Expr::ArrayAccess { array: Box::new(expr), index: Box::new(index) };
            } else {
                break;
            }
        }
        Some(expr)
    }

    fn parse_binary(&mut self) -> Option<Expr> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Option<Expr> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match &self.lookahead {
                Token::Lt => BinOp::Lt,
                Token::Gt => BinOp::Gt,
                Token::LtEq => BinOp::LtEq,
                Token::GtEq => BinOp::GtEq,
                Token::EqEq => BinOp::EqEq,
                Token::NotEq => BinOp::NotEq,
                _ => break,
            };
            self.bump();
            let right = self.parse_additive()?;
            left = Expr::BinaryOp { op, left: Box::new(left), right: Box::new(right) };
        }
        Some(left)
    }

    fn parse_additive(&mut self) -> Option<Expr> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match &self.lookahead {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.bump();
            let right = self.parse_multiplicative()?;
            left = Expr::BinaryOp { op, left: Box::new(left), right: Box::new(right) };
        }
        Some(left)
    }

    fn parse_multiplicative(&mut self) -> Option<Expr> {
        let mut left = self.parse_postfix()?;
        loop {
            let op = match &self.lookahead {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                _ => break,
            };
            self.bump();
            let right = self.parse_postfix()?;
            left = Expr::BinaryOp { op, left: Box::new(left), right: Box::new(right) };
        }
        Some(left)
    }
}
