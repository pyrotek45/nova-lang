use crate::{
    tokens::{Operator, Position, Unary},
    ttype::TType,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Arg {
    pub identifier: String,
    pub ttype: TType,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub identifier: String,
    pub ttype: TType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub input: Vec<Arg>,
    pub output: TType,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Function,
    GenericFunction,
    Variable,
    Constructor,
    Parameter,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub id: String,
    pub ttype: TType,
    pub pos: Option<Position>,
    pub kind: SymbolKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ast {
    pub program: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Continue,
    Break,
    Pass,
    // type id value
    Let(TType, String, Expr, bool),
    // type id input output
    Function(TType, String, Vec<Arg>, Vec<Statement>),
    // type id fields
    Struct(TType, String, Vec<Field>),
    // type exression
    Return(TType, Expr, usize, usize),
    Expression(TType, Expr),
    // type test body {else}
    If(TType, Expr, Vec<Statement>, Option<Vec<Statement>>),
    While(Expr, Vec<Statement>),
    For(Expr, Expr, Expr, Vec<Statement>),
    Block(Vec<Statement>, String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    None,
    Char(char),
    Bool(bool),
    Id(String),
    Float(f64),
    String(String),
    Integer(i64),
    Call(String, Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Closure(TType, Vec<Arg>, Vec<Statement>, Vec<String>),
    ListConstructor(TType, Vec<Expr>),
    Field(TType, String, usize, Box<Expr>, Position),
    Indexed(TType, String, Box<Expr>, Box<Expr>, Position),
    Call(TType, String, Box<Expr>, Vec<Expr>),
    Unary(TType, Unary, Box<Expr>),
    Binop(TType, Operator, Box<Expr>, Box<Expr>),
    Literal(TType, Atom),
    None,
}

impl Expr {
    pub fn get_type(&self) -> TType {
        match self {
            Expr::Unary(t, _, _) => t.clone(),
            Expr::Binop(t, _, _, _) => t.clone(),
            Expr::Literal(t, _) => t.clone(),
            Expr::Field(t, _, _, _, _) => t.clone(),
            Expr::ListConstructor(t, _) => t.clone(),
            Expr::Indexed(t, _, _, _, _) => t.clone(),
            Expr::None => TType::None,
            Expr::Call(t, _, _, _) => t.clone(),
            Expr::Closure(t, _, _, _) => t.clone(),
        }
    }
}
