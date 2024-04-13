use std::collections::HashMap;

use crate::{
    fileposition::FilePosition,
    nodes::{Symbol, SymbolKind},
    table,
    ttype::{generate_unique_string, TType},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    pub captured: Vec<HashMap<String, Symbol>>,
    pub custom_types: HashMap<String, Vec<(String, TType)>>,
    pub no_override: table::Table<String>,
    pub values: Vec<HashMap<String, Symbol>>,
    pub type_alias: HashMap<String, TType>,
}

pub fn new_environment() -> Environment {
    Environment {
        custom_types: HashMap::default(),
        no_override: table::new(),
        captured: vec![HashMap::default()],
        values: vec![HashMap::default()],
        type_alias: HashMap::default(),
    }
}

impl Environment {
    pub fn insert_symbol(
        &mut self,
        id: &str,
        ttype: TType,
        pos: Option<FilePosition>,
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
        for (i, search) in self.values.iter().rev().enumerate() {
            if let Some(s) = search.get(symbol) {
                if i != 0 {
                    self.captured
                        .last_mut()
                        .unwrap()
                        .insert(s.id.clone(), s.clone());
                }
                return Some((s.ttype.clone(), s.id.clone(), s.kind.clone()));
            }
        }
        return None;
    }

    pub fn get_function_type_capture(
        &mut self,
        symbol: &str,
        arguments: &[TType],
    ) -> Option<(TType, String, SymbolKind)> {
        for (i, search) in self.values.iter().rev().enumerate() {
            if let Some(s) = search.get(&generate_unique_string(symbol, arguments)) {
                if i != 0 {
                    self.captured
                        .last_mut()
                        .unwrap()
                        .insert(s.id.clone(), s.clone());
                }
                if let TType::Function(_, _) = s.ttype {
                    return Some((
                        s.ttype.clone(),
                        generate_unique_string(symbol, arguments),
                        s.kind.clone(),
                    ));
                }
            } else {
                if let Some(s) = search.get(symbol) {
                    if i != 0 {
                        self.captured
                            .last_mut()
                            .unwrap()
                            .insert(s.id.clone(), s.clone());
                    }
                    if let TType::Function(_, _) = s.ttype {
                        return Some((s.ttype.clone(), s.id.clone(), s.kind.clone()));
                    }
                }
            }
        }
        return None;
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
        self.captured.push(self.captured.last().unwrap().clone());
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
