use std::collections::HashMap;

use crate::{
    table,
    tokens::{generate_unique_string, Operator, Position, TType, Unary},
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
pub struct Env {
    pub captured: Vec<HashMap<String, TType>>,
    pub custom_types: HashMap<String, Vec<(String, TType)>>,
    pub no_override: table::Table<String>,
    pub values: Vec<HashMap<String, Symbol>>,
}

pub fn new_env() -> Env {
    Env {
        custom_types: HashMap::default(),
        no_override: table::new(),
        captured: vec![HashMap::default()],
        values: vec![HashMap::default()],
    }
}

impl Env {
    pub fn insert_symbol(
        &mut self,
        id: &str,
        ttype: TType,
        pos: Option<Position>,
        kind: SymbolKind,
    ) {
        match kind {
            SymbolKind::GenericFunction => {
                self.values.last_mut().unwrap().insert(
                    id.to_string(),
                    Symbol {
                        id: id.to_string(),
                        ttype: ttype,
                        pos: pos,
                        kind: kind,
                    },
                );
            }
            SymbolKind::Function => {
                if let TType::Function(inputtypes, _) = ttype.clone() {
                    self.values.last_mut().unwrap().insert(
                        generate_unique_string(&id, &inputtypes),
                        Symbol {
                            id: generate_unique_string(&id, &inputtypes),
                            ttype: ttype,
                            pos: pos,
                            kind: kind,
                        },
                    );
                } else {
                    panic!("does not have type function")
                }
            }
            _ => {
                self.values.last_mut().unwrap().insert(
                    id.to_string(),
                    Symbol {
                        id: id.to_string(),
                        ttype: ttype,
                        pos: pos,
                        kind: kind,
                    },
                );
            }
        }
    }

    pub fn has(&mut self, symbol: &str) -> bool {
        self.values.last().unwrap().contains_key(symbol)
    }

    pub fn get(&mut self, symbol: &str) -> Option<Symbol> {
        self.values.last().unwrap().get(symbol).cloned()
    }

    pub fn get_type(&mut self, symbol: &str) -> Option<TType> {
        if let Some(s) = self.values.last().unwrap().get(symbol) {
            Some(s.ttype.clone())
        } else {
            None
        }
    }

    pub fn get_type_capture(&mut self, symbol: &str) -> Option<(TType, String, SymbolKind)> {
        if self.values.len() <= 1 {
            return None;
        }
        if let Some(s) = self.values.get(self.values.len() - 2).unwrap().get(symbol) {
            Some((s.ttype.clone(), s.id.clone(), s.kind.clone()))
        } else {
            None
        }
    }

    pub fn get_function_type_capture(
        &mut self,
        symbol: &str,
        arguments: &[TType],
    ) -> Option<(TType, String, SymbolKind)> {
        if self.values.len() <= 1 {
            return None;
        }
        if let Some(s) = self.values.get(self.values.len() - 2).unwrap().get(symbol) {
            if let TType::Function(_, _) = s.ttype {
                Some((s.ttype.clone(), s.id.clone(), s.kind.clone()))
            } else {
                None
            }
        } else {
            if let Some(s) = self
                .values
                .last()
                .unwrap()
                .get(&generate_unique_string(symbol, arguments))
            {
                if let TType::Function(_, _) = s.ttype {
                    Some((s.ttype.clone(), s.id.clone(), s.kind.clone()))
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    pub fn get_function_type(
        &mut self,
        symbol: &str,
        arguments: &[TType],
    ) -> Option<(TType, String, SymbolKind)> {
        if let Some(s) = self.values.last().unwrap().get(symbol) {
            if let TType::Function(_, _) = s.ttype {
                Some((s.ttype.clone(), s.id.clone(), s.kind.clone()))
            } else {
                None
            }
        } else {
            if let Some(s) = self
                .values
                .last()
                .unwrap()
                .get(&generate_unique_string(symbol, arguments))
            {
                if let TType::Function(_, _) = s.ttype {
                    Some((s.ttype.clone(), s.id.clone(), s.kind.clone()))
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    pub fn push_scope(&mut self) {
        let mut scope: HashMap<String, Symbol> = HashMap::default();
        self.captured.push(HashMap::default());
        for (id, sym) in self.values.last().unwrap().iter() {
            match sym.kind {
                SymbolKind::Function => {
                    scope.insert(id.clone(), sym.clone());
                }
                SymbolKind::GenericFunction => {
                    scope.insert(id.clone(), sym.clone());
                }
                SymbolKind::Constructor => {
                    scope.insert(id.clone(), sym.clone());
                }
                _ => {}
            }
        }
        self.values.push(scope)
    }

    pub fn pop_scope(&mut self) {
        self.values.pop();
        self.captured.pop();
    }

    pub fn push_block(&mut self) {
        self.values.push(self.values.last().unwrap().clone());
        self.captured.push(self.captured.last().unwrap().clone())
    }

    pub fn pop_block(&mut self) {
        self.values.pop();
        self.captured.pop();
    }
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
    Field(TType, String, usize, Box<Expr>),
    Indexed(TType, String, Box<Expr>, Box<Expr>),
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
            Expr::Field(t, _, _, _) => t.clone(),
            Expr::ListConstructor(t, _) => t.clone(),
            Expr::Indexed(t, _, _, _) => t.clone(),
            Expr::None => TType::None,
            Expr::Call(t, _, _, _) => t.clone(),
            Expr::Closure(t, _, _, _) => t.clone(),
        }
    }
}
