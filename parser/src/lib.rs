use std::collections::HashMap;

use common::{
    environment::{new_environment, Environment},
    error::NovaError,
    fileposition::FilePosition,
    nodes::{Arg, Ast, Atom, Expr, Field, Statement, Symbol, SymbolKind},
    table::{self, Table},
    tokens::{KeyWord, Operator, Token, TokenList, Unary},
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
    pub modules: table::Table<String>,
    pub repl: bool,
}

pub fn default() -> Parser {
    let env = create_environment();
    Parser {
        filepath: String::new(),
        ast: Ast { program: vec![] },
        input: vec![],
        index: 0,
        environment: env,
        modules: table::new(),
        repl: false,
    }
}

pub fn new(filepath: &str) -> Parser {
    let env = create_environment();
    Parser {
        filepath: filepath.to_string(),
        ast: Ast { program: vec![] },
        input: vec![],
        index: 0,
        environment: env,
        modules: table::new(),
        repl: false,
    }
}

fn create_environment() -> Environment {
    let mut env = new_environment();
    env.insert_symbol(
        "error",
        TType::Function {
            parameters: vec![TType::None],
            return_type: Box::new(TType::Void),
        },
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "todo",
        TType::Function {
            parameters: vec![TType::None],
            return_type: Box::new(TType::Generic {
                name: "T".to_string(),
            }),
        },
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "unreachable",
        TType::Function {
            parameters: vec![TType::None],
            return_type: Box::new(TType::Generic {
                name: "T".to_string(),
            }),
        },
        None,
        SymbolKind::GenericFunction,
    );
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
            parameters: vec![TType::Any],
            return_type: Box::new(TType::String),
        },
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "Option::isSome",
        TType::Function {
            parameters: vec![TType::Any],
            return_type: Box::new(TType::Bool),
        },
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "Option::unwrap",
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
        "None",
        TType::Function {
            parameters: vec![TType::None],
            return_type: Box::new(TType::Option {
                inner: Box::new(TType::Generic {
                    name: "T".to_string(),
                }),
            }),
        },
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "print",
        TType::Function {
            parameters: vec![TType::Any],
            return_type: Box::new(TType::Void),
        },
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "println",
        TType::Function {
            parameters: vec![TType::Any],
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
    env
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
                format!(
                    "Found {} arguments, but expecting {} arguments",
                    type_list2.len(),
                    type_list1.len()
                ),
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
                (
                    TType::Tuple {
                        elements: elements1,
                    },
                    TType::Tuple {
                        elements: elements2,
                    },
                ) => {
                    self.check_and_map_types(elements1, elements2, type_map, pos.clone())?;
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
                    if let Some(mapped_type) = type_map.clone().get(name1) {
                        if mapped_type != t2 {
                            return Err(NovaError::TypeMismatch {
                                expected: mapped_type.clone(),
                                found: t2.clone(),
                                position: pos.clone(),
                            });
                        }
                    } else {
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

                    self.check_and_map_types(params1, params2, type_map, pos.clone())?;
                    self.check_and_map_types(
                        &[*ret1.clone()],
                        &[*ret2.clone()],
                        type_map,
                        pos.clone(),
                    )?;
                }
                (
                    TType::Custom {
                        name: custom1,
                        type_params: gen1,
                    },
                    TType::Custom {
                        name: custom2,
                        type_params: gen2,
                    },
                ) => {
                    if custom1 == custom2 {
                        self.check_and_map_types(gen1, gen2, type_map, pos.clone())?;
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
        pos: FilePosition,
    ) -> Result<TType, NovaError> {
        match output.clone() {
            TType::Tuple { elements } => {
                let mut mapped_elements = Vec::new();
                for element in elements {
                    let mapped_element = self.get_output(element, type_map, pos.clone())?;
                    mapped_elements.push(mapped_element);
                }
                Ok(TType::Tuple {
                    elements: mapped_elements,
                })
            }
            TType::Generic { name } => {
                if let Some(mapped_type) = type_map.get(&name) {
                    Ok(mapped_type.clone())
                } else {
                    // return type error novaerror::typeError
                    if self.environment.live_generics.last().unwrap().has(&name) {
                        return Ok(TType::Generic { name });
                    } else {
                        //dbg!(self.environment.live_generics.last().unwrap());
                        return Err(NovaError::SimpleTypeError {
                            msg: format!("Generic type {} could not be inferred", name),
                            position: pos,
                        });
                    }
                }
            }
            TType::List { inner } => {
                let mapped_inner = self.get_output(*inner.clone(), type_map, pos)?;
                Ok(TType::List {
                    inner: Box::new(mapped_inner),
                })
            }
            TType::Option { inner } => {
                let mapped_inner = self.get_output(*inner.clone(), type_map, pos)?;
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
                    let mapped_arg = self.get_output(arg, type_map, pos.clone())?;
                    mapped_args.push(mapped_arg);
                }

                let mapped_return_type = self.get_output(*return_type.clone(), type_map, pos)?;

                Ok(TType::Function {
                    parameters: mapped_args,
                    return_type: Box::new(mapped_return_type),
                })
            }
            TType::Custom { name, type_params } => {
                let mut mapped_type_params = Vec::new();
                for param in type_params {
                    let mapped_param = self.get_output(param, type_map, pos.clone())?;
                    mapped_type_params.push(mapped_param);
                }

                Ok(TType::Custom {
                    name,
                    type_params: mapped_type_params,
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
            format!(
                "unexpected operator, got {}",
                self.current_token().to_string()
            ),
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
            format!(
                "unexpected symbol, got {}",
                self.current_token().to_string()
            ),
            format!("expecting {}", sym),
        ))
    }

    // consume a keyword
    fn consume_keyword(&mut self, kind: KeyWord) -> Result<(), NovaError> {
        if let Token::Keyword { keyword, .. } = self.current_token() {
            if kind == keyword {
                self.advance();
                return Ok(());
            }
        }
        Err(self.generate_error(
            format!(
                "unexpected keyword, got {}",
                self.current_token().to_string()
            ),
            format!("expecting {:?}", kind),
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
                    format!("unexpected identifier, got {}", current_token.to_string()),
                    match symbol {
                        Some(s) => format!("expecting {}", s),
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

    // peek with offset
    fn peek_offset(&self, offset: usize) -> Option<Token> {
        if self.index + offset < self.input.len() {
            Some(self.input[self.index + offset].clone())
        } else {
            None
        }
    }

    fn sign(&mut self) -> Result<Option<Unary>, NovaError> {
        match self.current_token() {
            Token::Operator { operator, .. } => match operator {
                Operator::Addition => Ok(Some(Unary::Positive)),
                Operator::Subtraction => Ok(Some(Unary::Negitive)),
                Operator::Not => Ok(Some(Unary::Not)),
                _ => {
                    return Err(self.generate_error(
                        format!(
                            "unexpected operation, got {}",
                            self.current_token().to_string()
                        ),
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
            self.process_expression(&mut exprs)?;
        }

        while self.current_token().is_symbol(',') {
            self.advance();
            if self.current_token().is_symbol(')') {
                break;
            }
            self.process_expression(&mut exprs)?;
        }

        self.consume_symbol(')')?;
        Ok(exprs)
    }

    fn expr_list(&mut self) -> Result<Vec<Expr>, NovaError> {
        let mut exprs = vec![];
        self.consume_symbol('[')?;

        if !self.current_token().is_symbol(']') {
            self.process_expression(&mut exprs)?;
        }

        while self.current_token().is_symbol(',') {
            self.advance();
            if self.current_token().is_symbol(']') {
                break;
            }
            self.process_expression(&mut exprs)?;
        }

        self.consume_symbol(']')?;
        Ok(exprs)
    }

    fn process_expression(&mut self, exprs: &mut Vec<Expr>) -> Result<(), NovaError> {
        let pos = self.get_current_token_position();
        let e = self.expr()?;
        if e.get_type() != TType::Void {
            exprs.push(e);
            Ok(())
        } else {
            Err(self.generate_error_with_pos(
                format!("cannot insert a void expression"),
                format!("expressions must not be void"),
                pos,
            ))
        }
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
        let mut field_exprs: HashMap<String, Expr> = HashMap::default();
        self.consume_symbol('{')?;
        self.parse_field(&mut field_exprs)?;
        while self.current_token().is_symbol(',') {
            self.advance();
            if self.current_token().is_symbol('}') {
                break;
            }
            self.parse_field(&mut field_exprs)?;
        }
        self.consume_symbol('}')?;
        self.validate_fields(constructor, &fields, conpos, &field_exprs)
    }

    fn parse_field(&mut self, field_exprs: &mut HashMap<String, Expr>) -> Result<(), NovaError> {
        let (id, _) = self.get_identifier()?;
        self.consume_operator(Operator::Colon)?;
        field_exprs.insert(id.clone(), self.expr()?);
        Ok(())
    }

    fn validate_fields(
        &mut self,
        constructor: &str,
        fields: &[(String, TType)],
        conpos: FilePosition,
        field_exprs: &HashMap<String, Expr>,
    ) -> Result<Vec<Expr>, NovaError> {
        let mut validated_exprs = vec![];
        for (field_name, field_type) in fields.iter() {
            if field_name == "type" {
                continue;
            }
            if let Some(expr) = field_exprs.get(field_name) {
                self.check_and_map_types(
                    &vec![field_type.clone()],
                    &vec![expr.get_type()],
                    &mut HashMap::default(),
                    conpos.clone(),
                )?;
                validated_exprs.push(expr.clone());
            } else {
                return Err(NovaError::Parsing {
                    msg: format!("{} is missing field {}", constructor, field_name),
                    note: String::new(),
                    position: conpos.clone(),
                    extra: None,
                });
            }
        }
        if field_exprs.len() != fields.len() - 1 {
            return Err(NovaError::Parsing {
                msg: format!(
                    "{} has {} fields, you have {}",
                    constructor,
                    fields.len() - 1,
                    field_exprs.len()
                ),
                note: String::new(),
                position: conpos.clone(),
                extra: None,
            });
        }
        if validated_exprs.len() != fields.len() - 1 {
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
        Ok(validated_exprs)
    }

    fn method(
        &mut self,
        mut identifier: String,
        first_argument: Expr,
        pos: FilePosition,
    ) -> Result<Expr, NovaError> {
        let mut arguments = vec![first_argument];
        arguments.extend(self.argument_list()?);
        let mut argument_types: Vec<TType> = arguments.iter().map(|t| t.get_type()).collect();

        match self.current_token() {
            Token::Operator {
                operator: Operator::Colon,
                ..
            } => {
                self.advance();
                // call get closure
                let (typeinput, input, output, statement, captured) = self.bar_closure()?;
                //dbg!(typeinput.clone(), input.clone(), output.clone(), statement.clone());
                let last_closure = Expr::Closure {
                    ttype: TType::Function {
                        parameters: typeinput,
                        return_type: Box::new(output),
                    },
                    args: input,
                    body: statement,
                    captures: captured,
                };
                argument_types.push(last_closure.get_type());
                arguments.push(last_closure);
            }
            _ => {}
        }

        if argument_types.is_empty() {
            argument_types.push(TType::None)
        }
        let old_identifier = identifier.clone();
        if self.environment.custom_types.contains_key(&identifier) {
        } else if let Some(TType::Custom { name, .. }) = argument_types.get(0) {
            if self.environment.custom_types.contains_key(name) {
                //dbg!(&name);
                identifier = format!("{}::{}", name, identifier);
            }
        } else if let Some(ttype) = argument_types.get(0) {
            match ttype {
                TType::List { .. } => {
                    identifier = format!("List::{}", identifier);
                }
                TType::Option { .. } => {
                    identifier = format!("Option::{}", identifier);
                }
                TType::Function { parameters, .. } => {
                    let repeated_elements: String = "(_)".repeat(parameters.len());
                    identifier = format!("Function{}::{}",repeated_elements, identifier);
                }
                TType::Tuple { elements } => {
                    let repeated_elements: String = "(_)".repeat(elements.len());
                    identifier = format!("Tuple{}::{}",repeated_elements, identifier);
                }
                TType::Bool => {
                    identifier = format!("Bool::{}", identifier);
                }
                TType::Int => {
                    identifier = format!("Int::{}", identifier);
                }
                TType::Float => {
                    identifier = format!("Float::{}", identifier);
                }
                TType::Char => {
                    identifier = format!("Char::{}", identifier);
                }
                TType::String => {
                    identifier = format!("String::{}", identifier);
                }
                _ => {
                    return Err(self.generate_error_with_pos(
                        format!("E1 Not a valid call: {}", identifier),
                        format!(
                            "No function signature '{}' with {} as arguments, Cant call method on type {}",
                            identifier,
                            argument_types
                                .iter()
                                .map(|t| t.to_string())
                                .collect::<Vec<String>>()
                                .join(", "),
                            ttype.to_string()
                        ),
                        pos,
                    ))
                }
            }
        }
        //dbg!(identifier.clone(), argument_types.clone());

        self.varargs(&identifier, &mut argument_types, &mut arguments);

        if let Some((function_type, function_id, function_kind)) = self
            .environment
            .get_function_type(&identifier, &argument_types)
        {
            self.handle_function_call(
                function_type,
                function_id,
                function_kind,
                arguments,
                argument_types,
                pos,
            )
        } else if let Some((function_type, function_id, function_kind)) =
            self.environment.get_type_capture(&identifier)
        {
            //println!("captured id {}", identifier);
            let pos = self.get_current_token_position();
            self.environment.captured.last_mut().unwrap().insert(
                identifier.clone(),
                Symbol {
                    id: identifier.clone(),
                    ttype: function_type.clone(),
                    pos: Some(pos.clone()),
                    kind: SymbolKind::Captured,
                },
            );
            self.handle_function_call(
                function_type,
                function_id,
                function_kind,
                arguments,
                argument_types,
                pos,
            )
        } else {
            if let Some((function_type, function_id, function_kind)) = self
                .environment
                .get_function_type(&old_identifier, &argument_types)
            {
                self.handle_function_call(
                    function_type,
                    function_id,
                    function_kind,
                    arguments,
                    argument_types,
                    pos,
                )
            } else if let Some((function_type, function_id, function_kind)) =
                self.environment.get_type_capture(&old_identifier)
            {
                //println!("captured id {}", identifier);
                let pos = self.get_current_token_position();
                self.environment.captured.last_mut().unwrap().insert(
                    identifier.clone(),
                    Symbol {
                        id: old_identifier.clone(),
                        ttype: function_type.clone(),
                        pos: Some(pos.clone()),
                        kind: SymbolKind::Captured,
                    },
                );
                self.handle_function_call(
                    function_type,
                    function_id,
                    function_kind,
                    arguments,
                    argument_types,
                    pos,
                )
            } else {
                Err(self.generate_error_with_pos(
                    format!("E1 Not a valid call: {}", identifier),
                    format!(
                        "No function signature '{}' with {} as arguments",
                        identifier,
                        argument_types
                            .iter()
                            .map(|t| t.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                    ),
                    pos,
                ))
            }
        }
    }

    fn handle_function_call(
        &mut self,
        function_type: TType,
        mut function_id: String,
        function_kind: SymbolKind,
        arguments: Vec<Expr>,
        argument_types: Vec<TType>,
        pos: FilePosition,
    ) -> Result<Expr, NovaError> {
        let (parameters, mut return_type) = match function_type {
            TType::Function {
                parameters,
                return_type,
            } => (parameters, return_type),
            _ => {
                return Err(self.generate_error_with_pos(
                    format!(
                        "E2 Not a valid function type: {}",
                        function_type.to_string()
                    ),
                    String::new(),
                    pos,
                ))
            }
        };

        let mut generic_list = self.collect_generics(&[*return_type.clone()]);
        generic_list.extend(self.collect_generics(&parameters));
        //dbg!(generic_list.clone());
        let mut type_map = self.check_and_map_types(
            &parameters,
            &argument_types,
            &mut HashMap::default(),
            pos.clone(),
        )?;

        if let SymbolKind::GenericFunction | SymbolKind::Constructor = function_kind {
            self.map_generic_types(&parameters, &argument_types, &mut type_map, pos.clone())?;
        }
        //dbg!(self.current_token());
        // if current token is @ then parse [T: Type] and replace the generic type and inset that into the type_map
        self.modify_type_map(&mut type_map, pos.clone(), generic_list)?;
        return_type = Box::new(self.get_output(*return_type, &mut type_map, pos.clone())?);

        if let Some(subtype) = self.environment.generic_type_map.get(&function_id) {
            function_id = subtype.clone();
        }

        return Ok(Expr::Literal {
            ttype: *return_type.clone(),
            value: Atom::Call {
                name: function_id,
                arguments,
                position: pos.clone(),
            },
        });
    }

    fn modify_type_map(
        &mut self,
        type_map: &mut HashMap<String, TType>,
        pos: FilePosition,
        generics_list: table::Table<String>,
    ) -> Result<(), NovaError> {
        //dbg!(type_map.clone());
        Ok(if self.current_token().is_symbol('@') {
            self.advance();
            self.consume_symbol('[')?;
            let (generic_type, _) = self.get_identifier()?;
            if !generics_list.has(&generic_type) {
                return Err(NovaError::SimpleTypeError {
                    msg: format!("E2 Type '{}' is not a generic type", generic_type),
                    position: pos,
                });
            }
            self.consume_operator(Operator::Colon)?;
            let ttype = self.ttype()?;
            // check to see if type is generic and then checkt to see if it is live and if it is not live, throw an error
            let generic_list = self.collect_generics(&[ttype.clone()]);
            for generic in generic_list.items {
                if !self.environment.live_generics.last().unwrap().has(&generic) {
                    //dbg!(self.environment.live_generics.last().unwrap());
                    return Err(NovaError::SimpleTypeError {
                        msg: format!("E1 Generic Type '{}' is not live", generic.clone()),
                        position: pos,
                    });
                }
            }
            if let Some(t) = type_map.get(&generic_type) {
                if t != &ttype {
                    return Err(NovaError::TypeError {
                        msg: format!(
                            "E1 Type '{}' is already inferred as {}",
                            generic_type,
                            t.to_string()
                        ),
                        expected: ttype.to_string(),
                        found: generic_type.clone(),
                        position: pos,
                    });
                }
            }
            type_map.insert(generic_type.clone(), ttype.clone());

            while self.current_token().is_symbol(',') {
                self.advance();
                let (generic_type, _) = self.get_identifier()?;
                if !generics_list.has(&generic_type) {
                    return Err(NovaError::SimpleTypeError {
                        msg: format!("E2 Type '{}' is not a generic type", generic_type),
                        position: pos,
                    });
                }
                self.consume_operator(Operator::Colon)?;
                let ttype = self.ttype()?;
                let generic_list = self.collect_generics(&[ttype.clone()]);
                for generic in generic_list.items {
                    if !self.environment.live_generics.last().unwrap().has(&generic) {
                        return Err(NovaError::SimpleTypeError {
                            msg: format!("E1 Generic Type '{}' is not live", generic.clone()),
                            position: pos,
                        });
                    }
                }
                if let Some(t) = type_map.get(&generic_type) {
                    if t != &ttype {
                        return Err(NovaError::TypeError {
                            msg: format!(
                                "E2 Type '{}' is already inferred as {}",
                                generic_type,
                                t.to_string()
                            ),
                            expected: ttype.to_string(),
                            found: generic_type.clone(),
                            position: pos,
                        });
                    }
                }
                type_map.insert(generic_type.clone(), ttype.clone());
            }
            self.consume_symbol(']')?;
            //dbg!(type_map.clone());
        })
    }

    fn map_generic_types(
        &mut self,
        parameters: &[TType],
        argument_types: &[TType],
        type_map: &mut HashMap<String, TType>,
        pos: FilePosition,
    ) -> Result<(), NovaError> {
        for (param_type, arg_type) in parameters.iter().zip(argument_types.iter()) {
            if let (
                TType::Custom {
                    name: param_name, ..
                },
                TType::Custom { name: arg_name, .. },
            ) = (param_type, arg_type)
            {
                if let Some(internal_type) = self.environment.generic_type_map.get(arg_name) {
                    if param_name == internal_type {
                        if let Some(param_list) = self.environment.get_type(param_name) {
                            let mut s = self.clone();
                            if let Some(arg_list) = s.environment.get_type(arg_name) {
                                *type_map = self.check_and_map_types(
                                    &[param_list],
                                    &[arg_list],
                                    type_map,
                                    pos.clone(),
                                )?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
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
        let mut arguments = self.get_field_arguments(&identifier, pos.clone())?;
        let mut argument_types: Vec<TType> = arguments.iter().map(|t| t.get_type()).collect();

        match self.current_token() {
            Token::Operator {
                operator: Operator::Colon,
                ..
            } => {
                self.advance();
                // call get closure
                let (typeinput, input, output, statement, captured) = self.bar_closure()?;
                //dbg!(typeinput.clone(), input.clone(), output.clone(), statement.clone());
                let last_closure = Expr::Closure {
                    ttype: TType::Function {
                        parameters: typeinput,
                        return_type: Box::new(output),
                    },
                    args: input,
                    body: statement,
                    captures: captured,
                };
                argument_types.push(last_closure.get_type());
                arguments.push(last_closure);
            }
            _ => {}
        }

        if argument_types.is_empty() {
            argument_types.push(TType::None)
        }

        self.varargs(&identifier, &mut argument_types, &mut arguments);

        if let Some((function_type, function_id, function_kind)) = self
            .environment
            .get_function_type(&identifier, &argument_types)
        {
            self.handle_function_call(
                function_type,
                function_id,
                function_kind,
                arguments,
                argument_types,
                pos,
            )
        } else if let Some((function_type, function_id, function_kind)) =
            self.environment.get_type_capture(&identifier)
        {
            //println!("captured id: call {}", identifier);
            let pos = self.get_current_token_position();
            self.environment.captured.last_mut().unwrap().insert(
                identifier.clone(),
                Symbol {
                    id: identifier.clone(),
                    ttype: function_type.clone(),
                    pos: Some(pos.clone()),
                    kind: SymbolKind::Captured,
                },
            );
            self.handle_function_call(
                function_type,
                function_id,
                function_kind,
                arguments,
                argument_types,
                pos,
            )
        } else {
            Err(self.generate_error_with_pos(
                format!("E1 Not a valid call: {}", identifier),
                format!(
                    "No function signature '{}' with {} as arguments",
                    identifier,
                    argument_types
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
                pos,
            ))
        }
    }

    fn get_field_arguments(
        &mut self,
        identifier: &str,
        pos: FilePosition,
    ) -> Result<Vec<Expr>, NovaError> {
        if let Some(fields) = self.environment.custom_types.get(identifier) {
            if self.current_token().is_symbol('{') {
                self.field_list(identifier, fields.to_vec(), pos)
            } else {
                self.argument_list()
            }
        } else {
            self.argument_list()
        }
    }

    fn replace_generic_types(&self, ttype: &TType, x: &[String], type_params: &[TType]) -> TType {
        match ttype {
            TType::Generic { name: n } => {
                if let Some(index) = x.iter().position(|x| x == n) {
                    type_params[index].clone()
                } else {
                    ttype.clone()
                }
            }
            TType::None
            | TType::Any
            | TType::Int
            | TType::Float
            | TType::Bool
            | TType::String
            | TType::Char
            | TType::Void
            | TType::Auto => ttype.clone(),
            TType::Custom {
                name,
                type_params: inner_params,
            } => {
                let new_params = inner_params
                    .iter()
                    .map(|param| self.replace_generic_types(param, x, type_params))
                    .collect();
                TType::Custom {
                    name: name.clone(),
                    type_params: new_params,
                }
            }
            TType::List { inner } => TType::List {
                inner: Box::new(self.replace_generic_types(inner, x, type_params)),
            },
            TType::Function {
                parameters,
                return_type,
            } => {
                let new_params = parameters
                    .iter()
                    .map(|param| self.replace_generic_types(param, x, type_params))
                    .collect();
                TType::Function {
                    parameters: new_params,
                    return_type: Box::new(self.replace_generic_types(return_type, x, type_params)),
                }
            }
            TType::Option { inner } => TType::Option {
                inner: Box::new(self.replace_generic_types(inner, x, type_params)),
            },
            TType::Tuple { elements } => {
                let new_elements = elements
                    .iter()
                    .map(|element| self.replace_generic_types(element, x, type_params))
                    .collect();
                TType::Tuple {
                    elements: new_elements,
                }
            }
        }
    }

    fn field(
        &mut self,
        identifier: String,
        mut lhs: Expr,
        pos: FilePosition,
    ) -> Result<Expr, NovaError> {
        if let Some(type_name) = lhs.get_type().custom_to_string() {
            if let Some(fields) = self.environment.custom_types.get(&type_name) {
                //dbg!(&identifier, lhs.get_type().custom_to_string(), fields);
                let new_fields =
                    if let Some(x) = self.environment.generic_type_struct.get(&type_name) {
                        let TType::Custom { type_params, .. } = lhs.get_type() else {
                            panic!("not a custom type")
                        };
                        //dbg!(&fields);
                        fields
                            .iter()
                            .map(|(name, ttype)| {
                                let new_ttype = self.replace_generic_types(ttype, x, &type_params);
                                (name.clone(), new_ttype)
                            })
                            .collect::<Vec<(String, TType)>>()
                    } else {
                        fields.clone()
                    };
                //dbg!(&new_fields);
                if let Some((index, field_type)) = self.find_field(&identifier, &new_fields) {
                    lhs = Expr::Field {
                        ttype: field_type.clone(),
                        name: type_name.clone(),
                        index,
                        expr: Box::new(lhs),
                        position: pos.clone(),
                    };
                } else {
                    return self.generate_field_not_found_error(
                        &identifier,
                        &type_name,
                        fields,
                        pos,
                    );
                }
            } else {
                return self.generate_field_not_found_error(&identifier, &type_name, &[], pos);
            }
        } else {
            return Err(self.generate_error_with_pos(
                format!("E1 Not a valid field access: {}", identifier),
                format!("{} is not a custom type", lhs.get_type().to_string()),
                pos,
            ));
        }
        Ok(lhs)
    }

    fn find_field<'a>(
        &self,
        identifier: &str,
        fields: &'a [(String, TType)],
    ) -> Option<(usize, &'a TType)> {
        fields
            .iter()
            .enumerate()
            .find_map(|(index, (field_name, field_type))| {
                if field_name == identifier {
                    Some((index, field_type))
                } else {
                    None
                }
            })
    }

    fn generate_field_not_found_error(
        &self,
        identifier: &str,
        type_name: &str,
        fields: &[(String, TType)],
        pos: FilePosition,
    ) -> Result<Expr, NovaError> {
        let mut lexicon = Lexicon::new();
        for (field_name, _) in fields.iter() {
            lexicon.insert(field_name);
        }
        let corrections = lexicon.corrections_for(identifier);
        Err(self.generate_error_with_pos(
            format!("No field '{}' found for {}", identifier, type_name),
            format!(
                "cannot retrieve field\nDid you mean? {}",
                corrections.join(", ")
            ),
            pos,
        ))
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
                    if let Some(custom_type) = self.environment.get_type(&identifier) {
                        rhs = self.field(
                            field.clone(),
                            Expr::Literal {
                                ttype: custom_type,
                                value: Atom::Id {
                                    name: identifier.clone(),
                                },
                            },
                            pos,
                        )?;
                    } else {
                        return self.generate_identifier_not_found_error(&identifier, pos);
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
                            format!("Incorrect number of arguments"),
                            format!("Got {}, expected {}", arguments.len(), parameters.len()),
                            pos,
                        ));
                    }
                    let input_types: Vec<_> = arguments.iter().map(|arg| arg.get_type()).collect();
                    let mut type_map: HashMap<String, TType> = HashMap::default();
                    type_map = self.check_and_map_types(
                        &parameters,
                        &input_types,
                        &mut type_map,
                        pos.clone(),
                    )?;
                    return_type =
                        Box::new(self.get_output(*return_type.clone(), &mut type_map, pos)?);
                    lhs = Expr::Call {
                        ttype: *return_type,
                        name: "anon".to_string(),
                        function: Box::new(lhs),
                        args: arguments,
                    };
                } else {
                    return Err(self.generate_error_with_pos(
                        format!("Cannot call {}", lhs.get_type().to_string()),
                        format!("Not a function"),
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

    fn generate_identifier_not_found_error(
        &self,
        identifier: &str,
        pos: FilePosition,
    ) -> Result<Expr, NovaError> {
        let mut lexicon = Lexicon::new();
        self.environment.values.last().iter().for_each(|table| {
            table.iter().for_each(|(name, symbol)| {
                if let Symbol {
                    kind: SymbolKind::Variable,
                    ..
                } = symbol
                {
                    lexicon.insert(name);
                }
            })
        });

        let corrections = lexicon.corrections_for(identifier);
        Err(self.generate_error_with_pos(
            format!("'{}' does not exist", identifier),
            format!(
                "Cannot retrieve field\nDid you mean? {}",
                corrections.join(", ")
            ),
            pos,
        ))
    }

    fn index(
        &mut self,
        identifier: String,
        mut lhs: Expr,
        container_type: TType,
    ) -> Result<Expr, NovaError> {
        match container_type {
            TType::List {
                inner: element_type,
            } => {
                self.consume_symbol('[')?;

                let mut is_slice = false;
                let mut end_expr = None;
                let mut step = None;

                let position = self.get_current_token_position();
                let mut start_expr: Option<Box<Expr>> = None;
                if !self.current_token().is_op(Operator::Colon) {
                    start_expr = Some(Box::new(self.expr()?));
                }
                // do list slice if next token is a colon
                // dbg!(self.current_token());
                if self.current_token().is_op(Operator::Colon) {
                    self.advance();
                    if !self.current_token().is_symbol(']') {
                        if self.current_token().is_symbol('$') {
                            self.advance();

                            step = Some(Box::new(self.expr()?));
                        } else {
                            end_expr = Some(Box::new(self.expr()?));
                            if self.current_token().is_symbol('$') {
                                self.advance();
                                step = Some(Box::new(self.expr()?));
                            }
                        }
                    }
                    self.consume_symbol(']')?;

                    if let Some(start_expr) = &start_expr {
                        if start_expr.get_type() != TType::Int {
                            return Err(self.generate_error_with_pos(
                                format!("Must index List with an int"),
                                format!(
                                    "Cannot index into {} with {}",
                                    lhs.get_type().to_string(),
                                    start_expr.get_type().to_string()
                                ),
                                position,
                            ));
                        }
                    }

                    if let Some(step_expr) = &step {
                        if step_expr.get_type() != TType::Int {
                            return Err(self.generate_error_with_pos(
                                format!("Must index List with an int"),
                                format!(
                                    "Cannot index into {} with {}",
                                    lhs.get_type().to_string(),
                                    step_expr.get_type().to_string()
                                ),
                                position,
                            ));
                        }
                    }

                    if let Some(end_expr) = &end_expr {
                        if end_expr.get_type() != TType::Int {
                            return Err(self.generate_error_with_pos(
                                format!("Must index List with an int"),
                                format!(
                                    "Cannot index into {} with {}",
                                    lhs.get_type().to_string(),
                                    end_expr.get_type().to_string()
                                ),
                                position,
                            ));
                        }
                    }

                    is_slice = true;
                } else {
                    self.consume_symbol(']')?;
                }

                if is_slice {
                    lhs = Expr::Sliced {
                        ttype: TType::List {
                            inner: element_type.clone(),
                        },
                        name: identifier.clone(),
                        start: start_expr,
                        end: end_expr,
                        step: step,
                        container: Box::new(lhs),
                        position,
                    };
                } else {
                    if let Some(start_expr) = start_expr {
                        // typecheck
                        if start_expr.get_type() != TType::Int {
                            return Err(self.generate_error_with_pos(
                                format!("Must index List with an int"),
                                format!(
                                    "Cannot index into {} with {}",
                                    lhs.get_type().to_string(),
                                    start_expr.get_type().to_string()
                                ),
                                position,
                            ));
                        }
                        lhs = Expr::Indexed {
                            ttype: *element_type.clone(),
                            name: identifier.clone(),
                            index: start_expr,
                            container: Box::new(lhs),
                            position,
                        };
                    }
                }
                if self.current_token().is_symbol('[') {
                    lhs = self.index(identifier.clone(), lhs, *element_type)?;
                }
            }
            TType::Tuple {
                elements: tuple_elements,
            } => {
                self.consume_symbol('[')?;
                let position = self.get_current_token_position();
                if let Token::Integer { value: index, .. } = self.current_token() {
                    self.advance();
                    self.consume_symbol(']')?;
                    if index as usize >= tuple_elements.len() {
                        return self.generate_tuple_index_error(
                            index,
                            tuple_elements.len(),
                            position,
                        );
                    }
                    let element_type = &tuple_elements[index as usize];
                    lhs = Expr::Indexed {
                        ttype: element_type.clone(),
                        name: identifier.clone(),
                        index: Box::new(Expr::Literal {
                            ttype: TType::Int,
                            value: Atom::Integer { value: index },
                        }),
                        container: Box::new(lhs),
                        position,
                    };
                    if self.current_token().is_symbol('[') {
                        lhs = self.index(identifier.clone(), lhs, element_type.clone())?;
                    }
                } else {
                    return Err(self.generate_error_with_pos(
                        format!("Must index Tuple with an int"),
                        format!(
                            "Cannot index into {} with {}",
                            lhs.get_type().to_string(),
                            self.current_token().to_string()
                        ),
                        position,
                    ));
                }
            }
            _ => {
                return Err(self.generate_error(
                    format!("Cannot index into non-list or non-tuple"),
                    format!("Must be of type list or tuple"),
                ));
            }
        }

        Ok(lhs)
    }

    fn generate_tuple_index_error(
        &self,
        index: i64,
        tuple_size: usize,
        position: FilePosition,
    ) -> Result<Expr, NovaError> {
        Err(self.generate_error_with_pos(
            format!("Tuple cannot index into {index}"),
            format!("Tuple has {} values", tuple_size),
            position,
        ))
    }

    fn anchor(&mut self, identifier: String, pos: FilePosition) -> Result<Expr, NovaError> {
        let anchor = match self.current_token() {
            Token::Operator {
                operator: Operator::RightArrow,
                ..
            } => {
                self.consume_operator(Operator::RightArrow)?;
                let (field, field_position) = self.get_identifier()?;
                if let Some(identifier_type) = self.environment.get_type(&identifier) {
                    let mut arguments =
                        vec![self.create_literal_expr(identifier.clone(), identifier_type.clone())];
                    let left_expr = self.field(
                        field.clone(),
                        self.create_literal_expr(identifier.clone(), identifier_type.clone()),
                        field_position.clone(),
                    )?;
                    arguments.extend(self.argument_list()?);
                    //dbg!(arguments.clone());
                    if let TType::Function {
                        parameters,
                        mut return_type,
                    } = left_expr.get_type()
                    {
                        if arguments.len() != parameters.len() {
                            return Err(self.generate_error_with_pos(
                                format!("E3 Incorrect number of arguments"),
                                format!("Got {}, expected {}", arguments.len(), parameters.len()),
                                field_position,
                            ));
                        }
                        let input_types: Vec<TType> =
                            arguments.iter().map(|arg| arg.get_type()).collect();
                        //dbg!(input_types.clone());
                        let mut type_map: HashMap<String, TType> = HashMap::default();
                        type_map = self.check_and_map_types(
                            &parameters,
                            &input_types,
                            &mut type_map,
                            field_position.clone(),
                        )?;
                        return_type =
                            Box::new(self.get_output(*return_type.clone(), &mut type_map, pos)?);
                        Expr::Call {
                            ttype: *return_type,
                            name: field.to_string(),
                            function: Box::new(left_expr),
                            args: arguments,
                        }
                    } else {
                        return Err(self.generate_error_with_pos(
                            format!("Cannot call {}", left_expr.get_type().to_string()),
                            format!("Not a function"),
                            field_position,
                        ));
                    }
                } else {
                    return Err(self.generate_error_with_pos(
                        format!("Cannot get {field} from {}", identifier.clone()),
                        format!("{} is not defined", identifier),
                        field_position,
                    ));
                }
            }
            Token::Symbol { symbol: '[', .. } => {
                self.handle_indexing(identifier.clone(), pos.clone())?
            }
            Token::Symbol { symbol: '(', .. } => self.call(identifier.clone(), pos)?,
            _ => {
                if self.current_token().is_symbol('{')
                    && self.environment.custom_types.contains_key(&identifier)
                {
                    self.call(identifier.clone(), pos.clone())?
                } else {
                    self.handle_literal_or_capture(identifier.clone(), pos.clone())?
                }
            }
        };

        Ok(anchor)
    }

    fn create_literal_expr(&self, identifier: String, ttype: TType) -> Expr {
        Expr::Literal {
            ttype,
            value: Atom::Id { name: identifier },
        }
    }

    fn handle_indexing(
        &mut self,
        identifier: String,
        position: FilePosition,
    ) -> Result<Expr, NovaError> {
        if let Some(ttype) = self.environment.get_type(&identifier) {
            self.index(
                identifier.clone(),
                self.create_literal_expr(identifier.clone(), ttype.clone()),
                ttype.clone(),
            )
        } else if let Some((ttype, _, kind)) = self.environment.get_type_capture(&identifier) {
            self.environment.captured.last_mut().unwrap().insert(
                identifier.clone(),
                Symbol {
                    id: identifier.clone(),
                    ttype: ttype.clone(),
                    pos: Some(position.clone()),
                    kind: SymbolKind::Captured,
                },
            );
            self.environment.insert_symbol(
                &identifier,
                ttype.clone(),
                Some(position.clone()),
                kind,
            );
            self.index(
                identifier.clone(),
                self.create_literal_expr(identifier.clone(), ttype.clone()),
                ttype.clone(),
            )
        } else {
            let mut lexicon = Lexicon::new();
            for (id, _) in self.environment.values.last().unwrap().iter() {
                lexicon.insert(id)
            }
            let corrections = lexicon.corrections_for(&identifier);
            Err(self.generate_error_with_pos(
                format!("E1 Not a valid symbol: {}", identifier),
                format!(
                    "Unknown identifier\nDid you mean? {}",
                    corrections.join(", ")
                ),
                position,
            ))
        }
    }

    fn handle_literal_or_capture(
        &mut self,
        identifier: String,
        position: FilePosition,
    ) -> Result<Expr, NovaError> {
        if let Some(ttype) = self.environment.get_type(&identifier) {
            //println!("identifier hloc-not-capture {}", identifier);
            Ok(self.create_literal_expr(identifier.clone(), ttype.clone()))
        } else if let Some((ttype, _, kind)) = self.environment.get_type_capture(&identifier) {
            // println!("identifier hloc-capture {}", identifier);
            // println!(
            //     "environment {:?}",
            //     self.environment.captured.last().unwrap()
            // );
            self.environment.captured.last_mut().unwrap().insert(
                identifier.clone(),
                Symbol {
                    id: identifier.clone(),
                    ttype: ttype.clone(),
                    pos: Some(position.clone()),
                    kind: SymbolKind::Captured,
                },
            );
            self.environment.insert_symbol(
                &identifier,
                ttype.clone(),
                Some(position.clone()),
                kind,
            );
            Ok(self.create_literal_expr(identifier.clone(), ttype.clone()))
        } else {
            let mut lexicon = Lexicon::new();
            for (id, _) in self.environment.values.last().unwrap().iter() {
                lexicon.insert(id)
            }
            let corrections = lexicon.corrections_for(&identifier);
            Err(self.generate_error_with_pos(
                format!("E2 Not a valid symbol: {}", identifier),
                format!(
                    "Unknown identifier\nDid you mean? {}",
                    corrections.join(", ")
                ),
                position,
            ))
        }
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
                let mut generic_list = self.collect_generics(&typeinput);
                generic_list.extend(self.collect_generics(&vec![output.clone()]));
                if let Some(livemap) = self.environment.live_generics.last_mut() {
                    for generic in generic_list.items.iter() {
                        // add generics to live map
                        if !livemap.has(generic) {
                            livemap.insert(generic.clone());
                        }
                    }
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

                //dbg!(self.environment.live_generics.last().unwrap().clone());
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
                        //dbg!(c);
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

                for arg in parameters.iter() {
                    let name = arg.1.clone();
                    // check if name is in captured
                    if captured.contains(&name) {
                        captured = captured
                            .iter()
                            .filter(|&x| x != &name)
                            .map(|x| x.clone())
                            .collect();
                    }
                }

                for dc in captured.iter() {
                    if let Some(v) = self.environment.values.last().unwrap().get(dc) {
                        if let SymbolKind::Captured = v.kind {
                        } else {
                            self.environment.captured.last_mut().unwrap().remove(dc);
                        }
                    }
                }

                // check return types

                let (_, has_return) =
                    self.check_returns(&statements, output.clone(), pos.clone())?;
                if !has_return && output != TType::Void {
                    if let Some(Statement::Pass) = statements.last() {
                        // do nothing
                    } else {
                        return Err(self.generate_error(
                            "Function is missing a return statement in a branch".to_string(),
                            "Function missing return".to_string(),
                        ));
                    }
                }

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
                //dbg!(&captured);
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
            Token::Symbol { symbol: '|', .. }
            | Token::Operator {
                operator: Operator::Or,
                ..
            } => {
                let (typeinput, input, output, statement, captured) = self.bar_closure()?;

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

                // add list comprehension using the for keyword
                // if symbol is colon operator then it is a list comprehension
                match self.peek_offset(2) {
                    Some(Token::Keyword {
                        keyword: KeyWord::In,
                        ..
                    }) => {
                        let mut loops = vec![];
                        self.consume_symbol('[')?;
                        // first get ident, then in keyword, then expr, and then any guards
                        let (ident, mut pos) = self.get_identifier()?;
                        self.consume_keyword(KeyWord::In)?;
                        let listexpr = self.expr()?;

                        if let TType::List { inner } = listexpr.get_type() {
                            self.environment.insert_symbol(
                                &ident,
                                *inner.clone(),
                                Some(pos.clone()),
                                SymbolKind::Variable,
                            );
                        } else {
                            return Err(self.generate_error_with_pos(
                                format!("List comprehension must be a list"),
                                format!("{} is not a list", listexpr.get_type().to_string()),
                                pos,
                            ));
                        }

                        loops.push((ident.clone(), listexpr.clone()));
                        // while comma is present, get ident, in keyword, expr
                        while self.current_token().is_symbol(',') {
                            self.consume_symbol(',')?;
                            let (ident, _) = self.get_identifier()?;
                            self.consume_keyword(KeyWord::In)?;
                            let listexpr = self.expr()?;
                            // insert identifer into scope for typechecking
                            if let TType::List { inner } = listexpr.get_type() {
                                self.environment.insert_symbol(
                                    &ident,
                                    *inner.clone(),
                                    Some(pos.clone()),
                                    SymbolKind::Variable,
                                );
                            } else {
                                return Err(self.generate_error_with_pos(
                                    format!("List comprehension must be a list"),
                                    format!("{} is not a list", listexpr.get_type().to_string()),
                                    pos,
                                ));
                            }
                            loops.push((ident.clone(), listexpr.clone()));
                        }
                        self.consume_symbol('|')?;

                        self.environment.push_block();
                        let mut outexpr = vec![self.expr()?];
                        // continue parsing expr if there is a comma after the outexpr
                        if self.current_token().is_symbol(',') {
                            self.advance();
                            outexpr.push(self.expr()?);
                        }
                        // typecheck taht outexpr is not void
                        if outexpr.last().unwrap().get_type() == TType::Void {
                            return Err(self.generate_error_with_pos(
                                format!("List comprehension must return a value"),
                                format!("Return expression is Void"),
                                pos,
                            ));
                        }

                        let mut guards = vec![];
                        // now grab list of guards seprerated by bar
                        while self.current_token().is_symbol('|') {
                            pos = self.get_current_token_position();
                            self.consume_symbol('|')?;
                            guards.push(self.expr()?);
                        }

                        // check that all the guard types are bool
                        for guard in guards.iter() {
                            if guard.get_type() != TType::Bool {
                                return Err(self.generate_error_with_pos(
                                    format!("Guard must be a boolean"),
                                    format!("{} is not a boolean", guard.get_type().to_string()),
                                    pos,
                                ));
                            }
                        }
                        self.environment.pop_block();
                        self.consume_symbol(']')?;
                        // remove ident from scope
                        for (ident, _) in loops.iter().cloned() {
                            if let Some(v) = self.environment.values.last_mut() {
                                if let Some(_) = v.get(&ident) {
                                    v.remove(&ident);
                                }
                            }
                        }
                        left = Expr::ListCompConstructor {
                            ttype: TType::List {
                                inner: Box::new(outexpr.last().unwrap().get_type()),
                            },
                            loops,
                            expr: outexpr,
                            guards,
                        };
                    }
                    _ => {
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

                        let generic_list = self.collect_generics(&[ttype.clone()]);
                        for generic in generic_list.items {
                            if !self.environment.live_generics.last().unwrap().has(&generic) {
                                //dbg!(self.environment.live_generics.last().unwrap().clone());
                                return Err(NovaError::SimpleTypeError {
                                    msg: format!("Generic Type '{}' is not live", generic.clone()),
                                    position: pos,
                                });
                            }
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
                }
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
                    Token::Operator {
                        operator: Operator::DoubleColon,
                        ..
                    } => {
                        if self.environment.custom_types.contains_key(&identifier) {
                            self.advance();
                            let (name, _) = self.get_identifier()?;
                            identifier = format!("{}::{}", identifier, name);
                        } else if self.modules.has(&identifier) {
                            self.advance();
                            let (name, _) = self.get_identifier()?;
                            identifier = format!("{}::{}", identifier, name);
                        }
                    }
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
                    left = self.handle_inner_function_call(left)?;
                }
                Token::Operator {
                    operator: Operator::DoubleColon,
                    ..
                } => {
                    self.consume_operator(Operator::DoubleColon)?;
                    left = self.handle_field_access(left)?;
                }
                Token::Symbol { symbol: '.', .. } => {
                    self.consume_symbol('.')?;
                    left = self.handle_method_chain(left)?;
                }
                Token::Symbol { symbol: '(', .. } => {
                    left = self.handle_function_pointer_call(left)?;
                }
                Token::Symbol { symbol: '[', .. } => {
                    left = self.handle_chain_indexint(left)?;
                }
                _ => {
                    break;
                }
            }
        }

        Ok(left)
    }

    fn bar_closure(
        &mut self,
    ) -> Result<(Vec<TType>, Vec<Arg>, TType, Vec<Statement>, Vec<String>), NovaError> {
        let pos = self.get_current_token_position();
        let parameters = match self.consume_symbol('|') {
            Ok(_) => {
                let p = self.parameter_list()?;
                self.consume_symbol('|')?;
                p
            }
            Err(_) => {
                self.consume_operator(Operator::Or)?;
                vec![]
            }
        };
        let mut typeinput = vec![];
        for arg in parameters.iter() {
            typeinput.push(arg.0.clone())
        }
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
        if typeinput.is_empty() {
            typeinput.push(TType::None)
        }
        let generic_list = self.collect_generics(&typeinput);
        self.environment.live_generics.push(generic_list.clone());
        self.environment.push_scope();
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
            //dbg!(&typeinput, output.clone());
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
        self.environment.live_generics.pop();
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
        for arg in parameters.iter() {
            let name = arg.1.clone();
            // check if name is in captured
            if captured.contains(&name) {
                // remove from captured
                // remove from captured variable
                captured = captured
                    .iter()
                    .filter(|&x| x != &name)
                    .map(|x| x.clone())
                    .collect();
            }
        }
        for dc in captured.iter() {
            if let Some(v) = self.environment.values.last().unwrap().get(dc) {
                if let SymbolKind::Captured = v.kind {
                } else {
                    self.environment.captured.last_mut().unwrap().remove(dc);
                }
            }
        }
        Ok((typeinput, input, output, statement, captured))
    }

    fn handle_inner_function_call(&mut self, left: Expr) -> Result<Expr, NovaError> {
        let (target_field, pos) = self.get_identifier()?;
        let mut arguments = vec![left.clone()];
        let function_expr = self.field(target_field.clone(), left.clone(), pos.clone())?;
        arguments.extend(self.argument_list()?);
        //dbg!(arguments.clone());
        self.create_call_expression(function_expr, target_field, arguments, pos)
    }

    fn handle_field_access(&mut self, left: Expr) -> Result<Expr, NovaError> {
        let (field, pos) = self.get_identifier()?;
        //dbg!(&field);
        self.field(field.clone(), left, pos)
    }

    fn handle_method_chain(&mut self, left: Expr) -> Result<Expr, NovaError> {
        self.chain(left)
    }

    fn handle_function_pointer_call(&mut self, left: Expr) -> Result<Expr, NovaError> {
        let pos = self.get_current_token_position();
        let mut arguments = self.argument_list()?;
        if arguments.is_empty() {
            arguments.push(Expr::None)
        }
        self.create_call_expression(left, "anon".to_string(), arguments, pos)
    }

    fn handle_chain_indexint(&mut self, left: Expr) -> Result<Expr, NovaError> {
        self.index("anon".to_string(), left.clone(), left.get_type().clone())
    }

    fn create_call_expression(
        &mut self,
        function_expr: Expr,
        function_name: String,
        arguments: Vec<Expr>,
        pos: FilePosition,
    ) -> Result<Expr, NovaError> {
        if let TType::Function {
            parameters,
            mut return_type,
        } = function_expr.get_type()
        {
            if arguments.len() != parameters.len() {
                return Err(self.generate_error_with_pos(
                    format!("Incorrect number of arguments"),
                    format!("Got {}, expected {}", arguments.len(), parameters.len()),
                    pos.clone(),
                ));
            }
            let mut input_types = vec![];
            for arg in arguments.iter() {
                input_types.push(arg.get_type())
            }
            let mut type_map: HashMap<String, TType> = HashMap::default();
            type_map =
                self.check_and_map_types(&parameters, &input_types, &mut type_map, pos.clone())?;
            return_type = Box::new(self.get_output(*return_type.clone(), &mut type_map, pos)?);
            Ok(Expr::Call {
                ttype: *return_type,
                name: function_name,
                function: Box::new(function_expr),
                args: arguments,
            })
        } else {
            Err(self.generate_error_with_pos(
                format!("Cannot call {}", function_expr.get_type().to_string()),
                format!("Not a function"),
                pos.clone(),
            ))
        }
    }

    fn term(&mut self) -> Result<Expr, NovaError> {
        let mut left_expr = self.factor()?;
        let current_pos = self.get_current_token_position();
        while self.current_token().is_multi_op() {
            if let Some(operation) = self.current_token().get_operator() {
                self.advance();
                let right_expr = self.factor()?;
                if left_expr.clone().get_type() == right_expr.clone().get_type()
                    && (left_expr.clone().get_type() == TType::Int
                        || left_expr.clone().get_type() == TType::Float)
                    && (right_expr.clone().get_type() == TType::Int
                        || right_expr.clone().get_type() == TType::Float)
                {
                    self.check_and_map_types(
                        &[left_expr.clone().get_type()],
                        &[right_expr.clone().get_type()],
                        &mut HashMap::default(),
                        current_pos.clone(),
                    )?;
                    left_expr = self.create_binop_expr(
                        left_expr.clone(),
                        right_expr,
                        operation,
                        left_expr.get_type(),
                    );
                } else {
                    return Err(self.create_type_error(
                        left_expr.clone(),
                        right_expr.clone(),
                        operation,
                        current_pos.clone(),
                    ));
                }
            }
        }
        Ok(left_expr)
    }

    fn expr(&mut self) -> Result<Expr, NovaError> {
        let mut left_expr = self.top_expr()?;
        let current_pos = self.get_current_token_position();
        while self.current_token().is_assign() {
            if let Some(operation) = self.current_token().get_operator() {
                self.advance();
                let right_expr = self.top_expr()?;
                match left_expr.clone() {
                    Expr::ListConstructor { .. }
                    | Expr::Binop { .. }
                    | Expr::Call { .. }
                    | Expr::Unary { .. }
                    | Expr::Closure { .. }
                    | Expr::None => {
                        return Err(self.generate_error_with_pos(
                            "Error: left hand side of `=` must be assignable".to_string(),
                            "Cannot assign a value to a literal value".to_string(),
                            current_pos.clone(),
                        ));
                    }
                    Expr::Literal { value: v, .. } => match v {
                        Atom::Id { .. } => {
                            self.check_and_map_types(
                                &vec![left_expr.get_type()],
                                &vec![right_expr.get_type()],
                                &mut HashMap::default(),
                                current_pos.clone(),
                            )?;
                        }
                        _ => {
                            return Err(self.generate_error_with_pos(
                                format!(
                                    "cannot assign {} to {}",
                                    right_expr.get_type().to_string(),
                                    left_expr.get_type().to_string()
                                ),
                                "Cannot assign a value to a literal value".to_string(),
                                current_pos.clone(),
                            ));
                        }
                    },
                    _ => {
                        if &right_expr.get_type() != &left_expr.get_type() {
                            return Err(self.generate_error_with_pos(
                                format!(
                                    "cannot assign {} to {}",
                                    right_expr.get_type().to_string(),
                                    left_expr.get_type().to_string()
                                ),
                                "Cannot assign a value to a literal value".to_string(),
                                current_pos.clone(),
                            ));
                        }
                    }
                }
                left_expr = Expr::Binop {
                    ttype: TType::Void,
                    op: operation,
                    lhs: Box::new(left_expr),
                    rhs: Box::new(right_expr),
                };
            }
        }

        match self.current_token() {
            Token::Operator {
                operator: Operator::RightTilde,
                ..
            } => {
                //dbg!("right tilde");
                //dbg!(left_expr.clone());
                // the syntax is expr ~> id { statements }
                self.consume_operator(Operator::RightTilde)?;
                let (identifier, pos) = self.get_identifier()?;

                // if current token is { else its expr,
                match self.current_token() {
                    Token::Symbol { symbol: '{', .. } => {
                        // cant assing a void
                        if left_expr.get_type() == TType::Void {
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
                            self.environment.push_block();
                            self.environment.insert_symbol(
                                &identifier,
                                left_expr.get_type(),
                                Some(pos.clone()),
                                SymbolKind::Variable,
                            );
                            let expr_block = self.block_expr_inline()?;
                            self.environment.pop_block();

                            if let Some(Statement::Expression { ttype, .. }) = expr_block.last() {
                                left_expr = Expr::StoreExpr {
                                    ttype: ttype.clone(),
                                    name: identifier.clone(),
                                    expr: Box::new(left_expr),
                                    body: expr_block,
                                };
                            } else {
                                left_expr = Expr::StoreExpr {
                                    ttype: TType::Void,
                                    name: identifier.clone(),
                                    expr: Box::new(left_expr),
                                    body: expr_block,
                                };
                            }
                        }
                    }
                    _ => {
                        // return error
                        return Err(self.generate_error_with_pos(
                            format!("Expected block after `~>`"),
                            "Make sure to use a block after `~>`".to_string(),
                            pos.clone(),
                        ));
                    }
                }
            }
            _ => {}
        }
        Ok(left_expr)
    }
    fn top_expr(&mut self) -> Result<Expr, NovaError> {
        let mut left_expr = self.mid_expr()?;
        let current_pos = self.get_current_token_position();
        while self.current_token().is_relop() {
            if let Some(operation) = self.current_token().get_operator() {
                self.advance();
                let right_expr = self.mid_expr()?;
                match operation {
                    Operator::And | Operator::Or => {
                        if (left_expr.get_type() != TType::Bool)
                            || (right_expr.get_type() != TType::Bool)
                        {
                            return Err(self.generate_error_with_pos(
                                "Logical operation expects bool".to_string(),
                                format!(
                                    "got {} {}",
                                    left_expr.get_type().to_string(),
                                    right_expr.get_type().to_string()
                                ),
                                current_pos.clone(),
                            ));
                        }
                        left_expr =
                            self.create_binop_expr(left_expr, right_expr, operation, TType::Bool);
                    }
                    Operator::GreaterThan
                    | Operator::GtrOrEqu
                    | Operator::LssOrEqu
                    | Operator::LessThan => {
                        match (left_expr.get_type(), right_expr.get_type()) {
                            (TType::Int, TType::Int) => {}
                            (TType::Float, TType::Float) => {}
                            _ => {
                                return Err(self.generate_error_with_pos(
                                    "Comparison operation expects int or float".to_string(),
                                    format!(
                                        "expected {} , but found {}",
                                        left_expr.get_type().to_string(),
                                        right_expr.get_type().to_string()
                                    ),
                                    current_pos.clone(),
                                ));
                            }
                        }
                        left_expr =
                            self.create_binop_expr(left_expr, right_expr, operation, TType::Bool);
                    }
                    _ => {
                        left_expr =
                            self.create_binop_expr(left_expr, right_expr, operation, TType::Bool);
                    }
                }
            }
        }
        Ok(left_expr)
    }

    fn mid_expr(&mut self) -> Result<Expr, NovaError> {
        let mut left_expr = self.term()?;
        let current_pos = self.get_current_token_position();
        while self.current_token().is_adding_op() {
            if let Some(operation) = self.current_token().get_operator() {
                self.advance();
                let right_expr = self.term()?;

                match (left_expr.get_type(), right_expr.get_type()) {
                    (TType::Int, TType::Int)
                    | (TType::Float, TType::Float)
                    | (TType::String, TType::String) => {
                        left_expr = self.create_binop_expr(
                            left_expr.clone(),
                            right_expr,
                            operation,
                            left_expr.get_type(),
                        );
                    }
                    (TType::List { inner }, TType::List { inner: inner2 }) => {
                        if inner == inner2 {
                            left_expr = self.create_binop_expr(
                                left_expr.clone(),
                                right_expr,
                                operation,
                                left_expr.get_type(),
                            );
                        } else {
                            return Err(self.create_type_error(
                                left_expr.clone(),
                                right_expr.clone(),
                                operation,
                                current_pos.clone(),
                            ));
                        }
                    }
                    (_, _) => {
                        return Err(self.create_type_error(
                            left_expr.clone(),
                            right_expr.clone(),
                            operation,
                            current_pos.clone(),
                        ));
                    }
                }
            }
        }
        Ok(left_expr)
    }

    fn create_binop_expr(
        &self,
        left_expr: Expr,
        right_expr: Expr,
        operation: Operator,
        ttype: TType,
    ) -> Expr {
        Expr::Binop {
            ttype,
            op: operation,
            lhs: Box::new(left_expr),
            rhs: Box::new(right_expr),
        }
    }

    fn create_type_error(
        &self,
        left_expr: Expr,
        right_expr: Expr,
        operation: Operator,
        pos: FilePosition,
    ) -> NovaError {
        NovaError::TypeError {
            expected: left_expr.get_type().to_string(),
            found: right_expr.get_type().to_string(),
            position: pos,
            msg: format!(
                "Type error, cannot apply operation {:?} to {} and {}",
                operation,
                right_expr.get_type().to_string(),
                left_expr.get_type().to_string()
            ),
        }
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
                if let Some(_) = self.environment.custom_types.get(&identifier) {
                    let mut type_annotation = vec![];
                    if let Token::Symbol { symbol: '(', .. } = self.current_token() {
                        self.consume_symbol('(')?;

                        let ta = self.ttype()?;
                        type_annotation.push(ta);
                        while self.current_token().is_symbol(',') {
                            self.advance();
                            let ta = self.ttype()?;
                            type_annotation.push(ta);
                        }
                        self.consume_symbol(')')?;
                    }
                    if let Some(generic_len) = self.environment.generic_type_struct.get(&identifier)
                    {
                        if generic_len.len() != type_annotation.iter().count() {
                            return Err(self.generate_error_with_pos(
                                format!("Expected {} type parameters", generic_len.len()),
                                format!("Got {} type parameters", type_annotation.iter().count()),
                                pos,
                            ));
                        }
                    }

                    Ok(TType::Custom {
                        name: identifier,
                        type_params: type_annotation,
                    })
                } else {
                    if let Some(alias) = self.environment.type_alias.get(&identifier) {
                        return Ok(alias.clone());
                    } else {
                        return Err(self.generate_error_with_pos(
                            "Unknown type".to_string(),
                            format!("Unknown type '{identifier}' "),
                            pos,
                        ));
                    }
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

    fn enum_list(&mut self) -> Result<Vec<(TType, String)>, NovaError> {
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
            // if no colon, then its a unit variant
            if !self.current_token().is_op(Operator::Colon) {
                arguments.push((TType::None, identifier));
                if !self.current_token().is_symbol(',') {
                    break;
                }
                self.advance();
                continue;
            }
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
                format!("got {}", test.get_type().to_string()),
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
        let import_filepath = match self.current_token() {
            Token::String {
                value: filepath, ..
            } => {
                self.advance();
                filepath.clone()
            }
            Token::Identifier { name, .. } => {
                let mut import_filepath = {
                    if name == "super" {
                        "..".to_string()
                    } else {
                        name
                    }
                };
                self.advance();
                while self.current_token().is_symbol('.') {
                    self.advance();
                    import_filepath.push_str("/");
                    let (identifier, _) = self.get_identifier()?;
                    if identifier == "super" {
                        import_filepath.push_str("..");
                    } else {
                        import_filepath.push_str(&identifier);
                    }
                }
                //dbg!(self.current_token());
                import_filepath.push_str(".nv");
                import_filepath
            }
            _ => panic!(),
        };
        let resolved_filepath: String = match extract_current_directory(&self.filepath) {
            Some(mut current_dir) => {
                current_dir.push_str(&import_filepath);
                current_dir
            }
            None => import_filepath.clone(),
        };
        let tokens = Lexer::new(&resolved_filepath)?.tokenize()?;
        let mut parser = self.clone();
        parser.repl = false;
        parser.index = 0;
        parser.filepath = resolved_filepath.clone();
        parser.input = tokens;
        parser.parse()?;
        self.environment = parser.environment.clone();
        self.modules = parser.modules.clone();
        Ok(Some(Statement::Block {
            body: parser.ast.program.clone(),
            filepath: resolved_filepath,
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
                self.environment.pop_block();
                return Ok(Some(common::nodes::Statement::Unwrap {
                    ttype: id_type,
                    identifier,
                    body,
                    alternative,
                }));
            } else {
                return Err(self.generate_error_with_pos(
                    format!("unwrap expects an option type"),
                    format!("got {}", id_type.to_string()),
                    pos,
                ));
            }
        } else {
            return Err(self.generate_error_with_pos(
                format!("unknown identifier"),
                format!("got {}", identifier),
                pos,
            ));
        }
    }

    fn match_statement(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("match"))?;
        let expr = self.expr()?;

        if let Some(_) = expr.get_type().custom_to_string() {
        } else {
            return Err(self.generate_error_with_pos(
                format!("Match statement expects an enum type"),
                format!("got {}", expr.get_type().to_string()),
                self.get_current_token_position(),
            ));
        }

        let pos = self.get_current_token_position();
        let mut branches = vec![];
        self.consume_symbol('{')?;
        let mut default_branch = None;
        while !self.current_token().is_symbol('}') {
            let (variant, pos) = self.get_identifier()?;
            if variant == "_" {
                // check to see if default branch is already defined
                if default_branch.is_some() {
                    return Err(self.generate_error_with_pos(
                        format!("default branch already defined"),
                        format!("make sure only one default branch is defined"),
                        pos,
                    ));
                }
                self.consume_operator(Operator::RightArrow)?;
                default_branch = Some(self.block()?);
                continue;
            }
            // collect identifiers
            let mut enum_id = String::new();
            if self.current_token().is_symbol('(') {
                self.consume_symbol('(')?;
                if !self.current_token().is_symbol(')') {
                    enum_id = self.get_identifier()?.0;
                }
                self.consume_symbol(')')?;
            }
            self.consume_operator(Operator::RightArrow)?;

            if let Some(fields) = self
                .environment
                .custom_types
                .get(&expr.get_type().custom_to_string().unwrap())
            {
                let new_fields = if let Some(x) = self
                    .environment
                    .generic_type_struct
                    .get(&expr.get_type().custom_to_string().unwrap())
                {
                    let TType::Custom { type_params, .. } = expr.get_type() else {
                        panic!("not a custom type")
                    };
                    //dbg!(&fields);
                    fields
                        .iter()
                        .map(|(name, ttype)| {
                            let new_ttype = self.replace_generic_types(ttype, x, &type_params);
                            (name.clone(), new_ttype)
                        })
                        .collect::<Vec<(String, TType)>>()
                } else {
                    fields.clone()
                };
                let mut tag = 0;
                // mark if the variant is found
                let mut found = false;
                let mut vtype = TType::None;

                for (i, field) in new_fields.iter().enumerate() {
                    //dbg!(field);
                    if variant == field.0 {
                        tag = i;
                        vtype = field.1.clone();
                        found = true;
                    }
                }

                if vtype != TType::None {
                    if enum_id.is_empty() {
                        return Err(self.generate_error_with_pos(
                            format!("variant '{}' is missing Identifier", variant),
                            format!("Variant(id), id is missing"),
                            pos,
                        ));
                    }
                }

                if !found {
                    return Err(self.generate_error_with_pos(
                        format!("variant '{}' not found in type", variant),
                        format!("make sure the variant is in the type"),
                        pos,
                    ));
                }

                self.environment.push_block();
                self.environment
                    .insert_symbol(&enum_id, vtype, None, SymbolKind::Variable);
                // get expression if no { }

                let body = if self.current_token().is_symbol('{') {
                    let body = self.block()?;
                    branches.push((tag, Some(enum_id.clone()), body.clone()));
                    body.clone()
                } else {
                    let body = self.expr()?;
                    branches.push((
                        tag,
                        Some(enum_id.clone()),
                        vec![Statement::Expression {
                            ttype: body.clone().get_type(),
                            expr: body.clone(),
                        }],
                    ));
                    vec![Statement::Expression {
                        ttype: body.clone().get_type(),
                        expr: body,
                    }]
                };

                self.environment.pop_block();
                if enum_id.is_empty() {
                    branches.push((tag, None, body));
                } else {
                    branches.push((tag, Some(enum_id.clone()), body));
                }
            }
        }
        self.consume_symbol('}')?;

        if let Some(_) = default_branch.clone() {
        } else {
            // check to see if all variants are covered
            let mut covered = vec![];
            for (tag, _, _) in branches.clone() {
                covered.push(tag);
            }
            if let Some(fields) = self
                .environment
                .custom_types
                .get(&expr.get_type().custom_to_string().unwrap())
            {
                let new_fields = if let Some(x) = self
                    .environment
                    .generic_type_struct
                    .get(&expr.get_type().custom_to_string().unwrap())
                {
                    let TType::Custom { type_params, .. } = expr.get_type() else {
                        panic!("not a custom type")
                    };
                    //dbg!(&fields);
                    fields
                        .iter()
                        .map(|(name, ttype)| {
                            let new_ttype = self.replace_generic_types(ttype, x, &type_params);
                            (name.clone(), new_ttype)
                        })
                        .collect::<Vec<(String, TType)>>()
                } else {
                    fields.clone()
                };
                for (i, field) in new_fields.iter().enumerate() {
                    if field.0 != "type" && !covered.contains(&i) {
                        return Err(self.generate_error_with_pos(
                            format!("variant '{}' is not covered", field.0),
                            format!("make sure all variants are covered"),
                            pos,
                        ));
                    }
                }
            }
        }

        Ok(Some(Statement::Match {
            ttype: TType::Void,
            expr,
            arms: branches,
            default: default_branch,
        }))
    }

    // new statement for making type aliases
    // alias identifer = <type>
    fn type_alias(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("alias"))?;
        let (alias, _) = self.get_identifier()?;
        if let Some(_) = self.environment.custom_types.get(&alias) {
            return Err(self.generate_error_with_pos(
                format!("type '{}' already defined", alias),
                format!("try using another name"),
                self.get_current_token_position(),
            ));
        }
        self.consume_operator(Operator::Assignment)?;
        let ttype = self.ttype()?;
        self.environment.type_alias.insert(alias, ttype.clone());
        Ok(None)
    }

    fn statement(&mut self) -> Result<Option<Statement>, NovaError> {
        match self.current_token() {
            Token::Identifier { name: id, .. } => match id.as_str() {
                "match" => self.match_statement(),
                //"unwrap" => self.unwrap(), depricated
                "alias" => self.type_alias(),
                "import" => self.import_file(),
                "pass" => self.pass_statement(),
                "struct" => self.struct_declaration(),
                "if" => self.if_statement(),
                "while" => self.while_statement(),
                "let" => self.let_statement(),
                "return" => self.return_statement(),
                "fn" => self.function_declaration(),
                "enum" => self.enum_declaration(),
                "for" => self.for_statement(),
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
                TType::Custom { type_params, .. } => {
                    contracts.extend(self.collect_generics(&type_params.clone()))
                }
                TType::Tuple { elements } => {
                    contracts.extend(self.collect_generics(&elements.clone()))
                }
                _ => {}
            }
        }
        contracts
    }

    fn enum_declaration(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("enum"))?;
        let (enum_name, position) = self.get_identifier()?;

        // Initialize the struct in the environment for recursive types
        self.environment
            .custom_types
            .insert(enum_name.clone(), vec![]);

        self.environment.enums.insert(enum_name.clone());

        let mut generic_field_names = vec![];
        if let Token::Symbol { symbol: '(', .. } = self.current_token() {
            generic_field_names = self.get_id_list()?;
            self.environment
                .generic_type_struct
                .insert(enum_name.clone(), generic_field_names.clone());
        }

        self.consume_symbol('{')?;
        let parameter_list = self.enum_list()?;
        self.consume_symbol('}')?;
        //dbg!(parameter_list.clone());
        let mut fields: Vec<(String, TType)> = vec![];
        let mut type_parameters = vec![];
        let mut generics_table: Table<String> = table::new();

        for (field_type, field_name) in parameter_list.clone() {
            generics_table.extend(self.collect_generics(&[field_type.clone()]));
            type_parameters.push(field_type.clone());
            fields.push((field_name, field_type));
        }
        fields.push(("type".to_string(), TType::String));

        for generic_type in generics_table.items.iter() {
            if !generic_field_names.contains(generic_type) {
                return Err(self.generate_error_with_pos(
                    format!(
                        "enum '{}' is missing generic type {}",
                        enum_name, generic_type
                    ),
                    "You must include generic types in enum name(...generictypes)".to_string(),
                    position.clone(),
                ));
            }
        }

        let mut field_definitions = vec![];
        for (field_name, field_type) in fields.clone() {
            field_definitions.push(Field {
                identifier: field_name,
                ttype: field_type,
            });
        }

        for variants in field_definitions.clone() {
            //dbg!(variants.clone());

            if generics_table.is_empty() {
                //dbg!(enum_name.clone());
                self.environment.insert_symbol(
                    &format!("{}::{}", enum_name.clone(), variants.identifier.clone()),
                    TType::Function {
                        parameters: vec![variants.ttype.clone()],
                        return_type: Box::new(TType::Custom {
                            name: enum_name.clone(),
                            type_params: vec![],
                        }),
                    },
                    Some(position.clone()),
                    SymbolKind::Constructor,
                );
            } else {
                //dbg!(enum_name.clone());
                let genericmap = generic_field_names
                    .iter()
                    .map(|x| TType::Generic { name: x.clone() })
                    .collect::<Vec<TType>>();

                self.environment.insert_symbol(
                    &format!("{}::{}", enum_name.clone(), variants.identifier.clone()),
                    TType::Function {
                        parameters: vec![variants.ttype.clone()],
                        return_type: Box::new(TType::Custom {
                            name: enum_name.clone(),
                            type_params: genericmap,
                        }),
                    },
                    Some(position.clone()),
                    SymbolKind::Constructor,
                );
            }
        }

        self.environment
            .custom_types
            .insert(enum_name.clone(), fields);

        if !self.environment.has(&enum_name) {
            self.environment.no_override.insert(enum_name.to_string());
        } else {
            return Err(self.generate_error_with_pos(
                format!("Enum '{}' is already instantiated", enum_name),
                "Cannot reinstantiate the same type".to_string(),
                position.clone(),
            ));
        }

        Ok(Some(Statement::Enum {
            ttype: TType::Custom {
                name: enum_name.clone(),
                type_params: vec![],
            },
            identifier: enum_name,
            fields: field_definitions,
        }))
    }

    fn struct_declaration(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("struct"))?;
        let (struct_name, position) = self.get_identifier()?;

        // Initialize the struct in the environment for recursive types
        self.environment
            .custom_types
            .insert(struct_name.clone(), vec![]);

        let mut generic_field_names = vec![];
        if let Token::Symbol { symbol: '(', .. } = self.current_token() {
            generic_field_names = self.get_id_list()?;
            self.environment
                .generic_type_struct
                .insert(struct_name.clone(), generic_field_names.clone());
        }

        self.consume_symbol('{')?;
        let parameter_list = self.parameter_list()?;
        self.consume_symbol('}')?;

        let mut fields: Vec<(String, TType)> = vec![];
        let mut type_parameters = vec![];
        let mut generics_table: Table<String> = table::new();

        for (field_type, field_name) in parameter_list.clone() {
            generics_table.extend(self.collect_generics(&[field_type.clone()]));
            type_parameters.push(field_type.clone());
            fields.push((field_name, field_type));
        }
        fields.push(("type".to_string(), TType::String));

        for generic_type in generics_table.items.iter() {
            if !generic_field_names.contains(generic_type) {
                return Err(self.generate_error_with_pos(
                    format!(
                        "Struct '{}' is missing generic type {}",
                        struct_name, generic_type
                    ),
                    "You must include generic types in struct name(...generictypes)".to_string(),
                    position.clone(),
                ));
            }
        }

        let mut field_definitions = vec![];
        for (field_name, field_type) in fields.clone() {
            field_definitions.push(Field {
                identifier: field_name,
                ttype: field_type,
            });
        }

        if !self.environment.has(&struct_name) {
            self.environment.no_override.insert(struct_name.to_string());
            if generics_table.is_empty() {
                self.environment.insert_symbol(
                    &struct_name,
                    TType::Function {
                        parameters: type_parameters,
                        return_type: Box::new(TType::Custom {
                            name: struct_name.clone(),
                            type_params: vec![],
                        }),
                    },
                    Some(position.clone()),
                    SymbolKind::Constructor,
                );
            } else {
                let genericmap = generic_field_names
                    .iter()
                    .map(|x| TType::Generic { name: x.clone() })
                    .collect::<Vec<TType>>();

                self.environment.insert_symbol(
                    &struct_name,
                    TType::Function {
                        parameters: type_parameters,
                        return_type: Box::new(TType::Custom {
                            name: struct_name.clone(),
                            type_params: genericmap,
                        }),
                    },
                    Some(position.clone()),
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
                position.clone(),
            ));
        }

        Ok(Some(Statement::Struct {
            ttype: TType::Custom {
                name: struct_name.clone(),
                type_params: vec![],
            },
            identifier: struct_name,
            fields: field_definitions,
        }))
    }

    fn for_statement(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("for"))?;

        if let Some(Token::Keyword {
            keyword: KeyWord::In,
            ..
        }) = self.peek_offset(1)
        {
            // Handle foreach statement

            let (identifier, pos) = self.get_identifier()?;
            if self.environment.has(&identifier) {
                return Err(self.generate_error_with_pos(
                    format!("identifier already used"),
                    format!("identifier '{identifier}' is already used within this scope"),
                    pos.clone(),
                ));
            }
            self.consume_keyword(KeyWord::In)?;
            let arraypos = self.get_current_token_position();
            let array = self.expr()?;
            //dbg!(self.current_token());
            // check for inclusiverange operator
            match self.current_token() {
                Token::Operator {
                    operator: Operator::InclusiveRange,
                    ..
                } => {
                    let start_range = array;
                    self.consume_operator(Operator::InclusiveRange)?;
                    let end_range = self.expr()?;
                    self.environment.push_block();
                    self.environment.insert_symbol(
                        &identifier,
                        TType::Int,
                        Some(pos),
                        SymbolKind::Variable,
                    );
                    let body = self.block()?;
                    self.environment.pop_block();
                    Ok(Some(Statement::ForRange {
                        identifier,
                        body,
                        start: start_range,
                        end: end_range,
                        inclusive: true,
                        step: None,
                    }))
                }
                Token::Operator {
                    operator: Operator::ExclusiveRange,
                    ..
                } => {
                    let start_range = array;
                    self.consume_operator(Operator::ExclusiveRange)?;
                    let end_range = self.expr()?;
                    self.environment.push_block();
                    self.environment.insert_symbol(
                        &identifier,
                        TType::Int,
                        Some(pos),
                        SymbolKind::Variable,
                    );
                    let body = self.block()?;
                    self.environment.pop_block();
                    Ok(Some(Statement::ForRange {
                        identifier,
                        body,
                        start: start_range,
                        end: end_range,
                        inclusive: false,
                        step: None,
                    }))
                }
                _ => {
                    self.environment.push_block();
                    // check if array has type array and then assign identifier to that type
                    if let TType::List { inner } = array.get_type() {
                        self.environment.insert_symbol(
                            &identifier,
                            *inner,
                            Some(pos),
                            SymbolKind::Variable,
                        )
                    } else {
                        return Err(self.generate_error_with_pos(
                            format!("foreach can only iterate over arrays"),
                            format!("got {}", array.get_type().to_string()),
                            arraypos.clone(),
                        ));
                    }
                    let body = self.block()?;
                    self.environment.pop_block();

                    Ok(Some(Statement::Foreach {
                        identifier,
                        expr: array,
                        body,
                    }))
                }
            }
        } else {
            // Handle regular for statement
            let init = self.expr()?;
            self.consume_symbol(';')?;
            let testpos = self.get_current_token_position();
            let test = self.expr()?;
            self.consume_symbol(';')?;
            let inc = self.expr()?;
            if test.get_type() != TType::Bool && test.get_type() != TType::Void {
                return Err(self.generate_error_with_pos(
                    format!("test expression must return a bool"),
                    format!("got {}", test.get_type().to_string()),
                    testpos,
                ));
            }
            self.environment.push_block();
            let body = self.block()?;
            self.environment.pop_block();
            Ok(Some(Statement::For {
                init,
                test,
                inc,
                body,
            }))
        }
    }

    fn while_statement(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("while"))?;
        let testpos = self.get_current_token_position();
        let test = self.top_expr()?;
        if test.get_type() != TType::Bool && test.get_type() != TType::Void {
            return Err(self.generate_error_with_pos(
                format!("test expression must return a bool"),
                format!("got {}", test.get_type().to_string()),
                testpos,
            ));
        }
        self.environment.push_block();
        let statements = self.block()?;
        self.environment.pop_block();

        Ok(Some(Statement::While {
            test,
            body: statements,
        }))
    }

    fn if_statement(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("if"))?;

        if self.current_token().is_id("let") {
            // Handle if let statement
            self.advance(); // consume 'let'
            let mut global = false;
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
                    format!("got {}", expr.get_type().to_string()),
                    pos.clone(),
                ));
            };

            // make sure symbol doesn't already exist
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
                self.environment.pop_block();
                Ok(Some(Statement::IfLet {
                    ttype: expr.get_type(),
                    identifier,
                    expr,
                    body,
                    alternative,
                    global,
                }))
            }
        } else {
            // Handle regular if statement
            let testpos = self.get_current_token_position();
            let test = self.top_expr()?;
            if test.get_type() != TType::Bool {
                return Err(self.generate_error_with_pos(
                    format!("If statement's expression must return a bool"),
                    format!("got {}", test.get_type().to_string()),
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
    }

    fn let_statement(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("let"))?;
        let mut global = false;
        // refactor out into two parsing ways for ident. one with module and one without
        let (mut identifier, mut pos) = self.get_identifier()?;
        if self.modules.has(&identifier) {
            // throw error
            return Err(self.generate_error_with_pos(
                format!("Cannot use module as identifier"),
                format!("got {}", identifier),
                pos.clone(),
            ));
        }
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
            match (
                self.check_and_map_types(
                    &vec![ttype.clone()],
                    &vec![expr.get_type()],
                    &mut HashMap::default(),
                    pos.clone(),
                ),
                self.check_and_map_types(
                    &vec![expr.get_type()],
                    &vec![ttype.clone()],
                    &mut HashMap::default(),
                    pos.clone(),
                ),
            ) {
                (Ok(_), Ok(_)) => {}
                _ => {
                    return Err(self.generate_error_with_pos(
                        format!(
                            "Cannot assign {} to {}",
                            expr.get_type().to_string(),
                            ttype.to_string()
                        ),
                        "Make sure the expression returns the givin type".to_string(),
                        pos.clone(),
                    ));
                }
            }
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
            //dbg!(&self.environment);
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

    fn return_statement(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("return"))?;
        let expr = self.expr()?;
        Ok(Some(Statement::Return {
            ttype: expr.get_type(),
            expr,
        }))
    }

    fn is_generic(&self, params: &[TType]) -> bool {
        for param in params {
            match param {
                TType::Generic { .. } => return true,
                TType::Function {
                    parameters,
                    return_type,
                } => {
                    // dbg!(parameters.clone(), return_type.clone());
                    if self.is_generic(parameters) || self.is_generic(&[*return_type.clone()]) {
                        return true;
                    }
                }
                TType::List { inner } => {
                    if self.is_generic(&[*inner.clone()]) {
                        return true;
                    }
                }
                TType::Option { inner } => {
                    if self.is_generic(&[*inner.clone()]) {
                        return true;
                    }
                }
                TType::Custom { type_params, .. } => {
                    if self.is_generic(type_params) {
                        return true;
                    }
                }
                TType::Tuple { elements } => {
                    if self.is_generic(elements) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn function_declaration(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("fn"))?;

        let mut is_extended = false;
        let mut is_mod = false;
        let mut get_first = false;
        // check to see if next is the extends keyword with a custom type name and get the custom type name
        let mut custom_type = "".to_string();
        if self.current_token().is_id("extends") {
            self.advance();
            // if current token is ( then get the custom type name , otherwise extend from first argument
            if self.current_token().is_symbol('(') {
                self.consume_symbol('(')?;
                (custom_type, _) = self.get_identifier()?;
                // check to see if its a valid custom type
                if !self.environment.custom_types.contains_key(&custom_type) {
                    return Err(self.generate_error_with_pos(
                        format!("Custom type {} does not exist", custom_type),
                        "Cannot extend a non existent custom type".to_string(),
                        self.get_current_token_position(),
                    ));
                }
                self.consume_symbol(')')?;
                is_extended = true;
            } else {
                get_first = true;
            }
        } else if self.current_token().is_id("mod") {
            self.advance();
            self.consume_symbol('(')?;
            (custom_type, _) = self.get_identifier()?;
            // check to see if its a valid custom type
            if !self.modules.has(&custom_type) {
                return Err(self.generate_error_with_pos(
                    format!("Module {} does not exist", custom_type),
                    "Cannot extend a non existent module".to_string(),
                    self.get_current_token_position(),
                ));
            }
            self.consume_symbol(')')?;
            is_mod = true;
        }

        let (mut identifier, pos) = self.get_identifier()?;

        if is_extended || is_mod {
            identifier = format!("{}::{}", custom_type, identifier);
        }

        // get parameters
        self.consume_symbol('(')?;
        let parameters = self.parameter_list()?;
        //dbg!(&parameters);
        self.consume_symbol(')')?;
        // get output type

        if !is_extended && get_first {
            //println!("{} {}", identifier, parameters.len());
            if let Some((ttype, _)) = parameters.first() {
                match ttype {
                    TType::Custom { name, .. } => {
                        identifier = format!("{}::{}", name, identifier);
                    }
                    TType::List { .. } => {
                        identifier = format!("List::{}", identifier);
                    }
                    TType::Option { .. } => {
                        identifier = format!("Option::{}", identifier);
                    }
                    TType::Function { parameters, .. } => {
                        let repeated_elements: String = "(_)".repeat(parameters.len());
                        identifier = format!("Function{}::{}", repeated_elements, identifier);
                    }
                    TType::Tuple { elements } => {
                        let repeated_elements: String = "(_)".repeat(elements.len());
                        identifier = format!("Tuple{}::{}", repeated_elements, identifier);
                    }
                    TType::Bool => {
                        identifier = format!("Bool::{}", identifier);
                    }
                    TType::Int => {
                        identifier = format!("Int::{}", identifier);
                    }
                    TType::Float => {
                        identifier = format!("Float::{}", identifier);
                    }
                    TType::String => {
                        identifier = format!("String::{}", identifier);
                    }
                    TType::Char => {
                        identifier = format!("Char::{}", identifier);
                    }
                    _ => {
                        // error
                        return Err(self.generate_error_with_pos(
                            format!("Cannot extend from type"),
                            "Cannot extend from this type".to_string(),
                            pos.clone(),
                        ));
                    }
                }
            }
        }

        if self.environment.has(&identifier) {
            return Err(self.generate_error_with_pos(
                format!("Generic Function {identifier} already defined"),
                "Cannot overload a generic function".to_string(),
                pos.clone(),
            ));
        }

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
        //dbg!(generate_unique_string(&identifier, &typeinput));
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
                    "Function {identifier} with inputs {} is already defined",
                    typeinput
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
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
            //dbg!(&identifier, self.is_generic(&typeinput));
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

        //dbg!(self.environment.values.clone());
        self.environment.no_override.insert(identifier.clone());
        let mut generic_list = self.collect_generics(&typeinput);
        generic_list.extend(self.collect_generics(&vec![output.clone()]));
        self.environment.live_generics.push(generic_list.clone());
        //dbg!(generic_list);
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

        // capture variables -----------------------------------
        let mut captured: Vec<String> = self
            .environment
            .captured
            .last()
            .unwrap()
            .iter()
            .map(|v| v.0.clone())
            .collect();

        self.environment.pop_scope();
        self.environment.live_generics.pop();
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
        for arg in parameters.iter() {
            let name = arg.1.clone();
            // check if name is in captured
            if captured.contains(&name) {
                // remove from captured
                // remove from captured variable
                captured = captured
                    .iter()
                    .filter(|&x| x != &name)
                    .map(|x| x.clone())
                    .collect();
            }
        }

        for dc in captured.iter() {
            if let Some(v) = self.environment.values.last().unwrap().get(dc) {
                if let SymbolKind::Captured = v.kind {
                } else {
                    self.environment.captured.last_mut().unwrap().remove(dc);
                }
            }
        }

        // done capturing variables ----------------------------

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
        let (_, has_return) = self.check_returns(&statements, output.clone(), pos.clone())?;
        if !has_return && output != TType::Void {
            if let Some(Statement::Pass) = statements.last() {
                // do nothing
            } else {
                if !has_return {
                    return Err(self.generate_error_with_pos(
                        "Function is missing a return statement in a branch".to_string(),
                        "Function missing return".to_string(),
                        pos.clone(),
                    ));
                }
            }
        }
        //dbg!(&captured);
        Ok(Some(Statement::Function {
            ttype: output,
            identifier,
            parameters: input,
            body: statements,
            captures: captured,
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
                    if ttype != &return_type {
                        Err(self.generate_error_with_pos(
                            "Return type does not match function return type".to_string(),
                            format!(
                                "Expected {} got {}",
                                return_type.to_string(),
                                ttype.to_string()
                            ),
                            pos.clone(),
                        ))
                    } else {
                        Ok(true)
                    }
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
                Statement::IfLet {
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
                Statement::Match { arms, default, .. } => {
                    let mut has_return = has_return;
                    for arm in arms {
                        let (_, arm_has_return) =
                            self.check_returns(&arm.2, return_type.clone(), pos.clone())?;
                        has_return = has_return && arm_has_return;
                    }
                    if let Some(default) = default {
                        let (_, default_has_return) =
                            self.check_returns(default, return_type.clone(), pos.clone())?;
                        has_return = has_return && default_has_return;
                    }
                    Ok(has_return)
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

    fn block_expr_inline(&mut self) -> Result<Vec<Statement>, NovaError> {
        self.consume_symbol('{')?;
        let statements = self.compound_statement()?;
        self.consume_symbol('}')?;
        Ok(statements)
    }

    fn compound_statement(&mut self) -> Result<Vec<Statement>, NovaError> {
        let mut initial_statements = vec![];
        if let Some(statement) = self.statement()? {
            initial_statements.push(statement)
        };
        let statements = {
            let mut statements = initial_statements;

            while self.current_token().is_symbol(';') || !self.is_current_eof() {
                let index_change = self.index;
                if self.current_token().is_symbol(';') {
                    self.advance()
                }
                if self.current_token().is_symbol('}') {
                    break;
                }
                if let Some(statement) = self.statement()? {
                    statements.push(statement);
                }
                if self.index == index_change {
                    return Err(self.generate_error(
                        "Expected statement".to_string(),
                        "Expected statement".to_string(),
                    ));
                }
            }
            statements
        };
        Ok(statements)
    }

    pub fn parse(&mut self) -> Result<(), NovaError> {
        // if repl mode no need to parse module
        if self.repl {
            self.ast.program = self.compound_statement()?;
            return self.eof();
        } else {
            if self.current_token().is_id("module") {
                self.consume_identifier(Some("module"))?;
                let (module_name, _) = self.get_identifier()?;
                if self.modules.has(&module_name) {
                    return Ok(());
                }
                self.modules.insert(module_name);
            } else {
                return Err(self.generate_error(
                    "Expected module declaration".to_string(),
                    "Module declaration must be the first statement".to_string(),
                ));
            }
            self.ast.program = self.compound_statement()?;
            return self.eof();
        }
    }
}
