use std::{path::Path, rc::Rc};

use crate::{
    fileposition::FilePosition,
    tokens::{Operator, Unary},
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
    Captured,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub id: String,
    pub ttype: TType,
    pub pos: Option<FilePosition>,
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
    Let {
        ttype: TType,
        identifier: String,
        expr: Expr,
        global: bool,
    },
    Function {
        ttype: TType,
        identifier: String,
        parameters: Vec<Arg>,
        body: Vec<Statement>,
        captures: Vec<String>,
    },
    Struct {
        ttype: TType,
        identifier: String,
        fields: Vec<Field>,
    },
    Enum {
        ttype: TType,
        identifier: String,
        fields: Vec<Field>,
    },
    Return {
        ttype: TType,
        expr: Expr,
    },
    Expression {
        ttype: TType,
        expr: Expr,
    },
    If {
        ttype: TType,
        test: Expr,
        body: Vec<Statement>,
        alternative: Option<Vec<Statement>>,
    },
    Unwrap {
        ttype: TType,
        identifier: String,
        body: Vec<Statement>,
        alternative: Option<Vec<Statement>>,
    },
    IfLet {
        ttype: TType,
        identifier: String,
        expr: Expr,
        body: Vec<Statement>,
        alternative: Option<Vec<Statement>>,
        global: bool,
    },
    While {
        test: Expr,
        body: Vec<Statement>,
    },
    For {
        init: Expr,
        test: Expr,
        inc: Expr,
        body: Vec<Statement>,
    },
    Foreach {
        identifier: String,
        expr: Expr,
        body: Vec<Statement>,
    },
    ForRange {
        identifier: String,
        start: Expr,
        end: Expr,
        inclusive: bool,
        step: Option<Expr>,
        body: Vec<Statement>,
    },
    Block {
        body: Vec<Statement>,
        filepath: Option<Rc<Path>>,
    },
    Match {
        ttype: TType,
        expr: Expr,
        arms: Vec<(usize, Option<String>, Vec<Statement>)>,
        default: Option<Vec<Statement>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    None,
    Char {
        value: char,
    },
    Bool {
        value: bool,
    },
    Id {
        name: String,
    },
    Float {
        value: f64,
    },
    String {
        value: String,
    },
    Integer {
        value: i64,
    },
    Call {
        name: String,
        arguments: Vec<Expr>,
        position: FilePosition,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Closure {
        ttype: TType,
        args: Vec<Arg>,
        body: Vec<Statement>,
        captures: Vec<String>,
    },
    ListConstructor {
        ttype: TType,
        elements: Vec<Expr>,
    },
    ListCompConstructor {
        ttype: TType,
        loops: Vec<(String, Expr)>,
        expr: Vec<Expr>,
        guards: Vec<Expr>,
    },
    Field {
        ttype: TType,
        name: String,
        index: usize,
        expr: Box<Expr>,
        position: FilePosition,
    },
    Indexed {
        ttype: TType,
        name: String,
        container: Box<Expr>,
        index: Box<Expr>,
        position: FilePosition,
    },
    Sliced {
        ttype: TType,
        name: String,
        container: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        step: Option<Box<Expr>>,
        position: FilePosition,
    },
    Call {
        ttype: TType,
        name: String,
        function: Box<Expr>,
        args: Vec<Expr>,
    },
    Unary {
        ttype: TType,
        op: Unary,
        expr: Box<Expr>,
    },
    Binop {
        ttype: TType,
        op: Operator,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Literal {
        ttype: TType,
        value: Atom,
    },
    StoreExpr {
        ttype: TType,
        name: String,
        expr: Box<Expr>,
        body: Vec<Statement>,
    },
    None,
}

impl Expr {
    pub fn get_type(&self) -> TType {
        match self {
            Expr::Unary { ttype, .. } => ttype.clone(),
            Expr::Binop { ttype, .. } => ttype.clone(),
            Expr::Literal { ttype, .. } => ttype.clone(),
            Expr::Field { ttype, .. } => ttype.clone(),
            Expr::ListConstructor { ttype, .. } => ttype.clone(),
            Expr::Indexed { ttype, .. } => ttype.clone(),
            Expr::None => TType::None,
            Expr::Call { ttype, .. } => ttype.clone(),
            Expr::Closure { ttype, .. } => ttype.clone(),
            Expr::ListCompConstructor { ttype, .. } => ttype.clone(),
            Expr::Sliced { ttype, .. } => ttype.clone(),
            Expr::StoreExpr { ttype, .. } => ttype.clone(),
        }
    }
}
