#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Str,
    Array { base: Box<Type>, size: usize },
    Struct(String),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Str(String),
    Ident(String),
    BinaryOp { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    Call { name: String, args: Vec<Expr> },
    ArrayAccess { array: Box<Expr>, index: Box<Expr> },
    ArrayLiteral(Vec<Expr>),
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp { 
    Add, Sub, Mul, Div,
    Lt, Gt, LtEq, GtEq, EqEq, NotEq,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    Let { name: String, ty: Option<Type>, value: Expr },
    Assign { name: String, value: Expr },
    ArrayAssign { array: String, index: Expr, value: Expr },
    Func { name: String, params: Vec<String>, body: Vec<Stmt> },
    Return(Option<Expr>),
    Import { path: String },
    Use { lib: String },
    If { cond: Expr, then_body: Vec<Stmt>, else_body: Option<Vec<Stmt>> },
    While { cond: Expr, body: Vec<Stmt> },
    ForRange { var: String, start: Expr, end: Expr, body: Vec<Stmt> },
    StructDef { name: String, fields: Vec<(String, Type)> },
}

#[derive(Debug, Clone)]
pub struct Program { pub items: Vec<Stmt> }
