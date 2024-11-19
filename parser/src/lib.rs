use std::collections::HashMap;

use common::{
    environment::{new_environment, Environment},
    error::NovaError,
    fileposition::FilePosition,
    nodes::{Arg, Ast, Atom, Expr, Field, Statement, Symbol, SymbolKind},
    table::{self, Table},
    tokens::{Operator, Token, TokenList, Unary},
    ttype::{generate_unique_string, TType},
};
use dym::Lexicon;
use lexer::Lexer;

fn extract_current_directory(path: &str) -> Option<String> {
    if let Some(last_slash_index) = path.rfind('/') {
        return Some(path[..last_slash_index + 1].to_string());
    }
    None
}

#[derive(Debug, Clone)]
pub struct Parser {
    filepath: String,
    pub input: TokenList,
    index: usize,
    pub ast: Ast,
    pub environment: Environment,
}

pub fn new(filepath: &str) -> Parser {
    let mut env = new_environment();
    env.insert_symbol(
        "exit",
        TType::Function {
            parameters: vec![TType::None],
            return_type: Box::new(TType::Void),
        },
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "typeof",
        TType::Function {
            parameters: vec![TType::Generic {
                name: "a".to_string(),
            }],
            return_type: Box::new(TType::String),
        },
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "strlen",
        TType::Function {
            parameters: vec![TType::String],
            return_type: Box::new(TType::Int),
        },
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "isSome",
        TType::Function {
            parameters: vec![TType::Option {
                inner: Box::new(TType::Generic {
                    name: "a".to_string(),
                }),
            }],
            return_type: Box::new(TType::Bool),
        },
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "unwrap",
        TType::Function {
            parameters: vec![TType::Option {
                inner: Box::new(TType::Generic {
                    name: "a".to_string(),
                }),
            }],
            return_type: Box::new(TType::Generic {
                name: "a".to_string(),
            }),
        },
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "Some",
        TType::Function {
            parameters: vec![TType::Generic {
                name: "a".to_string(),
            }],
            return_type: Box::new(TType::Option {
                inner: Box::new(TType::Generic {
                    name: "a".to_string(),
                }),
            }),
        },
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "free",
        TType::Function {
            parameters: vec![TType::Any],
            return_type: Box::new(TType::Void),
        },
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "print",
        TType::Function {
            parameters: vec![TType::Generic {
                name: "a".to_string(),
            }],
            return_type: Box::new(TType::Void),
        },
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "println",
        TType::Function {
            parameters: vec![TType::Generic {
                name: "a".to_string(),
            }],
            return_type: Box::new(TType::Void),
        },
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "clone",
        TType::Function {
            parameters: vec![TType::Generic {
                name: "a".to_string(),
            }],
            return_type: Box::new(TType::Generic {
                name: "a".to_string(),
            }),
        },
        None,
        SymbolKind::GenericFunction,
    );

    Parser {
        filepath: filepath.to_string(),
        ast: Ast { program: vec![] },
        input: vec![],
        index: 0,
        environment: env,
    }
}

impl Parser {
    fn check_and_map_types(
        &self,
        type_list1: &[TType],
        type_list2: &[TType],
        type_map: &mut HashMap<String, TType>,
        pos: FilePosition,
    ) -> Result<HashMap<String, TType>, NovaError> {
        if type_list1.len() != type_list2.len() {
            return Err(self.generate_error_with_pos(
                "E2 Incorrect amount of arguments".to_owned(),
                format!("Found {:?} , but expecting {:?}", type_list2, type_list1),
                pos,
            ));
        }
        for (t1, t2) in type_list1.iter().zip(type_list2.iter()) {
            match (t1, t2) {
                (TType::Any, _) => {
                    continue;
                }
                (_, TType::Any) => {
                    continue;
                }
                (TType::Generic { name: name1 }, _) => {
                    if t2 == &TType::None {
                        return Err(NovaError::TypeMismatch {
                            expected: t1.clone(),
                            found: t2.clone(),
                            position: pos.clone(),
                        });
                    }
                    if t2 == &TType::Void {
                        return Err(NovaError::TypeMismatch {
                            expected: t1.clone(),
                            found: t2.clone(),
                            position: pos.clone(),
                        });
                    }
                    if let TType::Option { .. } = t2 {
                        return Err(NovaError::TypeMismatch {
                            expected: t1.clone(),
                            found: t2.clone(),
                            position: pos.clone(),
                        });
                    }
                    if let Some(mapped_type) = type_map.get(name1) {
                        // If the types are not equal, return an error
                        if mapped_type != t2 {
                            return Err(NovaError::TypeMismatch {
                                expected: t1.clone(),
                                found: t2.clone(),
                                position: pos.clone(),
                            });
                        }
                    } else {
                        // If name1 is not in the type_map, insert it with the corresponding type (t2)
                        type_map.insert(name1.clone(), t2.clone());
                    }
                }

                (TType::List { inner: inner1 }, TType::List { inner: inner2 }) => {
                    self.check_and_map_types(
                        &[*inner1.clone()],
                        &[*inner2.clone()],
                        type_map,
                        pos.clone(),
                    )?;
                }
                (TType::Option { inner: inner1 }, TType::Option { inner: inner2 }) => {
                    self.check_and_map_types(
                        &[*inner1.clone()],
                        &[*inner2.clone()],
                        type_map,
                        pos.clone(),
                    )?;
                }
                (
                    TType::Function {
                        parameters: params1,
                        return_type: ret1,
                    },
                    TType::Function {
                        parameters: params2,
                        return_type: ret2,
                    },
                ) => {
                    if params1.len() != params2.len() {
                        return Err(NovaError::TypeMismatch {
                            expected: t1.clone(),
                            found: t2.clone(),
                            position: pos.clone(),
                        });
                    }

                    if let (Err(_), Err(_)) = (
                        self.check_and_map_types(params1, params2, type_map, pos.clone()),
                        self.check_and_map_types(
                            &[*ret1.clone()],
                            &[*ret2.clone()],
                            type_map,
                            pos.clone(),
                        ),
                    ) {
                        return Err(NovaError::TypeMismatch {
                            expected: t1.clone(),
                            found: t2.clone(),
                            position: pos.clone(),
                        });
                    }
                }
                (TType::Custom { name: custom1 }, TType::Custom { name: custom2 }) => {
                    //self.check_and_map_types(&gen1, &gen2, type_map, pos.clone())?;
                    if custom1 == custom2 {
                        continue;
                    }

                    if let Some(subtype) = self.environment.generic_type_map.get(custom2) {
                        if subtype == custom1 {
                            continue;
                        }
                    } else {
                        return Err(NovaError::TypeMismatch {
                            expected: t1.clone(),
                            found: t2.clone(),
                            position: pos.clone(),
                        });
                    }
                }
                _ if t1 == t2 => continue,
                _ => {
                    return Err(NovaError::TypeMismatch {
                        expected: t1.clone(),
                        found: t2.clone(),
                        position: pos.clone(),
                    });
                }
            }
        }
        Ok(type_map.clone())
    }

    pub fn get_output(
        &self,
        output: TType,
        type_map: &mut HashMap<String, TType>,
    ) -> Result<TType, NovaError> {
        match output {
            TType::Generic { name } => {
                if let Some(mapped_type) = type_map.get(&name) {
                    Ok(mapped_type.clone())
                } else {
                    Ok(TType::Generic { name: name.clone() })
                }
            }
            TType::List { inner } => {
                let mapped_inner = self.get_output(*inner.clone(), type_map)?;
                Ok(TType::List {
                    inner: Box::new(mapped_inner),
                })
            }
            TType::Option { inner } => {
                let mapped_inner = self.get_output(*inner.clone(), type_map)?;
                Ok(TType::Option {
                    inner: Box::new(mapped_inner),
                })
            }
            TType::Function {
                parameters: args,
                return_type,
            } => {
                let mut mapped_args = Vec::new();
                for arg in args {
                    let mapped_arg = self.get_output(arg, type_map)?;
                    mapped_args.push(mapped_arg);
                }

                let mapped_return_type = self.get_output(*return_type.clone(), type_map)?;

                Ok(TType::Function {
                    parameters: mapped_args,
                    return_type: Box::new(mapped_return_type),
                })
            }
            _ => Ok(output.clone()),
        }
    }

    fn eof(&mut self) -> Result<(), NovaError> {
        if matches!(self.current_token(), Token::EOF { .. }) {
            Ok(())
        } else {
            Err(NovaError::Parsing {
                msg: "Parsing not completed, left over tokens unparsed".to_string(),
                note: "Make sure your statement ends with ';'.".to_string(),
                position: self.get_current_token_position(),
                extra: None,
            })
        }
    }

    fn is_current_eof(&mut self) -> bool {
        matches!(self.current_token(), Token::EOF { .. })
    }

    fn generate_error(&self, msg: String, note: String) -> NovaError {
        NovaError::Parsing {
            msg,
            note,
            position: self.get_current_token_position(),
            extra: None,
        }
    }

    fn generate_error_with_pos(&self, msg: String, note: String, pos: FilePosition) -> NovaError {
        NovaError::Parsing {
            msg,
            note,
            position: pos,
            extra: None,
        }
    }

    fn get_line_and_row(&self) -> (usize, usize) {
        let line = self.current_token().line();
        let row = self.current_token().row();
        (line, row)
    }

    fn get_current_token_position(&self) -> FilePosition {
        self.current_token().position()
    }

    fn consume_operator(&mut self, op: Operator) -> Result<(), NovaError> {
        if let Token::Operator { operator, .. } = self.current_token() {
            if op == operator {
                self.advance();
                return Ok(());
            }
        }
        Err(self.generate_error(
            format!("unexpected operator, got {:?}", self.current_token()),
            format!("expecting {:?}", op),
        ))
    }

    fn consume_symbol(&mut self, sym: char) -> Result<(), NovaError> {
        if let Token::Symbol { symbol, .. } = self.current_token() {
            if sym == symbol {
                self.advance();
                return Ok(());
            }
        }
        Err(self.generate_error(
            format!("unexpected symbol, got {:?}", self.current_token()),
            format!("expecting {:?}", sym),
        ))
    }

