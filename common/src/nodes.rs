use std::{path::Path, rc::Rc};

use crate::{
    fileposition::FilePosition,
    tokens::{Operator, Unary},
    ttype::TType,
};

/// A pattern for generalized pattern matching (non-enum types).
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Match any value, discard: `_`
    Wildcard,
    /// Bind the value to a variable: `x`
    Variable(Rc<str>),
    /// Match a literal Int: `42`, `-1`
    IntLiteral(i64),
    /// Match a literal Float: `3.14`
    FloatLiteral(f64),
    /// Match a literal String: `"hello"`
    StringLiteral(Rc<str>),
    /// Match a literal Bool: `true`, `false`
    BoolLiteral(bool),
    /// Match a literal Char: `'a'`
    CharLiteral(char),
    /// Match a tuple with sub-patterns: `(p1, p2, ...)`
    Tuple(Vec<Pattern>),
    /// Match a list with exact elements: `[p1, p2, ...]`
    List(Vec<Pattern>),
    /// Match a list with head elements and a rest tail: `[p1, p2, ..rest]`
    ListCons(Vec<Pattern>, Rc<str>),
    /// Match an empty list: `[]`
    EmptyList,
    /// Match any of several alternatives: `1 | 2 | 3`
    Or(Vec<Pattern>),
    /// Match an enum variant: `Red()`, `Leaf(val)` (for user-defined enums)
    Enum {
        variant: Rc<str>,
        binding: Option<Box<Pattern>>,
        /// Resolved tag index (position in enum definition). Set by `resolve_option_patterns`.
        tag: Option<usize>,
    },
    /// Match Option::Some with an inner pattern: `Some(x)`
    OptionSome(Option<Box<Pattern>>),
    /// Match Option::None: `None()`
    OptionNone,
    /// Match a struct by field patterns: `Point { x: 0, y }`
    Struct {
        name: Rc<str>,
        fields: Vec<(Rc<str>, Pattern)>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arg {
    pub identifier: Rc<str>,
    pub ttype: TType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub identifier: Rc<str>,
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
    pub id: Rc<str>,
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
    Function {
        ttype: TType,
        identifier: Rc<str>,
        parameters: Vec<Arg>,
        body: Vec<Statement>,
        captures: Vec<Rc<str>>,
    },
    Struct {
        ttype: TType,
        identifier: Rc<str>,
        fields: Vec<Field>,
    },
    Enum {
        ttype: TType,
        identifier: Rc<str>,
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
        identifier: Rc<str>,
        body: Vec<Statement>,
        alternative: Option<Vec<Statement>>,
    },
    IfLet {
        ttype: TType,
        identifier: Rc<str>,
        expr: Expr,
        body: Vec<Statement>,
        alternative: Option<Vec<Statement>>,
        global: bool,
    },
    While {
        test: Expr,
        body: Vec<Statement>,
    },
    WhileLet {
        identifier: Rc<str>,
        expr: Expr,
        body: Vec<Statement>,
    },
    For {
        init: Expr,
        test: Expr,
        inc: Expr,
        body: Vec<Statement>,
    },
    Foreach {
        identifier: Rc<str>,
        expr: Expr,
        body: Vec<Statement>,
        position: FilePosition,
    },
    ForRange {
        identifier: Rc<str>,
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
        arms: Vec<(usize, Option<Rc<str>>, Vec<Statement>)>,
        default: Option<Vec<Statement>>,
        position: FilePosition,
    },
    /// Generalized pattern match on non-enum types (Int, String, List, Tuple, etc.)
    ValueMatch {
        ttype: TType,
        expr: Expr,
        arms: Vec<(Pattern, Vec<Statement>)>,
        default: Option<Vec<Statement>>,
        position: FilePosition,
    },
    ForwardDec {
        identifier: Rc<str>,
    },
    /// `for (a, b) in list { … }` – destructuring foreach
    ForeachDestructure {
        pattern: Pattern,
        expr: Expr,
        body: Vec<Statement>,
        position: FilePosition,
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
        name: Rc<str>,
    },
    Float {
        value: f64,
    },
    String {
        value: Rc<str>,
    },
    Integer {
        value: i64,
    },
    Call {
        name: Rc<str>,
        arguments: Vec<Expr>,
        position: FilePosition,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Let {
        ttype: TType,
        identifier: Rc<str>,
        expr: Box<Expr>,
        global: bool,
    },
    /// `let (a, b) = expr` – pattern destructuring in let
    LetDestructure {
        ttype: TType,
        pattern: Pattern,
        expr: Box<Expr>,
        position: FilePosition,
    },
    Closure {
        ttype: TType,
        args: Vec<Arg>,
        body: Vec<Statement>,
        captures: Vec<Rc<str>>,
    },
    ListConstructor {
        ttype: TType,
        elements: Vec<Expr>,
    },
    ListCompConstructor {
        ttype: TType,
        loops: Vec<(Rc<str>, Expr)>,
        expr: Vec<Expr>,
        guards: Vec<Expr>,
        position: FilePosition,
    },
    Field {
        ttype: TType,
        name: Rc<str>,
        index: usize,
        expr: Box<Expr>,
        position: FilePosition,
    },
    Indexed {
        ttype: TType,
        name: Rc<str>,
        container: Box<Expr>,
        index: Box<Expr>,
        position: FilePosition,
    },
    Sliced {
        ttype: TType,
        name: Rc<str>,
        container: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        step: Option<Box<Expr>>,
        position: FilePosition,
    },
    Call {
        ttype: TType,
        name: Rc<str>,
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
        name: Rc<str>,
        expr: Box<Expr>,
        body: Vec<Statement>,
    },
    Return {
        ttype: TType,
        expr: Box<Expr>,
    },
    IfExpr {
        ttype: TType,
        test: Box<Expr>,
        body: Box<Expr>,
        alternative: Box<Expr>,
    },
    Block {
        ttype: TType,
        body: Vec<Statement>,
    },
    DynField {
        ttype: TType,
        name: Rc<str>,
        expr: Box<Expr>,
        position: FilePosition,
    },
    MatchExpr {
        ttype: TType,
        expr: Box<Expr>,
        arms: Vec<(usize, Option<Rc<str>>, Vec<Statement>)>,
        default: Option<Vec<Statement>>,
        position: FilePosition,
    },
    /// Generalized pattern match expression on non-enum types
    ValueMatchExpr {
        ttype: TType,
        expr: Box<Expr>,
        arms: Vec<(Pattern, Vec<Statement>)>,
        default: Option<Vec<Statement>>,
        position: FilePosition,
    },
    None,
    Void,
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
            Expr::Return { ttype, .. } => ttype.clone(),
            Expr::IfExpr { ttype, .. } => ttype.clone(),
            Expr::Block { ttype, .. } => ttype.clone(),
            Expr::Let { ttype, .. } => ttype.clone(),
            Expr::LetDestructure { ttype, .. } => ttype.clone(),
            Expr::DynField { ttype, .. } => ttype.clone(),
            Expr::MatchExpr { ttype, .. } => ttype.clone(),
            Expr::ValueMatchExpr { ttype, .. } => ttype.clone(),
            Expr::Void => TType::Void,
        }
    }
}
