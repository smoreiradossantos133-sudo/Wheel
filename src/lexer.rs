#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Int(i64),
    Str(String),
    Ident(String),
    Plus, Minus, Star, Slash, Percent,
    LParen, RParen, LBrace, RBrace, Comma,
    LBracket, RBracket, Colon,
    Let, Func, Return, Import, From, Use,
    Print, If, Else, Then, While, For, In, Range,
    Set, Struct,
    Eq,
    Lt, Gt, LtEq, GtEq, EqEq, NotEq,
    Hash,
    Semicolon,
    EOF,
}

pub struct Lexer<'a> { src: &'a str, pos: usize }

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self { Self { src, pos: 0 } }

    fn peek(&self) -> Option<char> { self.src[self.pos..].chars().next() }
    fn bump(&mut self) { if let Some(ch) = self.peek() { self.pos += ch.len_utf8(); } }

    pub fn next_token(&mut self) -> Token {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() { self.bump(); continue }
            if ch.is_ascii_digit() {
                let start = self.pos;
                while let Some(c) = self.peek() { if c.is_ascii_digit() { self.bump(); } else { break } }
                let s = &self.src[start..self.pos];
                let v = s.parse::<i64>().unwrap_or(0);
                return Token::Int(v);
            }
            if ch.is_ascii_alphabetic() || ch == '_' {
                let start = self.pos;
                while let Some(c) = self.peek() { if c.is_ascii_alphanumeric() || c == '_' { self.bump(); } else { break } }
                let s = &self.src[start..self.pos];
                return match s {
                    "let" => Token::Let,
                    "func" => Token::Func,
                    "return" => Token::Return,
                    "import" => Token::Import,
                    "from" => Token::From,
                    "use" => Token::Use,
                    "in" => Token::In,
                    "range" => Token::Range,
                    "set" => Token::Set,
                    "struct" => Token::Struct,
                    "print" => Token::Print,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "then" => Token::Then,
                    "while" => Token::While,
                    "for" => Token::For,
                    _ => Token::Ident(s.to_string()),
                }
            }
            match ch {
                '"' => {
                    // parse string literal
                    self.bump();
                    let start = self.pos;
                    while let Some(c) = self.peek() {
                        if c == '"' { break }
                        self.bump();
                    }
                    let s = &self.src[start..self.pos];
                    if self.peek() == Some('"') { self.bump(); }
                    return Token::Str(s.to_string());
                }
                '+' => { self.bump(); return Token::Plus }
                '-' => { self.bump(); return Token::Minus }
                '*' => { self.bump(); return Token::Star }
                '/' => { self.bump(); return Token::Slash }
                '(' => { self.bump(); return Token::LParen }
                ')' => { self.bump(); return Token::RParen }
                '{' => { self.bump(); return Token::LBrace }
                '}' => { self.bump(); return Token::RBrace }
                '[' => { self.bump(); return Token::LBracket }
                ']' => { self.bump(); return Token::RBracket }
                ',' => { self.bump(); return Token::Comma }
                ':' => { self.bump(); return Token::Colon }
                '=' => {
                    self.bump();
                    if self.peek() == Some('=') {
                        self.bump();
                        return Token::EqEq;
                    }
                    return Token::Eq;
                }
                '<' => {
                    self.bump();
                    if self.peek() == Some('=') {
                        self.bump();
                        return Token::LtEq;
                    }
                    return Token::Lt;
                }
                '>' => {
                    self.bump();
                    if self.peek() == Some('=') {
                        self.bump();
                        return Token::GtEq;
                    }
                    return Token::Gt;
                }
                '!' => {
                    self.bump();
                    if self.peek() == Some('=') {
                        self.bump();
                        return Token::NotEq;
                    }
                    return Token::Ident("!".to_string()); // fallback
                }
                '%' => { self.bump(); return Token::Percent }
                '#' => { self.bump(); return Token::Hash }
                ';' => { self.bump(); return Token::Semicolon }
                _ => { self.bump(); continue }
            }
        }
        Token::EOF
    }
}