    fn consume_identifier(&mut self, symbol: Option<&str>) -> Result<(), NovaError> {
        match self.current_token() {
            Token::Identifier { name: sym, .. } if symbol.map_or(true, |s| sym == s) => {
                self.advance();
                Ok(())
            }
            _ => {
                let current_token = self.current_token();
                return Err(self.generate_error(
                    format!("unexpected identifier, got {:?}", current_token.to_string()),
                    match symbol {
                        Some(s) => format!("expecting {:?}", s),
                        None => "expecting an identifier".to_string(),
                    },
                ));
            }
        }
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn current_token(&self) -> Token {
        self.input[self.index].clone()
    }

    fn sign(&mut self) -> Result<Option<Unary>, NovaError> {
        match self.current_token() {
            Token::Operator { operator, .. } => match operator {
                Operator::Addition => Ok(Some(Unary::Positive)),
                Operator::Subtraction => Ok(Some(Unary::Negitive)),
                Operator::Not => Ok(Some(Unary::Not)),
                _ => {
                    return Err(self.generate_error(
                        format!("unexpected operation, got {:?}", self.current_token()),
                        format!("expected unary sign ( + | - )"),
                    ));
                }
            },
            _ => Ok(None),
        }
    }

    fn tuple_list(&mut self) -> Result<Vec<Expr>, NovaError> {
        let mut exprs = vec![];
        self.consume_symbol('(')?;
        if !self.current_token().is_symbol(')') {
            let pos = self.get_current_token_position();
            let e = self.expr()?;
            if e.get_type() != TType::Void {
                exprs.push(e);
            } else {
                return Err(self.generate_error_with_pos(
                    format!("cannot insert a void expression"),
                    format!("tuple expressions must not be void"),
                    pos,
                ));
            }
        }
        while self.current_token().is_symbol(',') {
            if self.current_token().is_symbol(')') {
                break;
            }
            self.advance();
            let pos = self.get_current_token_position();
            if self.current_token().is_symbol(')') {
                break;
            }
            let e = self.expr()?;
            if e.get_type() != TType::Void {
                exprs.push(e);
            } else {
                return Err(self.generate_error_with_pos(
                    format!("cannot insert a void expression"),
                    format!("tuple expressions must not be void"),
                    pos,
                ));
            }
        }
        self.consume_symbol(')')?;
        Ok(exprs)
    }

    fn expr_list(&mut self) -> Result<Vec<Expr>, NovaError> {
        let mut exprs = vec![];
        self.consume_symbol('[')?;
        if !self.current_token().is_symbol(']') {
            let pos = self.get_current_token_position();
            let e = self.expr()?;
            if e.get_type() != TType::Void {
                exprs.push(e);
            } else {
                return Err(self.generate_error_with_pos(
                    format!("cannot insert a void expression"),
                    format!("List expressions must not be void"),
                    pos,
                ));
            }
        }
        while self.current_token().is_symbol(',') {
            if self.current_token().is_symbol(']') {
                break;
            }
            self.advance();
            let pos = self.get_current_token_position();
            if self.current_token().is_symbol(']') {
                break;
            }
            let e = self.expr()?;
            if e.get_type() != TType::Void {
                exprs.push(e);
            } else {
                return Err(self.generate_error_with_pos(
                    format!("cannot insert a void expression"),
                    format!("List expressions must not be void"),
                    pos,
                ));
            }
        }
        self.consume_symbol(']')?;
        Ok(exprs)
    }

    fn argument_list(&mut self) -> Result<Vec<Expr>, NovaError> {
        let mut exprs = vec![];
        self.consume_symbol('(')?;
        if !self.current_token().is_symbol(')') {
            exprs.push(self.expr()?);
        }
        while self.current_token().is_symbol(',') {
            self.advance();
            if self.current_token().is_symbol(')') {
                break;
            }
            exprs.push(self.expr()?);
        }
        self.consume_symbol(')')?;
        Ok(exprs)
    }

    fn field_list(
        &mut self,
        constructor: &str,
        fields: Vec<(String, TType)>,
        conpos: FilePosition,
    ) -> Result<Vec<Expr>, NovaError> {
        let mut exprs: HashMap<String, Expr> = HashMap::default();

        self.consume_symbol('{')?;

        let (id, pos) = self.get_identifier()?;
        self.consume_operator(Operator::Colon)?;
        exprs.insert(id.clone(), self.expr()?);

        while self.current_token().is_symbol(',') {
            self.advance();
            if self.current_token().is_symbol('}') {
                break;
            }

            if self.current_token().is_symbol('}') {
                break;
            }
            let (id, _) = self.get_identifier()?;
            self.consume_operator(Operator::Colon)?;
            exprs.insert(id.clone(), self.expr()?);
            if self.current_token().is_symbol('}') {
                break;
            }
        }

        self.consume_symbol('}')?;

        let mut new_exprs = vec![];

        for (fieldname, fieldtype) in fields.iter() {
            if fieldname == "type" {
                continue;
            }
            if let Some(innerexpr) = exprs.get(fieldname) {
                self.check_and_map_types(
                    &vec![fieldtype.clone()],
                    &vec![innerexpr.get_type()],
                    &mut HashMap::default(),
                    pos.clone(),
                )?;
                new_exprs.push(innerexpr.clone())
            } else {
                return Err(NovaError::Parsing {
                    msg: format!("{} is missing field {} ", constructor, fieldname.clone()),
                    note: format!(""),
                    position: FilePosition {
                        line: conpos.line,
                        row: conpos.row,
                        filepath: self.filepath.clone(),
                    },
                    extra: None,
                });
            }
        }

        if exprs.len() != fields.len() - 1 {
            return Err(NovaError::Parsing {
                msg: format!(
                    "{} has {} fields, you have {} ",
                    constructor,
                    fields.len() - 1,
                    exprs.len()
                ),
                note: format!(""),
                position: FilePosition {
                    line: conpos.line,
                    row: conpos.row,
                    filepath: self.filepath.clone(),
                },
                extra: None,
            });
        }

        if new_exprs.len() != fields.len() - 1 {
            return Err(NovaError::Parsing {
                msg: format!(
                    "{} has {} fields, not all of them are covered",
                    constructor,
                    fields.len() - 1
                ),
                note: String::new(),
                position: conpos,
                extra: None,
            });
        }

        Ok(new_exprs)
    }

    fn method(
        &mut self,
        identifier: String,
        first_argument: Expr,
        pos: FilePosition,
    ) -> Result<Expr, NovaError> {
        let mut arguments = vec![first_argument];
        arguments.extend(self.argument_list()?);
        let mut argument_types: Vec<TType> = arguments.iter().map(|t| t.get_type()).collect();

        if argument_types.is_empty() {
            argument_types.push(TType::None)
        }

        self.varargs(&identifier, &mut argument_types, &mut arguments);

        if let Some((
            TType::Function {
                parameters,
                mut return_type,
            },
            mut function_id,
            function_kind,
        )) = self
            .environment
            .get_function_type(&identifier, &argument_types)
        {
            match function_kind {
                SymbolKind::GenericFunction => {
                    let mut type_map = self.check_and_map_types(
                        &parameters,
                        &argument_types,
                        &mut HashMap::default(),
                        pos.clone(),
                    )?;
                    for (l, r) in parameters.iter().zip(argument_types.iter()) {
                        if let (TType::Custom { name: lc, .. }, TType::Custom { name: rc, .. }) =
                            (l, r)
                        {
                            if let Some(ic) = self.environment.generic_type_map.get(rc) {
                                if lc == ic {
                                    if let Some(list) = self.environment.get_type(&lc) {
                                        let mut s = self.clone();
                                        if let Some(outerlist) = s.environment.get_type(&rc) {
                                            type_map = self.check_and_map_types(
                                                &[list],
                                                &[outerlist],
                                                &mut type_map,
                                                pos.clone(),
                                            )?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    return_type = Box::new(self.get_output(*return_type, &mut type_map)?);
                    return Ok(Expr::Literal {
                        ttype: *return_type.clone(),
                        value: Atom::Call {
                            name: function_id,
                            arguments,
                        },
                    });
                }
                SymbolKind::Constructor
                | SymbolKind::Variable
                | SymbolKind::Parameter
                | SymbolKind::Function => {
                    let mut type_map = self.check_and_map_types(
                        &parameters,
                        &argument_types,
                        &mut HashMap::default(),
                        pos,
                    )?;
                    return_type = Box::new(self.get_output(*return_type, &mut type_map)?);
                    if let Some(subtype) = self.environment.generic_type_map.get(&function_id) {
                        function_id = subtype.clone();
                    }
                    return_type = Box::new(self.get_output(*return_type, &mut type_map)?);
                    return Ok(Expr::Literal {
                        ttype: *return_type.clone(),
                        value: Atom::Call {
                            name: function_id,
                            arguments,
                        },
                    });
                }
            }
        } else {
            if let Some((
                TType::Function {
                    parameters,
                    mut return_type,
                },
                mut function_id,
                function_kind,
            )) = self.environment.get_type_capture(&identifier)
            {
                let pos = self.get_current_token_position();
                self.environment.captured.last_mut().unwrap().insert(
                    identifier.clone(),
                    Symbol {
                        id: identifier.clone(),
                        ttype: TType::Function {
                            parameters: parameters.clone(),
                            return_type: return_type.clone(),
                        },
                        pos: Some(pos.clone()),
                        kind: SymbolKind::Parameter,
                    },
                );
                match function_kind {
                    SymbolKind::GenericFunction => {
                        let mut type_map = self.check_and_map_types(
                            &parameters,
                            &argument_types,
                            &mut HashMap::default(),
                            pos.clone(),
                        )?;
                        for (l, r) in parameters.iter().zip(argument_types.iter()) {
                            if let (
                                TType::Custom { name: lc, .. },
                                TType::Custom { name: rc, .. },
                            ) = (l, r)
                            {
                                if let Some(ic) = self.environment.generic_type_map.get(rc) {
                                    if lc == ic {
                                        if let Some(list) = self.environment.get_type(&lc) {
                                            let mut s = self.clone();
                                            if let Some(outerlist) = s.environment.get_type(&rc) {
                                                type_map = self.check_and_map_types(
                                                    &[list],
                                                    &[outerlist],
                                                    &mut type_map,
                                                    pos.clone(),
                                                )?;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        return_type = Box::new(self.get_output(*return_type, &mut type_map)?);
                        return Ok(Expr::Literal {
                            ttype: *return_type.clone(),
                            value: Atom::Call {
                                name: function_id,
                                arguments,
                            },
                        });
                    }
                    SymbolKind::Constructor
                    | SymbolKind::Variable
                    | SymbolKind::Parameter
                    | SymbolKind::Function => {
                        let mut type_map = self.check_and_map_types(
                            &parameters,
                            &argument_types,
                            &mut HashMap::default(),
                            pos,
                        )?;
                        return_type = Box::new(self.get_output(*return_type, &mut type_map)?);
                        if let Some(subtype) = self.environment.generic_type_map.get(&function_id) {
                            function_id = subtype.clone();
                        }
                        return_type = Box::new(self.get_output(*return_type, &mut type_map)?);
                        return Ok(Expr::Literal {
                            ttype: *return_type.clone(),
                            value: Atom::Call {
                                name: function_id,
                                arguments,
                            },
                        });
                    }
                }
            } else {
                return Err(self.generate_error_with_pos(
                    format!("E1 Not a valid call: {}", identifier),
                    format!(
                        "No function signature '{}' with {:?} as arguments",
                        identifier, argument_types
                    ),
                    pos,
                ));
            }
        }
    }

    fn varargs(
        &mut self,
        identifier: &String,
        argument_types: &mut Vec<TType>,
        arguments: &mut Vec<Expr>,
    ) {
        let mut type_flag: TType = TType::Any;
        let mut has_varargs = false;
        let mut element = 0;

        if let Some(_) = self
            .environment
            .get_function_type(identifier, &*argument_types)
        {
        } else {
            for i in 0..=argument_types.len() {
                // Split the list at the current index from the end
                let (left, right) = argument_types.split_at(argument_types.len() - i);
                // Check if all elements in 'right' are the same
                if let Some(first) = right.get(0) {
                    type_flag = first.clone();
                    let mut check = true;
                    for ttype in right.iter() {
                        if ttype != first {
                            check = false;
                            break;
                        }
                    }
                    // If all elements in 'right' are the same, create a TType::List
                    if check {
                        let mut new_right = left.to_vec();
                        new_right.push(TType::List {
                            inner: Box::new(first.clone()),
                        });
                        if let Some(_) = self
                            .environment
                            .get(&generate_unique_string(identifier, &new_right))
                        {
                            *argument_types = new_right;
                            element = i;
                            has_varargs = true;
                            break;
                        }
                    }
                }
            }
        }

        if has_varargs {
            arguments.reverse();
            let (leftexpr, rightexpr) = arguments.split_at(element);
            let mut leftexpr = leftexpr.to_vec();
            let mut rightexpr = rightexpr.to_vec();
            *arguments = vec![];
            rightexpr.reverse();
            arguments.append(&mut rightexpr);
            leftexpr.reverse();
            arguments.push(Expr::ListConstructor {
                ttype: type_flag.clone(),
                elements: leftexpr.clone(),
            });
        }
    }

    fn call(&mut self, identifier: String, pos: FilePosition) -> Result<Expr, NovaError> {
        let mut arguments: Vec<Expr>;
        // constructor
        if let Some(fields) = self.environment.custom_types.get(&identifier) {
            if self.current_token().is_symbol('{') {
                arguments = self.field_list(&identifier, fields.to_vec(), pos.clone())?;
            } else {
                arguments = self.argument_list()?;
            }
        } else {
            arguments = self.argument_list()?;
        }

        // normal function call <func(args)>
        // get list of types from arguments
        let mut argument_types: Vec<TType> = arguments.iter().map(|t| t.get_type()).collect();

        // if no arguments, push none
        if argument_types.is_empty() {
            argument_types.push(TType::None)
        }

        self.varargs(&identifier, &mut argument_types, &mut arguments);
        // check if function exists

        if let Some((
            TType::Function {
                parameters,
                mut return_type,
            },
            mut function_id,
            function_kind,
        )) = self
            .environment
            .get_function_type(&identifier, &argument_types)
        {
            match function_kind {
                SymbolKind::GenericFunction => {
                    let mut type_map = self.check_and_map_types(
                        &parameters,
                        &argument_types,
                        &mut HashMap::default(),
                        pos.clone(),
                    )?;
                    for (l, r) in parameters.iter().zip(argument_types.iter()) {
                        if let (TType::Custom { name: lc, .. }, TType::Custom { name: rc, .. }) =
                            (l, r)
                        {
                            if let Some(ic) = self.environment.generic_type_map.get(rc) {
                                if lc == ic {
                                    if let Some(list) = self.environment.get_type(&lc) {
                                        let mut s = self.clone();
                                        if let Some(outerlist) = s.environment.get_type(&rc) {
                                            type_map = self.check_and_map_types(
                                                &[list],
                                                &[outerlist],
                                                &mut type_map,
                                                pos.clone(),
                                            )?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    return_type = Box::new(self.get_output(*return_type, &mut type_map)?);
                    return Ok(Expr::Literal {
                        ttype: *return_type.clone(),
                        value: Atom::Call {
                            name: function_id,
                            arguments,
                        },
                    });
                }
                SymbolKind::Constructor
                | SymbolKind::Variable
                | SymbolKind::Parameter
                | SymbolKind::Function => {
                    let mut type_map = self.check_and_map_types(
                        &parameters,
                        &argument_types,
                        &mut HashMap::default(),
                        pos,
                    )?;
                    return_type = Box::new(self.get_output(*return_type, &mut type_map)?);
                    if let Some(subtype) = self.environment.generic_type_map.get(&function_id) {
                        function_id = subtype.clone();
                    }
                    return_type = Box::new(self.get_output(*return_type, &mut type_map)?);
                    return Ok(Expr::Literal {
                        ttype: *return_type.clone(),
                        value: Atom::Call {
                            name: function_id,
                            arguments,
                        },
                    });
                }
            }
        } else {
            if let Some((
                TType::Function {
                    parameters,
                    mut return_type,
                },
                mut function_id,
                function_kind,
            )) = self.environment.get_type_capture(&identifier)
            {
                let pos = self.get_current_token_position();
                self.environment.captured.last_mut().unwrap().insert(
                    identifier.clone(),
                    Symbol {
                        id: identifier.clone(),
                        ttype: TType::Function {
                            parameters: parameters.clone(),
                            return_type: return_type.clone(),
                        },
                        pos: Some(pos.clone()),
                        kind: SymbolKind::Parameter,
                    },
                );
                match function_kind {
                    SymbolKind::GenericFunction => {
                        let mut type_map = self.check_and_map_types(
                            &parameters,
                            &argument_types,
                            &mut HashMap::default(),
                            pos.clone(),
                        )?;
                        for (l, r) in parameters.iter().zip(argument_types.iter()) {
                            if let (
                                TType::Custom { name: lc, .. },
                                TType::Custom { name: rc, .. },
                            ) = (l, r)
                            {
                                if let Some(ic) = self.environment.generic_type_map.get(rc) {
                                    if lc == ic {
                                        if let Some(list) = self.environment.get_type(&lc) {
                                            let mut s = self.clone();
                                            if let Some(outerlist) = s.environment.get_type(&rc) {
                                                type_map = self.check_and_map_types(
                                                    &[list],
                                                    &[outerlist],
                                                    &mut type_map,
                                                    pos.clone(),
                                                )?;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        return_type = Box::new(self.get_output(*return_type, &mut type_map)?);
                        return Ok(Expr::Literal {
                            ttype: *return_type.clone(),
                            value: Atom::Call {
                                name: function_id,
                                arguments,
                            },
                        });
                    }
                    SymbolKind::Constructor
                    | SymbolKind::Variable
                    | SymbolKind::Parameter
                    | SymbolKind::Function => {
                        let mut type_map = self.check_and_map_types(
                            &parameters,
                            &argument_types,
                            &mut HashMap::default(),
                            pos,
                        )?;
                        return_type = Box::new(self.get_output(*return_type, &mut type_map)?);
                        if let Some(subtype) = self.environment.generic_type_map.get(&function_id) {
                            function_id = subtype.clone();
                        }
                        return_type = Box::new(self.get_output(*return_type, &mut type_map)?);
                        return Ok(Expr::Literal {
                            ttype: *return_type.clone(),
                            value: Atom::Call {
                                name: function_id,
                                arguments,
                            },
                        });
                    }
                }
            } else {
                return Err(self.generate_error_with_pos(
                    format!("E1 Not a valid call: {}", identifier),
                    format!(
                        "No function signature '{}' with {:?} as arguments",
                        identifier, argument_types
                    ),
                    pos,
                ));
            }
        }
    }

    fn field(
        &mut self,
        identifier: String,
        mut lhs: Expr,
        pos: FilePosition,
    ) -> Result<Expr, NovaError> {
        if let Some(name) = lhs.get_type().custom_to_string() {
            if let Some(fields) = self.environment.custom_types.get(&name) {
                let mut found = false;
                for (index, (field_name, ttype)) in fields.iter().enumerate() {
                    if &identifier == field_name {
                        lhs = Expr::Field {
                            ttype: ttype.clone(),
                            name: name.clone(),
                            index,
                            expr: Box::new(lhs),
                            position: pos.clone(),
                        };
                        found = true;
                        break;
                    }
                }
                if !found {
                    let mut lex = Lexicon::new();
                    for (i, _) in fields.iter() {
                        lex.insert(i)
                    }
                    let corrections = lex.corrections_for(&identifier);
                    return Err(self.generate_error(
                        format!("No field '{}' found for {}", identifier, name),
                        format!("cannot retrieve field\nDid you mean? {:?}", corrections),
                    ));
                }
            } else {
                return Err(self.generate_error(
                    format!("No field '{}' found for {}", identifier, name),
                    format!("cannot retrieve field"),
                ));
            }
        } else {
            return Err(self.generate_error(
                format!("{:?} has no '{}' field", lhs.get_type(), identifier),
                format!("cannot retrieve field"),
            ));
        }
        Ok(lhs)
    }

    fn chain(&mut self, mut lhs: Expr) -> Result<Expr, NovaError> {
        let (identifier, pos) = self.get_identifier()?;
        match self.current_token() {
            Token::Operator {
                operator: Operator::DoubleColon,
                ..
            } => {
                let mut rhs = lhs.clone();
                while self.current_token().is_op(Operator::DoubleColon) {
                    self.consume_operator(Operator::DoubleColon)?;
                    let (field, pos) = self.get_identifier()?;
                    if let Some(ctype) = self.environment.get_type(&identifier) {
                        rhs = self.field(
                            field.clone(),
                            Expr::Literal {
                                ttype: ctype,
                                value: Atom::Id {
                                    name: identifier.clone(),
                                },
                            },
                            pos,
                        )?;
                    } else {
                        let mut lex = Lexicon::new();
                        self.environment.values.last().iter().for_each(|table| {
                            table.iter().for_each(|value: (&String, &Symbol)| {
                                if let Symbol {
                                    kind: SymbolKind::Variable,
                                    id: _,
                                    ttype: _,
                                    pos: _,
                                } = value.1
                                {
                                    lex.insert(value.0)
                                }
                            })
                        });

                        let corrections = lex.corrections_for(&identifier);
                        return Err(self.generate_error_with_pos(
                            format!("'{}' does not exist ", identifier),
                            format!("cannot retrieve field\nDid you mean? {:?}", corrections),
                            pos,
                        ));
                    }
                }
                // function pointer return call <func()(args)>
                let mut arguments = vec![lhs.clone()];
                arguments.extend(self.argument_list()?);
                if let TType::Function {
                    parameters,
                    mut return_type,
                } = rhs.get_type()
                {
                    if arguments.len() != parameters.len() {
                        return Err(self.generate_error_with_pos(
                            format!("E1 Inccorrect number of arguments"),
                            format!("Got {:?}, expected {:?}", arguments.len(), parameters.len()),
                            pos,
                        ));
                    }
                    let mut inputtypes = vec![];
                    for t in arguments.iter() {
                        inputtypes.push(t.get_type())
                    }
                    let mut map: HashMap<String, TType> = HashMap::default();
                    map = self.check_and_map_types(&parameters, &inputtypes, &mut map, pos)?;
                    return_type = Box::new(self.get_output(*return_type.clone(), &mut map)?);
                    lhs = Expr::Call {
                        ttype: *return_type,
                        name: "anon".to_string(),
                        function: Box::new(lhs),
                        args: arguments,
                    };
                } else {
                    return Err(self.generate_error_with_pos(
                        format!("Cant call {:?}", lhs.get_type()),
                        format!("not a function"),
                        pos,
                    ));
                }
            }
            Token::Symbol { symbol: '(', .. } => {
                lhs = self.method(identifier.clone(), lhs, pos)?;
            }
            Token::Symbol { symbol: '[', .. } => {
                lhs = self.field(identifier.clone(), lhs, pos)?;
                lhs = self.index(identifier.clone(), lhs.clone(), lhs.get_type())?;
            }
            _ => {
                lhs = self.field(identifier.clone(), lhs, pos)?;
            }
        }

        Ok(lhs)
    }

    fn index(
        &mut self,
        identifier: String,
        mut lhs: Expr,
        ttype: TType,
    ) -> Result<Expr, NovaError> {
        match ttype {
            TType::List { inner } => {
                self.consume_symbol('[')?;
                let pos = self.get_current_token_position();
                let index = self.mid_expr()?;
                self.consume_symbol(']')?;
                if index.get_type() != TType::Int {
                    return Err(self.generate_error_with_pos(
                        format!("Must index list with an int"),
                        format!("Cannot index into list with {:?}", index.get_type()),
                        pos,
                    ));
                }
                lhs = Expr::Indexed {
                    ttype: *inner.clone(),
                    name: identifier.clone(),
                    index: Box::new(index),
                    container: Box::new(lhs),
                    position: self.get_current_token_position(),
                };
                if self.current_token().is_symbol('[') {
                    lhs = self.index(identifier.clone(), lhs, *inner)?;
                }
            }
            TType::Tuple { elements: inner } => {
                self.consume_symbol('[')?;
                let pos = self.get_current_token_position();
                if let Token::Integer { value: index, .. } = self.current_token() {
                    self.advance();
                    self.consume_symbol(']')?;
                    if index as usize >= inner.len() {
                        return Err(self.generate_error_with_pos(
                            format!("Tuple cannot index into {index}"),
                            format!("Tuple has {} values", inner.len()),
                            pos,
                        ));
                    }
                    let ttype = &inner[index as usize];
                    lhs = Expr::Indexed {
                        ttype: ttype.clone(),
                        name: "anon".to_string(),
                        index: Box::new(Expr::Literal {
                            ttype: TType::Int,
                            value: Atom::Integer { value: index },
                        }),
                        container: Box::new(lhs),
                        position: self.get_current_token_position(),
                    };
                    if self.current_token().is_symbol('[') {
                        lhs = self.index(identifier.clone(), lhs, ttype.clone())?;
                    }
                } else {
                    return Err(self.generate_error_with_pos(
                        format!("Must index tuple with an int"),
                        format!("Cannot index into tuple with {:?}", self.current_token()),
                        pos,
                    ));
                }
            }
            _ => {
                return Err(self.generate_error(
                    format!("Cannot index into non list"),
                    format!("Must be of type list"),
                ));
            }
        }

        Ok(lhs)
    }

    fn anchor(&mut self, identifier: String, pos: FilePosition) -> Result<Expr, NovaError> {
        let anchor = match self.current_token() {
            Token::Operator {
                operator: Operator::RightArrow,
                ..
            } => {
                self.consume_operator(Operator::RightArrow)?;
                let (field, pos) = self.get_identifier()?;
                if let Some(idtype) = self.environment.get_type(&identifier) {
                    let mut arguments = vec![Expr::Literal {
                        ttype: idtype.clone(),
                        value: Atom::Id {
                            name: identifier.clone(),
                        },
                    }];

                    let left = self.field(
                        field.clone(),
                        Expr::Literal {
                            ttype: idtype,
                            value: Atom::Id {
                                name: identifier.clone(),
                            },
                        },
                        pos.clone(),
                    )?;
                    arguments.extend(self.argument_list()?);

                    if let TType::Function {
                        parameters,
                        mut return_type,
                    } = left.get_type()
                    {
                        if arguments.len() != parameters.len() {
                            return Err(self.generate_error_with_pos(
                                format!("E3 Inccorrect number of arguments"),
                                format!(
                                    "Got {:?}, expected {:?}",
                                    arguments.len(),
                                    parameters.len(),
                                ),
                                pos,
                            ));
                        }
                        let mut inputtypes = vec![];
                        for t in arguments.iter() {
                            inputtypes.push(t.get_type())
                        }
                        let mut map: HashMap<String, TType> = HashMap::default();
                        map = self.check_and_map_types(
                            &parameters,
                            &inputtypes,
                            &mut map,
                            pos.clone(),
                        )?;
                        return_type = Box::new(self.get_output(*return_type.clone(), &mut map)?);
                        Expr::Call {
                            ttype: *return_type,
                            name: field.to_string(),
                            function: Box::new(left),
                            args: arguments,
                        }
                    } else {
                        return Err(self.generate_error_with_pos(
                            format!("Cant call {:?}", left.get_type()),
                            format!("not a function"),
                            pos,
                        ));
                    }
                } else {
                    return Err(self.generate_error_with_pos(
                        format!("Cant get {field} from {}", identifier.clone()),
                        format!("{} is not defined", identifier),
                        pos,
                    ));
                }
            }
            Token::Symbol { symbol: '[', .. } => {
                if let Some(ttype) = self.environment.get_type(&identifier) {
                    self.index(
                        identifier.clone(),
                        Expr::Literal {
                            ttype: ttype.clone(),
                            value: Atom::Id {
                                name: identifier.clone(),
                            },
                        },
                        ttype.clone(),
                    )?
                } else {
                    if let Some((ttype, _, kind)) = self.environment.get_type_capture(&identifier) {
                        self.environment.captured.last_mut().unwrap().insert(
                            identifier.clone(),
                            Symbol {
                                id: identifier.clone(),
                                ttype: ttype.clone(),
                                pos: Some(pos.clone()),
                                kind: kind.clone(),
                            },
                        );
                        self.environment.insert_symbol(
                            &identifier,
                            ttype.clone(),
                            Some(pos.clone()),
                            kind,
                        );
                        self.index(
                            identifier.clone(),
                            Expr::Literal {
                                ttype: ttype.clone(),
                                value: Atom::Id {
                                    name: identifier.clone(),
                                },
                            },
                            ttype.clone(),
                        )?
                    } else {
                        let mut lex = Lexicon::new();
                        for (i, _) in self.environment.values.last().unwrap().iter() {
                            lex.insert(i)
                        }

                        let corrections = lex.corrections_for(&identifier);
                        return Err(self.generate_error_with_pos(
                            format!("E1 Not a valid symbol: {}", identifier),
                            format!("Unknown identifier\nDid you mean? {:?}", corrections),
                            pos,
                        ));
                    }
                }
            }
            Token::Symbol { symbol: '(', .. } => self.call(identifier.clone(), pos)?,
            _ => {
                if self.current_token().is_symbol('{')
                    && self.environment.custom_types.contains_key(&identifier)
                {
                    self.call(identifier.clone(), pos.clone())?
                } else {
                    if let Some(ttype) = self.environment.get_type(&identifier) {
                        Expr::Literal {
                            ttype: ttype.clone(),
                            value: Atom::Id {
                                name: identifier.clone(),
                            },
                        }
                    } else {
                        if let Some((ttype, _, kind)) =
                            self.environment.get_type_capture(&identifier)
                        {
                            self.environment.captured.last_mut().unwrap().insert(
                                identifier.clone(),
                                Symbol {
                                    id: identifier.clone(),
                                    ttype: ttype.clone(),
                                    pos: Some(pos.clone()),
                                    kind: kind.clone(),
                                },
                            );
                            self.environment.insert_symbol(
                                &identifier,
                                ttype.clone(),
                                Some(pos.clone()),
                                kind,
                            );
                            Expr::Literal {
                                ttype: ttype.clone(),
                                value: Atom::Id {
                                    name: identifier.clone(),
                                },
                            }
                        } else {
                            let mut lex = Lexicon::new();
                            for (i, _) in self.environment.values.last().unwrap().iter() {
                                lex.insert(i)
                            }
                            dbg!(self.environment.values.last().unwrap());
                            let corrections = lex.corrections_for(&identifier);
                            return Err(self.generate_error_with_pos(
                                format!("E2 Not a valid symbol: {}", identifier),
                                format!("Unknown identifier\nDid you mean? {:?}", corrections),
                                pos,
                            ));
                        }
                    }
                }
            }
        };

        Ok(anchor)
    }

    fn factor(&mut self) -> Result<Expr, NovaError> {
        let sign = if let Ok(Some(sign)) = self.sign() {
            self.advance();
            Some(sign)
        } else {
            None
        };
        let mut left: Expr;
        match self.current_token() {
            Token::Symbol { symbol: '#', .. } => {
                self.consume_symbol('#')?;
                let mut typelist = vec![];

                let expressions = self.tuple_list()?;
                for ttype in expressions.iter() {
                    typelist.push(ttype.get_type());
                }
                left = Expr::ListConstructor {
                    ttype: TType::Tuple { elements: typelist },
                    elements: expressions,
                };
            }
            Token::Symbol { symbol: '?', .. } => {
                self.consume_symbol('?')?;
                let option_type = self.ttype()?;
                left = Expr::Literal {
                    ttype: TType::Option {
                        inner: Box::new(option_type),
                    },
                    value: Atom::None,
                };
            }
            Token::Char { value: char, .. } => {
                self.advance();
                left = Expr::Literal {
                    ttype: TType::Char,
                    value: Atom::Char { value: char },
                }
            }
            Token::Identifier { name: id, .. } if id.as_str() == "fn" => {
                let pos = self.get_current_token_position();
                self.advance();
                // get parameters
                self.consume_symbol('(')?;
                let parameters = self.parameter_list()?;
                self.consume_symbol(')')?;
                // get output type
                let mut output = TType::Void;
                if self.current_token().is_symbol('{') {
                } else {
                    self.consume_operator(Operator::RightArrow)?;
                    output = self.ttype()?;
                }
                // retrieve types for input
                let mut typeinput = vec![];
                for arg in parameters.iter() {
                    typeinput.push(arg.0.clone())
                }

                // build helper vecs
                let mut input = vec![];
                for (ttype, identifier) in parameters.clone() {
                    if let TType::Function { .. } = ttype.clone() {
                        // check if generic function exist
                        if self.environment.has(&identifier) {
                            return Err(self.generate_error_with_pos(
                                format!("Generic Function {} already defined", &identifier),
                                "Cannot redefine a generic function".to_string(),
                                pos.clone(),
                            ));
                        }
                        // check if normal function exist
                        if self.environment.has(&identifier) {
                            return Err(self.generate_error_with_pos(
                                format!("Function {} already defined", &identifier,),
                                "Cannot redefine a generic function".to_string(),
                                pos.clone(),
                            ));
                        }
                        // build argument list
                        input.push(Arg {
                            identifier,
                            ttype: ttype.clone(),
                        });
                    } else {
                        input.push(Arg {
                            identifier,
                            ttype: ttype.clone(),
                        });
                    }
                }
                // check if no params, place none if empty
                if typeinput.is_empty() {
                    typeinput.push(TType::None)
                }

                self.environment.push_scope();

                // insert params into scope
                for (ttype, id) in parameters.iter() {
                    match ttype.clone() {
                        TType::Function {
                            parameters: paraminput,
                            return_type: output,
                        } => {
                            self.environment.insert_symbol(
                                &id,
                                TType::Function {
                                    parameters: paraminput.clone(),
                                    return_type: Box::new(*output.clone()),
                                },
                                Some(pos.clone()),
                                SymbolKind::Parameter,
                            );
                        }
                        _ => self.environment.insert_symbol(
                            &id,
                            ttype.clone(),
                            Some(pos.clone()),
                            SymbolKind::Parameter,
                        ),
                    };
                }

                let mut statements = self.block()?;

                let mut captured: Vec<String> = self
                    .environment
                    .captured
                    .last()
                    .unwrap()
                    .iter()
                    .map(|v| v.0.clone())
                    .collect();

                self.environment.pop_scope();

                for c in captured.iter() {
                    if let Some(mc) = self.environment.get_type_capture(&c.clone()) {
                        let pos = self.get_current_token_position();

                        self.environment.captured.last_mut().unwrap().insert(
                            c.clone(),
                            Symbol {
                                id: mc.1,
                                ttype: mc.0,
                                pos: Some(pos),
                                kind: mc.2,
                            },
                        );
                    }
                }

                captured = self
                    .environment
                    .captured
                    .last()
                    .unwrap()
                    .iter()
                    .map(|v| v.0.clone())
                    .collect();

                for dc in captured.iter() {
                    if let Some(_v) = self.environment.values.last().unwrap().get(dc) {
                        self.environment.captured.last_mut().unwrap().remove(dc);
                    }
                }
                // check return types

                let (_, has_return) =
                    self.check_returns(&statements, output.clone(), pos.clone())?;
                if !has_return && output != TType::Void {
                    return Err(self.generate_error(
                        "Function is missing a return statement in a branch".to_string(),
                        "Function missing return".to_string(),
                    ));
                }

                //check to see if all generic types in output are present in input
                // let mut inputtable = table::new();
                // for i in typeinput.iter() {
                //     inputtable.extend(self.getgen(i.clone()))
                // }

                // let outputtable = self.getgen(output.clone());
                // dbg!(&inputtable, &outputtable);

                // for o in outputtable.items.iter() {
                //     if inputtable.has(o) {
                //     } else {
                //         return Err(error::parser_error(
                //             format!("Input is missing type {}", o),
                //             format!("All generic types in output must be present in input"),
                //             pos.line,
                //             pos.row,
                //             self.filepath.clone(),
                //             None,
                //         ));
                //     }
                // }

                if output == TType::Void {
                    match statements.last() {
                        Some(Statement::Return { .. }) => {}
                        _ => {
                            statements.push(Statement::Return {
                                ttype: output.clone(),
                                expr: Expr::None,
                            });
                        }
                    }
                }
                left = Expr::Closure {
                    ttype: TType::Function {
                        parameters: typeinput,
                        return_type: Box::new(output),
                    },
                    args: input,
                    body: statements,
                    captures: captured,
                };
            }
            Token::Symbol { symbol: '|', .. } => {
                let pos = self.get_current_token_position();
                // get parameters
                self.consume_symbol('|')?;
                let parameters = self.parameter_list()?;
                self.consume_symbol('|')?;

                // retrieve types for input
                let mut typeinput = vec![];
                for arg in parameters.iter() {
                    typeinput.push(arg.0.clone())
                }

                // build helper vecs
                let mut input = vec![];
                for (ttype, identifier) in parameters.clone() {
                    if let TType::Function { .. } = ttype.clone() {
                        // check if generic function exist
                        if self.environment.has(&identifier) {
                            return Err(self.generate_error_with_pos(
                                format!("Generic Function {} already defined", &identifier),
                                "Cannot redefine a generic function".to_string(),
                                pos.clone(),
                            ));
                        }
                        // check if normal function exist
                        if self.environment.has(&identifier) {
                            return Err(self.generate_error_with_pos(
                                format!("Function {} already defined", &identifier,),
                                "Cannot redefine a generic function".to_string(),
                                pos.clone(),
                            ));
                        }
                        // build argument list
                        input.push(Arg {
                            identifier,
                            ttype: ttype.clone(),
                        });
                    } else {
                        input.push(Arg {
                            identifier,
                            ttype: ttype.clone(),
                        });
                    }
                }
                // check if no params, place none if empty
                if typeinput.is_empty() {
                    typeinput.push(TType::None)
                }

                self.environment.push_scope();

                // insert params into scope
                for (ttype, id) in parameters.iter() {
                    match ttype.clone() {
                        TType::Function {
                            parameters: paraminput,
                            return_type: output,
                        } => {
                            self.environment.insert_symbol(
                                &id,
                                TType::Function {
                                    parameters: paraminput.clone(),
                                    return_type: Box::new(*output.clone()),
                                },
                                Some(pos.clone()),
                                SymbolKind::Parameter,
                            );
                        }
                        _ => self.environment.insert_symbol(
                            &id,
                            ttype.clone(),
                            Some(pos.clone()),
                            SymbolKind::Parameter,
                        ),
                    };
                }
                let mut output = TType::Void;
                let statement = if let Token::Symbol { symbol: '{', .. } = self.current_token() {
                    //println!("its a block");
                    let block = self.block_expr()?;
                    if let Some(Statement::Return { ttype, expr: _ }) = block.last() {
                        output = ttype.clone();
                    };
                    block
                } else {
                    //println!("its an expression");
                    let expression = self.expr()?;
                    output = expression.clone().get_type();
                    let (line, row) = self.get_line_and_row();
                    let statement = vec![Statement::Return {
                        ttype: expression.get_type(),
                        expr: expression.clone(),
                    }];
                    statement
                };

                let mut captured: Vec<String> = self
                    .environment
                    .captured
                    .last()
                    .unwrap()
                    .iter()
                    .map(|v| v.0.clone())
                    .collect();

                self.environment.pop_scope();

                for c in captured.iter() {
                    if let Some(mc) = self.environment.get_type_capture(&c.clone()) {
                        let pos = self.get_current_token_position();

                        self.environment.captured.last_mut().unwrap().insert(
                            c.clone(),
                            Symbol {
                                id: mc.1,
                                ttype: mc.0,
                                pos: Some(pos),
                                kind: mc.2,
                            },
                        );
                    }
                }

                captured = self
                    .environment
                    .captured
                    .last()
                    .unwrap()
                    .iter()
                    .map(|v| v.0.clone())
                    .collect();

                for dc in captured.iter() {
                    if let Some(_v) = self.environment.values.last().unwrap().get(dc) {
                        self.environment.captured.last_mut().unwrap().remove(dc);
                    }
                }

                //check to see if all generic types in output are present in input
                // let mut inputtable = table::new();
                // for i in typeinput.iter() {
                //     inputtable.extend(self.getgen(i.clone()))
                // }

                // let outputtable = self.getgen(output.clone());
                // dbg!(&inputtable, &outputtable);

                // for o in outputtable.items.iter() {
                //     if inputtable.has(o) {
                //     } else {
                //         return Err(error::parser_error(
                //             format!("Input is missing type {}", o),
                //             format!("All generic types in output must be present in input"),
                //             pos.line,
                //             pos.row,
                //             self.filepath.clone(),
                //             None,
                //         ));
                //     }
                // }

                left = Expr::Closure {
                    ttype: TType::Function {
                        parameters: typeinput,
                        return_type: Box::new(output),
                    },
                    args: input,
                    body: statement,
                    captures: captured,
                };
            }
            Token::Symbol { symbol: '[', .. } => {
                let pos = self.get_current_token_position();
                let expr_list = self.expr_list()?;
                let mut ttype = TType::None;
                if !expr_list.is_empty() {
                    ttype = expr_list[0].get_type()
                }
                for elem in expr_list.clone() {
                    if elem.get_type() != ttype {
                        return Err(NovaError::TypeError {
                            msg: format!("List must contain same type"),
                            expected: ttype.to_string(),
                            found: elem.get_type().to_string(),
                            position: pos,
                        });
                    }
                }
                match self.current_token() {
                    Token::Operator {
                        operator: Operator::Colon,
                        ..
                    } => {
                        self.consume_operator(Operator::Colon)?;
                        ttype = self.ttype()?;
                        if !expr_list.is_empty() {
                            if ttype != expr_list[0].get_type() {
                                return Err(NovaError::TypeError {
                                    msg: format!("List must contain same type"),
                                    expected: ttype.to_string(),
                                    found: expr_list[0].get_type().to_string(),
                                    position: pos,
                                });
                            }
                        }
                    }
                    _ => {}
                }
                if ttype == TType::None {
                    return Err(self.generate_error_with_pos(
                        format!("List must have a type"),
                        format!("use `[]: type` to annotate an empty list"),
                        pos,
                    ));
                }
                left = Expr::ListConstructor {
                    ttype: TType::List {
                        inner: Box::new(ttype),
                    },
                    elements: expr_list,
                };
            }
            Token::Symbol { symbol: '(', .. } => {
                self.consume_symbol('(')?;
                let expr = self.expr()?;
                self.consume_symbol(')')?;
                left = expr;
                if let Some(sign) = sign {
                    if Unary::Not == sign {
                        if left.get_type() != TType::Bool {
                            return Err(self.generate_error(
                                "cannot apply (Not) operation to a non bool".to_string(),
                                "Make sure expression returns a bool type".to_string(),
                            ));
                        }
                    }
                    left = Expr::Unary {
                        ttype: left.clone().get_type(),
                        op: sign,
                        expr: Box::new(left),
                    };
                }
            }
            Token::Identifier { .. } => {
                let (mut identifier, pos) = self.get_identifier()?;
                match self.current_token() {
                    Token::Symbol { symbol: '@', .. } => {
                        self.consume_symbol('@')?;
                        self.consume_symbol('(')?;
                        let mut type_annotation = vec![];
                        let ta = self.ttype()?;
                        type_annotation.push(ta);
                        while self.current_token().is_symbol(',') {
                            self.advance();
                            let ta = self.ttype()?;
                            type_annotation.push(ta);
                        }
                        self.consume_symbol(')')?;
                        identifier = generate_unique_string(&identifier, &type_annotation);
                    }
                    Token::Operator {
                        operator: Operator::LeftArrow,
                        ..
                    } => {
                        self.consume_operator(Operator::LeftArrow)?;
                        let expr = self.expr()?;

                        // check if identifier exists
                        if self.environment.has(&identifier) {
                            return Err(self.generate_error_with_pos(
                                format!("Variable '{}' has already been created", identifier),
                                "".to_string(),
                                pos.clone(),
                            ));
                        } else {
                            self.environment.insert_symbol(
                                &identifier,
                                expr.get_type(),
                                Some(pos.clone()),
                                SymbolKind::Variable,
                            );
                            return Ok(Expr::Binop {
                                ttype: TType::Void,
                                op: Operator::Assignment,
                                lhs: Box::new(Expr::Literal {
                                    ttype: expr.get_type(),
                                    value: Atom::Id {
                                        name: identifier.clone(),
                                    },
                                }),
                                rhs: Box::new(expr),
                            });
                        }
                        // cant assing a void
                        if expr.get_type() == TType::Void {
                            return Err(self.generate_error_with_pos(
                                format!("Variable '{}' cannot be assinged to void", identifier),
                                "Make sure the expression returns a value".to_string(),
                                pos.clone(),
                            ));
                        }

                        if self.environment.has(&identifier) {
                            return Err(self.generate_error_with_pos(
                                format!("Variable '{}' has already been created", identifier),
                                "".to_string(),
                                pos.clone(),
                            ));
                        } else {
                            self.environment.insert_symbol(
                                &identifier,
                                expr.get_type(),
                                Some(pos.clone()),
                                SymbolKind::Variable,
                            );
                            return Ok(Expr::Binop {
                                ttype: TType::Void,
                                op: Operator::Assignment,
                                lhs: Box::new(Expr::Literal {
                                    ttype: expr.get_type(),
                                    value: Atom::Id {
                                        name: identifier.clone(),
                                    },
                                }),
                                rhs: Box::new(expr),
                            });
                        }
                    }
                    _ => {}
                }

                let leftt = self.anchor(identifier, pos)?;
                left = leftt;
                if let Some(sign) = sign {
                    if Unary::Not == sign {
                        if left.get_type() != TType::Bool {
                            return Err(self.generate_error(
                                "cannot apply not operation to a non bool".to_string(),
                                "Make sure expression returns a bool type".to_string(),
                            ));
                        }
                    }
                    left = Expr::Unary {
                        ttype: left.clone().get_type(),
                        op: sign,
                        expr: Box::new(left),
                    };
                }
            }
            Token::Integer { value: v, .. } => {
                self.advance();
                left = Expr::Literal {
                    ttype: TType::Int,
                    value: Atom::Integer { value: v },
                };
                if let Some(sign) = sign {
                    if Unary::Not == sign {
                        if left.get_type() != TType::Bool {
                            return Err(self.generate_error(
                                "cannot apply (Not) operation to a non bool".to_string(),
                                "Make sure expression returns a bool type".to_string(),
                            ));
                        }
                    }
                    left = Expr::Unary {
                        ttype: left.clone().get_type(),
                        op: sign,
                        expr: Box::new(left),
                    };
                }
            }
            Token::Float { value: v, .. } => {
                self.advance();
                left = Expr::Literal {
                    ttype: TType::Float,
                    value: Atom::Float { value: v },
                };
                if let Some(sign) = sign {
                    if Unary::Not == sign {
                        if left.get_type() != TType::Bool {
                            return Err(self.generate_error(
                                "cannot apply (Not) operation to a non bool".to_string(),
                                "Make sure expression returns a bool type".to_string(),
                            ));
                        }
                    }
                    left = Expr::Unary {
                        ttype: left.clone().get_type(),
                        op: sign,
                        expr: Box::new(left),
                    };
                }
            }
            Token::String { value: v, .. } => {
                self.advance();
                left = Expr::Literal {
                    ttype: TType::String,
                    value: Atom::String { value: v },
                };
            }

            Token::Bool { value: v, .. } => {
                self.advance();
                left = Expr::Literal {
                    ttype: TType::Bool,
                    value: Atom::Bool { value: v },
                };
            }
            Token::EOF { .. } => {
                return Err(self
                    .generate_error(format!("End of file error"), format!("expected expression")));
            }
            _ => left = Expr::None,
        }
        loop {
            match self.current_token() {
                Token::Operator {
                    operator: Operator::RightArrow,
                    ..
                } => {
                    self.consume_operator(Operator::RightArrow)?;
                    let (target_field, pos) = self.get_identifier()?;
                    let mut arguments = vec![left.clone()];
                    left = self.field(target_field.clone(), left.clone(), pos.clone())?;
                    arguments.extend(self.argument_list()?);
                    if let TType::Function {
                        parameters,
                        mut return_type,
                    } = left.get_type()
                    {
                        if arguments.len() != parameters.len() {
                            return Err(self.generate_error_with_pos(
                                format!("Incorrect number of arguments"),
                                format!(
                                    "Got {:?}, expected {:?}",
                                    arguments.len(),
                                    parameters.len()
                                ),
                                pos.clone(),
                            ));
                        }
                        let mut input_types = vec![];
                        for arg in arguments.iter() {
                            input_types.push(arg.get_type())
                        }
                        let mut type_map: HashMap<String, TType> = HashMap::default();
                        type_map = self.check_and_map_types(
                            &parameters,
                            &input_types,
                            &mut type_map,
                            pos.clone(),
                        )?;
                        return_type =
                            Box::new(self.get_output(*return_type.clone(), &mut type_map)?);
                        left = Expr::Call {
                            ttype: *return_type,
                            name: target_field.to_string(),
                            function: Box::new(left),
                            args: arguments,
                        };
                    } else {
                        return Err(self.generate_error_with_pos(
                            format!("Cannot call {:?}", left.get_type()),
                            format!("Not a function"),
                            pos.clone(),
                        ));
                    }
                }
                Token::Operator {
                    operator: Operator::DoubleColon,
                    ..
                } => {
                    self.consume_operator(Operator::DoubleColon)?;
                    let (field, pos) = self.get_identifier()?;
                    left = self.field(field.clone(), left, pos)?;
                }
                Token::Symbol { symbol: '.', .. } => {
                    self.consume_symbol('.')?;
                    left = self.chain(left)?;
                }
                Token::Symbol { symbol: '(', .. } => {
                    // function pointer return call <func()(args)>
                    let pos = self.get_current_token_position();
                    let mut arguments = self.argument_list()?;
                    if arguments.is_empty() {
                        arguments.push(Expr::None)
                    }
                    if let TType::Function {
                        parameters,
                        mut return_type,
                    } = left.get_type()
                    {
                        if arguments.len() != parameters.len() {
                            return Err(self.generate_error_with_pos(
                                format!("Incorrect number of arguments"),
                                format!(
                                    "Got {:?}, expected {:?}",
                                    arguments.len(),
                                    parameters.len()
                                ),
                                pos.clone(),
                            ));
                        }
                        let mut input_types = vec![];
                        for arg in arguments.iter() {
                            input_types.push(arg.get_type())
                        }
                        let mut type_map: HashMap<String, TType> = HashMap::default();
                        type_map = self.check_and_map_types(
                            &parameters,
                            &input_types,
                            &mut type_map,
                            self.get_current_token_position(),
                        )?;
                        return_type =
                            Box::new(self.get_output(*return_type.clone(), &mut type_map)?);
                        left = Expr::Call {
                            ttype: *return_type,
                            name: "anon".to_string(),
                            function: Box::new(left),
                            args: arguments,
                        };
                    } else {
                        return Err(self.generate_error_with_pos(
                            format!("Cannot call {:?}", left.get_type()),
                            format!("Not a function"),
                            pos.clone(),
                        ));
                    }
                }
                Token::Symbol { symbol: '[', .. } => {
                    left = self.index("anon".to_string(), left.clone(), left.get_type().clone())?;
                }
                _ => {
                    break;
                }
            }
        }

        Ok(left)
    }

    fn term(&mut self) -> Result<Expr, NovaError> {
        let mut left = self.factor()?;
        let pos = self.get_current_token_position();
        while self.current_token().is_multi_op() {
            if let Some(operation) = self.current_token().get_operator() {
                self.advance();
                let right = self.factor()?;
                if left.clone().get_type() == right.clone().get_type()
                    && (left.clone().get_type() == TType::Int
                        || left.clone().get_type() == TType::Float)
                    && (right.clone().get_type() == TType::Int
                        || right.clone().get_type() == TType::Float)
                {
                    self.check_and_map_types(
                        &[left.clone().get_type()],
                        &[right.clone().get_type()],
                        &mut HashMap::default(),
                        pos.clone(),
                    )?;
                    left = Expr::Binop {
                        ttype: left.get_type(),
                        op: operation,
                        lhs: Box::new(left),
                        rhs: Box::new(right),
                    };
                } else {
                    return Err(NovaError::TypeError {
                        expected: left.clone().get_type().to_string(),
                        found: right.clone().get_type().to_string(),
                        position: pos.clone(),
                        msg: format!(
                            "Type error, cannot apply operation {:?} to {} and {}",
                            operation.clone(),
                            right.get_type().to_string(),
                            left.get_type().to_string()
                        ),
                    });
                }
            }
        }
        Ok(left)
    }

    fn expr(&mut self) -> Result<Expr, NovaError> {
        let mut left = self.top_expr()?;
        let pos = self.get_current_token_position();
        while self.current_token().is_assign() {
            if let Some(operation) = self.current_token().get_operator() {
                self.advance();
                let right = self.top_expr()?;
                match left.clone() {
                    Expr::ListConstructor { .. }
                    | Expr::Binop { .. }
                    | Expr::Call { .. }
                    | Expr::Unary { .. }
                    | Expr::Closure { .. }
                    | Expr::None => {
                        return Err(self.generate_error_with_pos(
                            format!("Error: left hand side of `=` must be assignable"),
                            format!("Cannot assign a value to a literal value"),
                            pos.clone(),
                        ));
                    }
                    Expr::Literal { value: v, .. } => match v {
                        Atom::Id { .. } => {
                            self.check_and_map_types(
                                &vec![left.get_type()],
                                &vec![right.get_type()],
                                &mut HashMap::default(),
                                pos.clone(),
                            )?;
                        }
                        _ => {
                            return Err(self.generate_error_with_pos(
                                format!(
                                    "cannot assign {} to {}",
                                    right.get_type().to_string(),
                                    left.get_type().to_string()
                                ),
                                format!("Cannot assign a value to a literal value"),
                                pos.clone(),
                            ));
                        }
                    },
                    _ => {
                        if &right.get_type() == &left.get_type() {
                        } else {
                            return Err(NovaError::TypeError {
                                expected: left.clone().get_type().to_string(),
                                found: right.clone().get_type().to_string(),
                                position: pos.clone(),
                                msg: format!(
                                    "Type error, cannot assign  {} to {}",
                                    right.get_type().to_string(),
                                    left.get_type().to_string()
                                ),
                            });
                        }
                    }
                }
                left = Expr::Binop {
                    ttype: TType::Void,
                    op: operation,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                };
            }
        }
        Ok(left)
    }

    fn top_expr(&mut self) -> Result<Expr, NovaError> {
        let mut left = self.mid_expr()?;
        let pos = self.get_current_token_position();
        while self.current_token().is_relop() {
            if let Some(operation) = self.current_token().get_operator() {
                self.advance();
                let right = self.mid_expr()?;
                match operation {
                    Operator::And | Operator::Or => {
                        if (left.get_type() != TType::Bool) || (right.get_type() != TType::Bool) {
                            return Err(self.generate_error_with_pos(
                                format!("Logical operation expects bool"),
                                format!(
                                    "got {:?} {:?}",
                                    left.get_type().clone(),
                                    right.get_type().clone()
                                ),
                                pos.clone(),
                            ));
                        }
                        left = Expr::Binop {
                            ttype: TType::Bool,
                            op: operation,
                            lhs: Box::new(left),
                            rhs: Box::new(right),
                        };
                    }
                    Operator::GreaterThan
                    | Operator::GtrOrEqu
                    | Operator::LssOrEqu
                    | Operator::LessThan => {
                        match (left.get_type(), right.get_type()) {
                            (TType::Int, TType::Int) => {}
                            (TType::Float, TType::Float) => {}
                            _ => {
                                return Err(self.generate_error_with_pos(
                                    format!("Comparison operation expects int or float"),
                                    format!(
                                        "got {:?} {:?}",
                                        left.get_type().clone(),
                                        right.get_type().clone()
                                    ),
                                    pos.clone(),
                                ));
                            }
                        }
                        left = Expr::Binop {
                            ttype: TType::Bool,
                            op: operation,
                            lhs: Box::new(left),
                            rhs: Box::new(right),
                        };
                    }
                    _ => {
                        left = Expr::Binop {
                            ttype: TType::Bool,
                            op: operation,
                            lhs: Box::new(left),
                            rhs: Box::new(right),
                        };
                    }
                }
            }
        }
        Ok(left)
    }

    fn mid_expr(&mut self) -> Result<Expr, NovaError> {
        let mut left = self.term()?;
        let pos = self.get_current_token_position();
        while self.current_token().is_adding_op() {
            if let Some(operation) = self.current_token().get_operator() {
                self.advance();
                let right = self.term()?;

                match (left.get_type(), right.get_type()) {
                    (TType::Int, TType::Int)
                    | (TType::Float, TType::Float)
                    | (TType::String, TType::String) => {
                        left = Expr::Binop {
                            ttype: left.clone().get_type(),
                            op: operation,
                            lhs: Box::new(left),
                            rhs: Box::new(right),
                        };
                    }
                    (_, _) => {
                        return Err(NovaError::TypeError {
                            expected: left.clone().get_type().to_string(),
                            found: right.clone().get_type().to_string(),
                            position: pos.clone(),
                            msg: format!(
                                "Type error, cannot apply operation {:?} to {} and {}",
                                operation.clone(),
                                right.get_type().to_string(),
                                left.get_type().to_string()
                            ),
                        });
                    }
                }
            }
        }
        Ok(left)
    }

    fn getgen(&self, ttype: TType) -> Table<String> {
        let mut gtable = table::new();
        match ttype {
            TType::List { inner } | TType::Option { inner } => {
                let innertable = self.getgen(*inner);
                gtable.extend(innertable);
            }
            TType::Function {
                parameters,
                return_type,
            } => {
                let mut input_table = table::new();
                let mut output_table = table::new();
                for i in parameters.iter() {
                    input_table.extend(self.getgen(i.clone()));
                }
                output_table.extend(self.getgen(*return_type));

                gtable.extend(input_table);
                gtable.extend(output_table);
            }
            TType::Generic { name: gen } => {
                gtable.insert(gen);
            }
            TType::Option { inner } => {
                let innertable = self.getgen(*inner);
                gtable.extend(innertable);
            }
            _ => {}
        }
        gtable
    }

    fn ttype(&mut self) -> Result<TType, NovaError> {
        match self.current_token() {
            Token::Symbol { symbol: '#', .. } => {
                self.consume_symbol('#')?;
                let mut typelist = vec![];
                self.consume_symbol('(')?;
                typelist.push(self.ttype()?);
                while self.current_token().is_symbol(',') {
                    self.consume_symbol(',')?;
                    typelist.push(self.ttype()?);
                }
                self.consume_symbol(')')?;
                Ok(TType::Tuple { elements: typelist })
            }
            Token::Symbol { symbol: '(', .. } => {
                self.consume_symbol('(')?;
                let mut input = vec![];
                if !self.current_token().is_symbol(')') {
                    let inner = self.ttype()?;
                    input.push(inner);
                    while self.current_token().is_symbol(',') {
                        self.consume_symbol(',')?;
                        let inner = self.ttype()?;
                        input.push(inner);
                    }
                    self.consume_symbol(')')?;
                    let mut output = TType::Void;
                    if self.current_token().is_op(Operator::RightArrow) {
                        self.consume_operator(Operator::RightArrow)?;
                        output = self.ttype()?;
                    }

                    // check to see if all generic types in output are present in input
                    // let mut inputtable = table::new();
                    // for i in input.iter() {
                    //     inputtable.extend(self.getgen(i.clone()))
                    // }

                    // let outputtable = self.getgen(output.clone());
                    // dbg!(&inputtable,&outputtable);

                    // for o in outputtable.items.iter() {
                    //     if inputtable.has(o) {

                    //     } else {
                    //         return Err(self.generate_error(
                    //             format!("Input is missing type {}", o),
                    //             format!("All generic types in output must be present in input"),
                    //         ));
                    //     }
                    // }

                    Ok(TType::Function {
                        parameters: *Box::new(input),
                        return_type: Box::new(output),
                    })
                } else {
                    self.consume_symbol(')')?;
                    let mut output = TType::Void;
                    if self.current_token().is_op(Operator::RightArrow) {
                        self.consume_operator(Operator::RightArrow)?;
                        output = self.ttype()?;
                    }

                    //check to see if all generic types in output are present in input
                    // let mut inputtable = table::new();
                    // for i in input.iter() {
                    //     inputtable.extend(self.getgen(i.clone()))
                    // }

                    // let outputtable = self.getgen(output.clone());
                    // dbg!(&inputtable,&outputtable);

                    // for o in outputtable.items.iter() {
                    //     if inputtable.has(o) {

                    //     } else {
                    //         return Err(self.generate_error(
                    //             format!("Input is missing type {}", o),
                    //             format!("All generic types in output must be present in input"),
                    //         ));
                    //     }
                    // }
                    //dbg!(&output);
                    Ok(TType::Function {
                        parameters: *Box::new(vec![TType::None]),
                        return_type: Box::new(output),
                    })
                }
            }
            Token::Symbol { symbol: '$', .. } => {
                self.consume_symbol('$')?;
                let (generictype, _) = self.get_identifier()?;
                Ok(TType::Generic { name: generictype })
            }
            Token::Symbol { symbol: '?', .. } => {
                self.consume_symbol('?')?;
                let ttype = self.ttype()?;
                if let TType::Option { .. } = ttype {
                    return Err(self.generate_error(
                        "Cannot have option directly inside an option".to_string(),
                        format!("Type Error: Try removing the extra `?`"),
                    ));
                }
                Ok(TType::Option {
                    inner: Box::new(ttype),
                })
            }
            Token::Symbol { symbol: '[', .. } => {
                self.consume_symbol('[')?;
                let mut inner = TType::None;
                if !self.current_token().is_symbol(']') {
                    inner = self.ttype()?;
                }
                self.consume_symbol(']')?;
                Ok(TType::List {
                    inner: Box::new(inner),
                })
            }
            Token::Type { ttype, .. } => {
                self.advance();
                Ok(ttype)
            }
            Token::Identifier { .. } => {
                let (identifier, pos) = self.get_identifier()?;

                // let mut type_annotation = vec![];
                // if let Token::Symbol { symbol: '@', .. } = self.current_token() {
                //     self.consume_symbol('@')?;
                //     self.consume_symbol('(')?;

                //     let first_type = self.ttype()?;

                //     type_annotation.push(first_type);
                //     while self.current_token().is_symbol(',') {
                //         self.advance();
                //         let ttype = self.ttype()?;
                //         type_annotation.push(ttype);
                //     }
                //     self.consume_symbol(')')?;
                //     identifier = generate_unique_string(&identifier, &type_annotation);
                // }

                // if let Some(ttype) = self.environment.type_alias.get(&identifier) {
                //     return Ok(ttype.clone());
                // }
                if let Some(_) = self.environment.custom_types.get(&identifier) {
                    // add generic support in type definition
                    Ok(TType::Custom { name: identifier })
                } else {
                    return Err(self.generate_error_with_pos(
                        "Expected type annotation".to_string(),
                        format!("Unknown type '{identifier}' "),
                        pos,
                    ));
                }
            }
            _ => {
                return Err(self.generate_error(
                    "Expected type annotation".to_string(),
                    format!("Unknown type value {}", self.current_token().to_string()),
                ));
            }
        }
    }

    fn get_identifier(&mut self) -> Result<(String, FilePosition), NovaError> {
        let identifier = match self.current_token().expect_id() {
            Some(id) => id,
            None => {
                return Err(self.generate_error(
                    "Expected identifier".to_string(),
                    format!(
                        "Cannot assign a value to {}",
                        self.current_token().to_string()
                    ),
                ));
            }
        };
        let (line, row) = self.get_line_and_row();
        self.advance();
        Ok((
            identifier,
            FilePosition {
                line,
                row,
                filepath: self.filepath.clone(),
            },
        ))
    }

    fn parameter_list(&mut self) -> Result<Vec<(TType, String)>, NovaError> {
        let mut parameters: Table<String> = table::new();
        let mut arguments = vec![];

        while self.current_token().is_identifier() {
            let (identifier, pos) = self.get_identifier()?;
            if parameters.has(&identifier) {
                return Err(self.generate_error_with_pos(
                    format!("parameter identifier already defined"),
                    format!("try using another name"),
                    pos,
                ));
            }
            parameters.insert(identifier.clone());
            self.consume_operator(Operator::Colon)?;
            let ttype = self.ttype()?;
            arguments.push((ttype, identifier));

            if !self.current_token().is_symbol(',') {
                break;
            }
            self.advance();
        }

        Ok(arguments)
    }

    fn alternative(&mut self) -> Result<Vec<Statement>, NovaError> {
        let test = self.top_expr()?;
        let pos = self.get_current_token_position();
        if test.get_type() != TType::Bool {
            return Err(self.generate_error_with_pos(
                format!("If statement expression must return a bool"),
                format!("got {:?}", test.get_type().clone()),
                pos,
            ));
        }
        let statements = self.block()?;
        let mut alternative: Option<Vec<Statement>> = None;
        if self.current_token().is_id("elif") {
            self.consume_identifier(Some("elif"))?;
            alternative = Some(self.alternative()?);
        } else if self.current_token().is_id("else") {
            self.consume_identifier(Some("else"))?;
            alternative = Some(self.block()?);
        }
        Ok(vec![Statement::If {
            ttype: TType::Void,
            test,
            body: statements,
            alternative,
        }])
    }

    fn import_file(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("import"))?;
        let ifilepath = match self.current_token() {
            Token::String {
                value: filepath, ..
            } => filepath,
            _ => {
                panic!()
            }
        };
        self.advance();
        let file = ifilepath.clone();

        let newfilepath: String = match extract_current_directory(&self.filepath) {
            Some(mut current_dir) => {
                current_dir.push_str(&file);
                current_dir
            }
            _ => file.clone(),
        };
        let tokenlist = Lexer::new(&newfilepath)?.tokenize()?;

        let mut iparser = self.clone();
        iparser.index = 0;
        iparser.filepath = newfilepath.clone();
        iparser.input = tokenlist;
        iparser.parse()?;
        self.environment = iparser.environment.clone();
        Ok(Some(Statement::Block {
            body: iparser.ast.program.clone(),
            filepath: newfilepath,
        }))
    }

    fn unwrap(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("unwrap"))?;
        let (identifier, pos) = self.get_identifier()?;
        // test if option type
        self.environment.push_block();
        if let Some(id_type) = self.environment.get_type(&identifier) {
            if let TType::Option { inner } = id_type.clone() {
                self.environment.insert_symbol(
                    &identifier,
                    *inner.clone(),
                    Some(pos),
                    SymbolKind::Variable,
                );
                let body = self.block()?;
                let alternative: Option<Vec<Statement>> = if self.current_token().is_id("else") {
                    self.consume_identifier(Some("else"))?;
                    Some(self.block()?)
                } else {
                    None
                };
                self.environment.pop_scope();
                return Ok(Some(common::nodes::Statement::Unwrap {
                    ttype: id_type,
                    identifier,
                    body,
                    alternative,
                }));
            } else {
                return Err(self.generate_error_with_pos(
                    format!("unwrap expects an option type"),
                    format!("got {:?}", id_type),
                    pos,
                ));
            }
        } else {
            return Err(self.generate_error_with_pos(
                format!("unknown identifier"),
                format!("got {:?}", identifier),
                pos,
            ));
        }
    }

    fn statement(&mut self) -> Result<Option<Statement>, NovaError> {
        match self.current_token() {
            Token::Identifier { name: id, .. } => match id.as_str() {
                "type" => self.typealias(),
                "bind" => self.bind(),
                "unwrap" => self.unwrap(),
                "import" => self.import_file(),
                "pass" => self.pass_statement(),
                "struct" => self.struct_declaration(),
                "if" => self.if_statement(),
                "while" => self.while_statement(),
                "let" => self.let_statement(),
                "return" => self.return_statement(),
                "fn" => self.function_declaration(),
                "for" => self.for_statement(),
                "foreach" => self.foreach_statement(),
                "break" => {
                    self.consume_identifier(Some("break"))?;
                    Ok(Some(Statement::Break))
                }
                "continue" => {
                    self.consume_identifier(Some("continue"))?;
                    Ok(Some(Statement::Continue))
                }
                _ => self.expression_statement(),
            },
            Token::EOF { .. } => Ok(None),
            _ => self.expression_statement(),
        }
    }

    fn pass_statement(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("pass"))?;
        Ok(Some(Statement::Pass))
    }

    // fn typealias(&mut self) -> Result<Option<Statement>, NovaError> {
    //     self.consume_identifier(Some("type"))?;
    //     // get type id
    //     let (id, pos) = self.identifier()?;
    //     if self.environment.custom_types.contains_key(&id) {
    //         return Err(self.generate_error(
    //             format!("Type '{}' is already instantiated", id),
    //             "Cannot alias a custom type".to_string(),
    //             pos.line,
    //             pos.row,
    //             self.filepath.clone(),
    //             None,
    //         ));
    //     } else {
    //         self.environment.custom_types.insert(id.clone(), vec![]);
    //     }
    //     // assingment
    //     self.consume_operator(Operator::Assignment)?;
    //     // get type
    //     let ttype = self.ttype()?;
    //     // insert into type alias

    //     let gmap = self.getgen(ttype.clone());
    //     if !gmap.is_empty() {
    //         return Err(self.generate_error(
    //             format!("Type alias cannot contain generic type"),
    //             format!("Try removing the generic type"),
    //         ));
    //     }

    //     self.environment.type_alias.insert(id, ttype);
    //     Ok(None)
    // }

    fn get_id_list(&mut self) -> Result<Vec<String>, NovaError> {
        let mut idlist = vec![];
        self.consume_symbol('(')?;
        if !self.current_token().is_symbol(')') {
            idlist.push(self.get_identifier()?.0);
        }
        while self.current_token().is_symbol(',') {
            self.advance();
            if self.current_token().is_symbol(')') {
                break;
            }
            idlist.push(self.get_identifier()?.0);
        }
        self.consume_symbol(')')?;
        Ok(idlist)
    }

    fn collect_generics(&self, input: &[TType]) -> Table<String> {
        let mut contracts = table::new();
        for t in input {
            match t {
                TType::Generic { name: generic } => contracts.insert(generic.clone()),
                TType::Function {
                    parameters: input,
                    return_type: output,
                } => {
                    contracts.extend(self.collect_generics(input));
                    contracts.extend(self.collect_generics(&[*output.clone()]))
                }
                TType::List { inner: list } => {
                    contracts.extend(self.collect_generics(&[*list.clone()]))
                }
                TType::Option { inner: option } => {
                    contracts.extend(self.collect_generics(&[*option.clone()]))
                }
                TType::Custom { name: _custom } => {
                    // find custom type and import any generic type variables it has.
                    // if let Some(dict) = self.environment.custom_types.get(custom) {
                    //     dbg!(custom,dict);
                    //     for t in dict.iter() {
                    //         contracts.extend(self.collect_type_contracts(&[t.1.clone()]));
                    //     }
                    // }
                }
                _ => {}
            }
        }
        contracts
    }

    fn struct_declaration(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("struct"))?;
        let (struct_name, pos) = self.get_identifier()?;
        // will overwrite, just needed for recursive types.
        self.environment
            .custom_types
            .insert(struct_name.clone(), vec![]);

        let mut field_names = vec![];
        if let Token::Symbol { symbol: '(', .. } = self.current_token() {
            field_names = self.get_id_list()?;
            self.environment
                .generic_type_struct
                .insert(struct_name.clone(), field_names.clone());
        }

        self.consume_symbol('{')?;
        let parameters = self.parameter_list()?;
        self.consume_symbol('}')?;

        let mut fields: Vec<(String, TType)> = vec![];
        let mut type_inputs = vec![];
        let mut generics: Table<String> = table::new();

        for (field_type, field_name) in parameters.clone() {
            generics.extend(self.collect_generics(&[field_type.clone()]));
            type_inputs.push(field_type.clone());
            fields.push((field_name, field_type));
        }
        fields.push(("type".to_string(), TType::String));
        for generic_type in generics.items.iter() {
            if field_names.contains(generic_type) {
            } else {
                return Err(self.generate_error_with_pos(
                    format!(
                        "Struct '{}' is missing generic type {generic_type}",
                        struct_name
                    ),
                    "You must include generics types in struct name(...generictypes) ".to_string(),
                    pos.clone(),
                ));
            }
        }
        let mut input = vec![];
        for (field_name, field_type) in fields.clone() {
            input.push(Field {
                identifier: field_name,
                ttype: field_type,
            })
        }

        if !self.environment.has(&struct_name) {
            self.environment.no_override.insert(struct_name.to_string());
            if generics.is_empty() {
                self.environment.insert_symbol(
                    &struct_name,
                    TType::Function {
                        parameters: type_inputs,
                        return_type: Box::new(TType::Custom {
                            name: struct_name.clone(),
                        }),
                    },
                    Some(pos.clone()),
                    SymbolKind::Constructor,
                );
            }
            self.environment
                .custom_types
                .insert(struct_name.clone(), fields);
        } else {
            return Err(self.generate_error_with_pos(
                format!("Struct '{}' is already instantiated", struct_name),
                "Cannot reinstantiate the same type".to_string(),
                pos.clone(),
            ));
        }

        Ok(Some(Statement::Struct {
            ttype: TType::Custom {
                name: struct_name.clone(),
            },
            identifier: struct_name,
            fields: input,
        }))
    }

    fn for_statement(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("for"))?;
        let init = self.expr()?;
        self.consume_symbol(';')?;
        let testpos = self.get_current_token_position();
        let test = self.expr()?;
        self.consume_symbol(';')?;
        let inc = self.expr()?;
        if test.get_type() != TType::Bool && test.get_type() != TType::Void {
            return Err(self.generate_error_with_pos(
                format!("test expression must return a bool"),
                format!("got {:?}", test.get_type().clone()),
                testpos,
            ));
        }
        self.environment.push_block();
        let body = self.block()?;
        self.environment.pop_scope();
        Ok(Some(Statement::For {
            init,
            test,
            inc,
            body,
        }))
    }

    fn foreach_statement(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("foreach"))?;
        let (identifier, pos) = self.get_identifier()?;
        if self.environment.has(&identifier) {
            return Err(self.generate_error_with_pos(
                format!("identifier already used"),
                format!("identifier '{identifier}' is already used within this scope"),
                pos.clone(),
            ));
        }
        self.consume_identifier(Some("in"))?;
        let arraypos = self.get_current_token_position();
        let array = self.expr()?;
        self.environment.push_block();
        // check if array has type array and then assign identifier to that type
        if let TType::List { inner } = array.get_type() {
            self.environment
                .insert_symbol(&identifier, *inner, Some(pos), SymbolKind::Variable)
        } else {
            return Err(self.generate_error_with_pos(
                format!("foreach can only iterate over arrays"),
                format!("got {:?}", array.get_type().clone()),
                arraypos.clone(),
            ));
        }
        let body = self.block()?;
        self.environment.pop_scope();
        Ok(Some(Statement::Foreach {
            identifier,
            expr: array,
            body,
        }))
    }

    fn while_statement(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("while"))?;
        let testpos = self.get_current_token_position();
        let test = self.top_expr()?;
        if test.get_type() != TType::Bool && test.get_type() != TType::Void {
            return Err(self.generate_error_with_pos(
                format!("test expression must return a bool"),
                format!("got {:?}", test.get_type().clone()),
                testpos,
            ));
        }
        self.environment.push_block();
        let statements = self.block()?;
        self.environment.pop_scope();

        Ok(Some(Statement::While {
            test,
            body: statements,
        }))
    }

    fn if_statement(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("if"))?;
        let testpos = self.get_current_token_position();
        let test = self.top_expr()?;
        if test.get_type() != TType::Bool {
            return Err(self.generate_error_with_pos(
                format!("If statement's expression must return a bool"),
                format!("got {:?}", test.get_type().clone()),
                testpos.clone(),
            ));
        }
        let body = self.block()?;
        let mut alternative: Option<Vec<Statement>> = None;
        if self.current_token().is_id("elif") {
            self.advance();
            alternative = Some(self.alternative()?);
        } else if self.current_token().is_id("else") {
            self.advance();
            alternative = Some(self.block()?);
        }
        Ok(Some(Statement::If {
            ttype: TType::Void,
            test,
            body,
            alternative,
        }))
    }

    fn let_statement(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("let"))?;
        let mut global = false;
        // refactor out into two parsing ways for ident. one with module and one without
        let (mut identifier, mut pos) = self.get_identifier()?;
        if identifier == "global" {
            (identifier, pos) = self.get_identifier()?;
            global = true
        }
        #[allow(unused_assignments)]
        let mut ttype = TType::None;
        #[allow(unused_assignments)]
        let mut expr = Expr::None;
        if self.current_token().is_op(Operator::Colon) {
            self.consume_operator(Operator::Colon)?;
            ttype = self.ttype()?;
            self.consume_operator(Operator::Assignment)?;
            expr = self.expr()?;
            self.check_and_map_types(
                &vec![ttype.clone()],
                &vec![expr.get_type()],
                &mut HashMap::default(),
                pos.clone(),
            )?;
        } else {
            self.consume_operator(Operator::Assignment)?;
            expr = self.expr()?;
            ttype = expr.get_type();
        }

        // cant assing a void
        if expr.get_type() == TType::Void {
            return Err(self.generate_error_with_pos(
                format!("Variable '{}' cannot be assinged to void", identifier),
                "Make sure the expression returns a value".to_string(),
                pos.clone(),
            ));
        }
        // make sure symbol doesnt already exist
        if self.environment.has(&identifier) {
            return Err(self.generate_error_with_pos(
                format!("Symbol '{}' is already instantiated", identifier),
                "Cannot reinstantiate the same symbol in the same scope".to_string(),
                pos.clone(),
            ));
        } else {
            self.environment.insert_symbol(
                &identifier,
                ttype.clone(),
                Some(pos.clone()),
                SymbolKind::Variable,
            );
            Ok(Some(Statement::Let {
                ttype,
                identifier,
                expr,
                global,
            }))
        }
    }

    fn bind(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("bind"))?;
        let mut global = false;
        // refactor out into two parsing ways for ident. one with module and one without
        let (mut identifier, mut pos) = self.get_identifier()?;
        if identifier == "global" {
            (identifier, pos) = self.get_identifier()?;
            global = true
        }
        self.consume_operator(Operator::Assignment)?;
        let expr = self.expr()?;
        let inner = if let TType::Option { inner } = expr.get_type() {
            inner
        } else {
            return Err(self.generate_error_with_pos(
                format!("unwrap expects an option type"),
                format!("got {:?}", expr.get_type()),
                pos.clone(),
            ));
        };

        // make sure symbol doesnt already exist
        if self.environment.has(&identifier) {
            return Err(self.generate_error_with_pos(
                format!("Symbol '{}' is already instantiated", identifier),
                "Cannot reinstantiate the same symbol in the same scope".to_string(),
                pos.clone(),
            ));
        } else {
            self.environment.push_block();
            self.environment.insert_symbol(
                &identifier,
                *inner.clone(),
                Some(pos),
                SymbolKind::Variable,
            );
            let body = self.block()?;
            let alternative: Option<Vec<Statement>> = if self.current_token().is_id("else") {
                self.consume_identifier(Some("else"))?;
                Some(self.block()?)
            } else {
                None
            };
            self.environment.pop_scope();
            Ok(Some(Statement::Bind {
                ttype: expr.get_type(),
                identifier,
                expr,
                body,
                alternative,
                global,
            }))
        }
    }

    fn typealias(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("type"))?;
        // get type id
        let (alias_id, alias_pos) = self.get_identifier()?;
        if self.environment.custom_types.contains_key(&alias_id) {
            return Err(self.generate_error_with_pos(
                format!("Type '{}' is already instantiated", alias_id),
                "Cannot alias a custom type".to_string(),
                alias_pos.clone(),
            ));
        } else {
            self.environment
                .custom_types
                .insert(alias_id.clone(), vec![]);
        }
        // assignment

        self.consume_operator(Operator::Assignment)?;
        // get generic type
        let (generic_type, generic_pos) = self.get_identifier()?;
        self.consume_symbol('(')?;
        let mut type_annotation = vec![];
        let type_arg = self.ttype()?;
        type_annotation.push(type_arg);
        while self.current_token().is_symbol(',') {
            self.advance();
            let type_arg = self.ttype()?;
            type_annotation.push(type_arg);
        }
        self.consume_symbol(')')?;
        #[allow(unused_assignments)]
        let mut generic_type_list = vec![];
        if let Some(list) = self.environment.generic_type_struct.get(&generic_type) {
            generic_type_list = list.clone();
            self.environment
                .generic_type_map
                .insert(alias_id.clone(), generic_type.clone());
        } else {
            return Err(self.generate_error_with_pos(
                "no generic".to_owned(),
                "".to_owned(),
                generic_pos,
            ));
        }

        // check if correct number of args
        //dbg!(&type_annotation,&generic_type_list);
        if type_annotation.len() != generic_type_list.len() {
            return Err(self.generate_error_with_pos(
                format!(
                    "not enough type arguments. Expecting {}, got {}",
                    generic_type_list.len(),
                    type_annotation.len()
                ),
                "".to_string(),
                generic_pos,
            ));
        }
        let mut generic_map: HashMap<String, TType> = HashMap::default();
        for (gen, t) in generic_type_list.iter().zip(type_annotation.iter()) {
            generic_map.insert(gen.clone(), t.clone());
        }
        // create new instance of type
        // with map

        if let Some(fields) = self.environment.custom_types.get(&generic_type) {
            let mut new_fields = vec![];
            let mut type_input = vec![];
            for (field, field_type) in fields.iter() {
                if field != "type" {
                    type_input.push(self.get_output(field_type.clone(), &mut generic_map.clone())?);
                }

                new_fields.push((
                    field.clone(),
                    self.get_output(field_type.clone(), &mut generic_map.clone())?,
                ))
            }

            if !self.environment.has(&alias_id) {
                self.environment.no_override.insert(alias_id.to_string());
                self.environment.insert_symbol(
                    &alias_id,
                    TType::Function {
                        parameters: type_input,
                        return_type: Box::new(TType::Custom {
                            name: alias_id.clone(),
                        }),
                    },
                    Some(alias_pos.clone()),
                    SymbolKind::Constructor,
                );
                self.environment
                    .custom_types
                    .insert(alias_id.clone(), new_fields);
            } else {
                return Err(self.generate_error_with_pos(
                    format!("Struct '{}' is already instantiated", alias_id),
                    "Cannot reinstantiate the same type".to_string(),
                    alias_pos.clone(),
                ));
            }
        } else {
            return Err(self.generate_error("broke".to_owned(), "no generic to be made".to_owned()));
        }

        Ok(None)
    }

    fn return_statement(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("return"))?;
        let expr = self.expr()?;
        Ok(Some(Statement::Return {
            ttype: expr.get_type(),
            expr,
        }))
    }

    fn is_generic(&self, params: &[TType]) -> bool {
        for t in params {
            match t {
                TType::Any => {
                    return true;
                }
                TType::Generic { .. } => {
                    return true;
                }
                TType::Function {
                    parameters: args,
                    return_type,
                } => {
                    if let TType::Generic { .. } = **return_type {
                        return true;
                    }
                    if self.is_generic(&args.clone())
                        || self.is_generic(&vec![*return_type.clone()])
                    {
                        return true;
                    }
                }
                TType::List { inner } => {
                    if let TType::Generic { .. } = **inner {
                        return true;
                    }
                    return self.is_generic(&vec![*inner.clone()]);
                }
                TType::Option { inner } => {
                    if let TType::Generic { .. } = **inner {
                        return true;
                    }
                    return self.is_generic(&vec![*inner.clone()]);
                }
                TType::Custom { name } => {
                    if self.environment.generic_type_struct.contains_key(name) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        return false;
    }

    fn function_declaration(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("fn"))?;
        let (mut identifier, pos) = self.get_identifier()?;

        // check to see if its already defined
        if self.environment.has(&identifier) {
            return Err(self.generate_error_with_pos(
                format!("Generic Function {identifier} already defined"),
                "Cannot overload a generic function".to_string(),
                pos.clone(),
            ));
        }
        // get parameters
        self.consume_symbol('(')?;
        let parameters = self.parameter_list()?;
        //dbg!(&parameters);
        self.consume_symbol(')')?;
        // get output type
        let mut output = TType::Void;
        if self.current_token().is_symbol('{') {
        } else {
            self.consume_operator(Operator::RightArrow)?;
            output = self.ttype()?;
        }
        // retrieve types for input
        let mut typeinput = vec![];
        for arg in parameters.iter() {
            typeinput.push(arg.0.clone())
        }
        // is function using generics?

        let generic = self.is_generic(&typeinput);
        // build helper vecs
        let mut input = vec![];
        for (ttype, identifier) in parameters.clone() {
            if let TType::Function { .. } = ttype.clone() {
                // check if generic function exist
                if self.environment.has(&identifier) {
                    return Err(self.generate_error_with_pos(
                        format!("Generic Function {} already defined", &identifier),
                        "Cannot redefine a generic function".to_string(),
                        pos.clone(),
                    ));
                }
                // check if normal function exist
                if self.environment.has(&identifier) {
                    return Err(self.generate_error_with_pos(
                        format!("Function {} already defined", &identifier,),
                        "Cannot redefine a generic function".to_string(),
                        pos.clone(),
                    ));
                }
                // build argument list
                input.push(Arg {
                    identifier,
                    ttype: ttype.clone(),
                });
            } else {
                input.push(Arg {
                    identifier,
                    ttype: ttype.clone(),
                });
            }
        }
        // check if no params, place none if empty
        if typeinput.is_empty() {
            typeinput.push(TType::None)
        }
        // check if normal function exist
        if self
            .environment
            .has(&generate_unique_string(&identifier, &typeinput))
        {
            return Err(self.generate_error_with_pos(
                format!(
                    "Function {identifier} with inputs {:?} is already defined",
                    typeinput
                ),
                "Cannot redefine a function with the same signature".to_string(),
                pos.clone(),
            ));
        }

        // insert function into environment
        if !generic {
            self.environment.insert_symbol(
                &identifier,
                TType::Function {
                    parameters: typeinput.clone(),
                    return_type: Box::new(output.clone()),
                },
                Some(pos.clone()),
                SymbolKind::Function,
            );
            identifier = generate_unique_string(&identifier, &typeinput);
        } else {
            if self.environment.no_override.has(&identifier) {
                return Err(self.generate_error_with_pos(
                    format!(
                        "Cannot create generic functon since, {} is already defined",
                        &identifier
                    ),
                    "Cannot create generic function since this function is overload-able"
                        .to_string(),
                    pos.clone(),
                ));
            }
            self.environment.insert_symbol(
                &identifier,
                TType::Function {
                    parameters: typeinput.clone(),
                    return_type: Box::new(output.clone()),
                },
                Some(pos.clone()),
                SymbolKind::GenericFunction,
            );
        }
        self.environment.no_override.insert(identifier.clone());
        // parse body with scope
        self.environment.push_scope();
        // insert params into scope
        for (ttype, id) in parameters.iter() {
            match ttype {
                TType::Function {
                    parameters,
                    return_type,
                } => {
                    self.environment.insert_symbol(
                        &id,
                        TType::Function {
                            parameters: parameters.clone(),
                            return_type: return_type.clone(),
                        },
                        Some(pos.clone()),
                        SymbolKind::Parameter,
                    );
                }
                _ => self.environment.insert_symbol(
                    &id,
                    ttype.clone(),
                    Some(pos.clone()),
                    SymbolKind::Parameter,
                ),
            }
        }
        let mut statements = self.block()?;
        self.environment.pop_scope();
        // check return types
        let (_, has_return) = self.check_returns(&statements, output.clone(), pos.clone())?;
        if !has_return && output != TType::Void {
            return Err(self.generate_error_with_pos(
                "Function is missing a return statement in a branch".to_string(),
                "Function missing return".to_string(),
                pos.clone(),
            ));
        }
        // if output void, insert return as last statement if one wasnt added
        if output == TType::Void {
            if let Some(Statement::Return { .. }) = statements.last() {
            } else {
                statements.push(Statement::Return {
                    ttype: output.clone(),
                    expr: Expr::None,
                });
            }
        }

        // if last statement isnt a return error
        if let Some(Statement::Return { .. }) = statements.last() {
        } else {
            return Err(self.generate_error_with_pos(
                "Function is missing a return statement in a branch".to_string(),
                "Function missing return".to_string(),
                pos.clone(),
            ));
        }

        Ok(Some(Statement::Function {
            ttype: output,
            identifier,
            parameters: input,
            body: statements,
        }))
    }

    fn check_returns(
        &self,
        statements: &[Statement],
        return_type: TType,
        pos: FilePosition,
    ) -> Result<(TType, bool), NovaError> {
        statements
            .iter()
            .try_fold(false, |has_return, statement| match statement {
                Statement::Pass => Ok(true),
                Statement::Return { ttype, .. } => {
                    self.check_and_map_types(
                        &vec![ttype.clone()],
                        &vec![return_type.clone()],
                        &mut HashMap::default(),
                        pos.clone(),
                    )?;
                    Ok(true)
                }
                Statement::If {
                    body, alternative, ..
                } => {
                    let (body_return_type, body_has_return) =
                        self.check_returns(body, return_type.clone(), pos.clone())?;
                    let alternative_result = alternative
                        .as_ref()
                        .map(|alt| self.check_returns(alt, return_type.clone(), pos.clone()))
                        .transpose()?;

                    match alternative_result {
                        Some((alternative_return_type, alternative_has_return))
                            if body_return_type == alternative_return_type =>
                        {
                            Ok(body_has_return && alternative_has_return)
                        }
                        Some(_) => Err(self.generate_error(
                            "Function is missing a return statement in a branch".to_string(),
                            "All branches of if-else must have a return statement".to_string(),
                        )),
                        None => Ok(body_has_return),
                    }
                }
                _ => Ok(has_return),
            })
            .map(|has_return| (return_type.clone(), has_return))
    }

    fn expression_statement(&mut self) -> Result<Option<Statement>, NovaError> {
        self.expr().map(|expr| {
            Some(Statement::Expression {
                ttype: expr.get_type(),
                expr,
            })
        })
    }

    fn block(&mut self) -> Result<Vec<Statement>, NovaError> {
        self.consume_symbol('{')?;
        let statements = self.compound_statement()?;
        self.consume_symbol('}')?;
        Ok(statements)
    }

    fn block_expr(&mut self) -> Result<Vec<Statement>, NovaError> {
        let pos = self.get_current_token_position();
        self.consume_symbol('{')?;
        let statements = self.compound_statement()?;
        self.consume_symbol('}')?;
        let error = self.generate_error_with_pos(
            "Block must have expression as last value".to_string(),
            "".to_string(),
            pos.clone(),
        );

        statements.split_last().map_or_else(
            || Err(error.clone()),
            |(last, initial_statements)| {
                if let Statement::Expression { ttype, expr, .. } = last {
                    let mut final_statements = initial_statements.to_vec();
                    final_statements.push(Statement::Return {
                        ttype: ttype.clone(),
                        expr: expr.clone(),
                    });
                    Ok(final_statements)
                } else {
                    Err(error.clone())
                }
            },
        )
    }

    fn compound_statement(&mut self) -> Result<Vec<Statement>, NovaError> {
        let mut initial_statements = vec![];
        if let Some(statement) = self.statement()? {
            initial_statements.push(statement)
        };

        let statements = {
            let mut statements = initial_statements;
            while self.current_token().is_symbol(';') || !self.is_current_eof() {
                if self.current_token().is_symbol(';') {
                    self.advance()
                }
                if self.current_token().is_symbol('}') {
                    break;
                }
                if let Some(statement) = self.statement()? {
                    statements.push(statement);
                }
            }
            statements
        };

        Ok(statements)
    }

    pub fn parse(&mut self) -> Result<(), NovaError> {
        self.ast.program = self.compound_statement()?;
        self.eof()
    }
}
