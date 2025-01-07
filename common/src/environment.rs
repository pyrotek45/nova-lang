use std::{collections::HashMap, rc::Rc};

use crate::{
    fileposition::FilePosition,
    nodes::{Symbol, SymbolKind},
    table::{self, Table},
    ttype::{generate_unique_string, TType},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    pub captured: Vec<HashMap<Rc<str>, Symbol>>,
    pub custom_types: HashMap<Rc<str>, Vec<(Rc<str>, TType)>>,
    pub enums: table::Table<Rc<str>>,
    pub no_override: table::Table<Rc<str>>,
    pub values: Vec<HashMap<Rc<str>, Symbol>>,
    pub type_alias: HashMap<Rc<str>, TType>,
    pub generic_type_struct: HashMap<Rc<str>, Vec<Rc<str>>>,
    pub generic_type_map: HashMap<Rc<str>, Rc<str>>,
    pub live_generics: Vec<table::Table<Rc<str>>>,
    pub forward_declarations: HashMap<Rc<str>, (Vec<TType>, TType, FilePosition)>,
}

impl Default for Environment {
    fn default() -> Self {
        Environment {
            custom_types: HashMap::default(),
            no_override: Table::new(),
            captured: vec![HashMap::default()],
            values: vec![HashMap::default()],
            type_alias: HashMap::default(),
            generic_type_struct: HashMap::default(),
            generic_type_map: HashMap::default(),
            live_generics: vec![Table::new()],
            enums: Table::new(),
            forward_declarations: HashMap::default(),
        }
    }
}

impl Environment {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn insert_symbol(
        &mut self,
        id: &str,
        ttype: TType,
        pos: Option<FilePosition>,
        kind: SymbolKind,
    ) {
        match kind {
            SymbolKind::GenericFunction => {
                let id: Rc<str> = id.into();
                self.values.last_mut().unwrap().insert(
                    id.clone(),
                    Symbol {
                        id,
                        ttype,
                        pos,
                        kind,
                    },
                );
            }
            SymbolKind::Function => {
                if let TType::Function {
                    parameters: input_types,
                    ..
                } = &ttype
                {
                    let unique_id: Rc<str> = generate_unique_string(id, input_types).into();
                    self.values.last_mut().unwrap().insert(
                        unique_id.clone(),
                        Symbol {
                            id: unique_id,
                            ttype,
                            pos,
                            kind,
                        },
                    );
                } else {
                    panic!("does not have type function");
                }
            }
            _ => {
                let id: Rc<str> = id.into();
                self.values.last_mut().unwrap().insert(
                    id.clone(),
                    Symbol {
                        id,
                        ttype,
                        pos,
                        kind,
                    },
                );
            }
        }
    }

    pub fn has(&mut self, symbol: &str) -> bool {
        if self.forward_declarations.contains_key(symbol) {
            self.forward_declarations.remove(symbol);
            return false;
        }
        self.values.last().unwrap().contains_key(symbol)
    }

    pub fn get(&mut self, symbol: &str) -> Option<Symbol> {
        self.values.last().unwrap().get(symbol).cloned()
    }

    pub fn get_type(&mut self, symbol: &str) -> Option<TType> {
        self.values
            .last()
            .unwrap()
            .get(symbol)
            .map(|s| s.ttype.clone())
    }

    pub fn get_type_capture(&mut self, symbol: &str) -> Option<(TType, Rc<str>, SymbolKind)> {
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
        None
    }

    pub fn get_function_type_capture(
        &mut self,
        symbol: &str,
        arguments: &[TType],
    ) -> Option<(TType, Rc<str>, SymbolKind)> {
        for (i, search) in self.values.iter().rev().enumerate() {
            let id = generate_unique_string(symbol, arguments);
            if let Some(s) = search.get(id.as_str()) {
                if i != 0 {
                    self.captured
                        .last_mut()
                        .unwrap()
                        .insert(s.id.clone(), s.clone());
                }
                if let TType::Function { .. } = s.ttype {
                    return Some((s.ttype.clone(), s.id.clone(), s.kind.clone()));
                }
            } else if let Some(s) = search.get(symbol) {
                if i != 0 {
                    self.captured
                        .last_mut()
                        .unwrap()
                        .insert(s.id.clone(), s.clone());
                }
                if let TType::Function { .. } = s.ttype {
                    return Some((s.ttype.clone(), s.id.clone(), s.kind.clone()));
                }
            }
        }
        None
    }

    pub fn get_function_type(
        &mut self,
        symbol: &str,
        arguments: &[TType],
    ) -> Option<(TType, Rc<str>, SymbolKind)> {
        if let Some(s) = self.values.last().unwrap().get(symbol) {
            if let TType::Function { .. } = s.ttype {
                Some((s.ttype.clone(), s.id.clone(), s.kind.clone()))
            } else {
                None
            }
        } else if let Some(s) = self
            .values
            .last()
            .unwrap()
            .get(generate_unique_string(symbol, arguments).as_str())
        {
            if let TType::Function { .. } = s.ttype {
                Some((s.ttype.clone(), s.id.clone(), s.kind.clone()))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn push_scope(&mut self) {
        let mut scope = HashMap::default();
        self.captured.push(self.captured.last().unwrap().clone());
        for (id, sym) in self.values.last().unwrap().iter() {
            match sym.kind {
                SymbolKind::Function | SymbolKind::GenericFunction | SymbolKind::Constructor => {
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
        //self.live_generics.push(self.live_generics.last().unwrap().clone());
        self.values.push(self.values.last().unwrap().clone());
        //self.captured.push(self.captured.last().unwrap().clone())
    }

    pub fn pop_block(&mut self) {
        //self.live_generics.pop();
        self.values.pop();
        //self.captured.pop();
    }
}
