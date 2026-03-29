use std::{
    borrow::Cow,
    collections::HashMap,
    ops::Deref,
    path::{Path, PathBuf},
    rc::Rc,
    str::FromStr,
};

use common::{
    error::{NovaError, NovaResult},
    fileposition::FilePosition,
    nodes::{Arg, Ast, Atom, Expr, Field, Statement, Symbol, SymbolKind},
    table::{self, Table},
    tokens::{
        KeyWord, Operator,
        StructuralSymbol::{self, *},
        Token, TokenList,
        TokenValue::{self, *},
        Unary,
    },
    ttype::{generate_unique_string, TType},
};

use lexer::Lexer;
use typechecker::TypeChecker;

#[derive(Debug, Clone)]
pub struct Parser {
    filepath: Option<Rc<Path>>,
    pub input: TokenList,
    index: usize,
    pub ast: Ast,
    pub typechecker: TypeChecker,
    pub modules: table::Table<Rc<str>>,
}

pub fn default() -> Parser {
    let tc = create_typechecker();
    Parser {
        filepath: None,
        ast: Ast { program: vec![] },
        input: vec![],
        index: 0,
        typechecker: tc,
        modules: Table::new(),
    }
}

pub fn new(filepath: impl AsRef<Path>) -> Parser {
    let tc = create_typechecker();
    Parser {
        filepath: Some(filepath.as_ref().into()),
        ast: Ast { program: vec![] },
        input: vec![],
        index: 0,
        typechecker: tc,
        modules: Table::new(),
    }
}

fn create_typechecker() -> TypeChecker {
    let mut tc = typechecker::new();
    tc.environment.insert_symbol(
        "error",
        TType::Function {
            parameters: vec![TType::None],
            return_type: Box::new(TType::Void),
        },
        None,
        SymbolKind::GenericFunction,
    );
    tc.environment.insert_symbol(
        "todo",
        TType::Function {
            parameters: vec![TType::None],
            return_type: Box::new(TType::Generic { name: "T".into() }),
        },
        None,
        SymbolKind::GenericFunction,
    );
    tc.environment.insert_symbol(
        "unreachable",
        TType::Function {
            parameters: vec![TType::None],
            return_type: Box::new(TType::Generic { name: "T".into() }),
        },
        None,
        SymbolKind::GenericFunction,
    );
    tc.environment.insert_symbol(
        "exit",
        TType::Function {
            parameters: vec![TType::None],
            return_type: Box::new(TType::Void),
        },
        None,
        SymbolKind::GenericFunction,
    );
    tc.environment.insert_symbol(
        "typeof",
        TType::Function {
            parameters: vec![TType::Any],
            return_type: Box::new(TType::String),
        },
        None,
        SymbolKind::GenericFunction,
    );
    tc.environment.insert_symbol(
        "Option::isSome",
        TType::Function {
            parameters: vec![TType::Any],
            return_type: Box::new(TType::Bool),
        },
        None,
        SymbolKind::GenericFunction,
    );
    tc.environment.insert_symbol(
        "Option::unwrap",
        TType::Function {
            parameters: vec![TType::Option {
                inner: Box::new(TType::Generic { name: "a".into() }),
            }],
            return_type: Box::new(TType::Generic { name: "a".into() }),
        },
        None,
        SymbolKind::GenericFunction,
    );
    tc.environment.insert_symbol(
        "Some",
        TType::Function {
            parameters: vec![TType::Generic { name: "a".into() }],
            return_type: Box::new(TType::Option {
                inner: Box::new(TType::Generic { name: "a".into() }),
            }),
        },
        None,
        SymbolKind::GenericFunction,
    );
    tc.environment.insert_symbol(
        "print",
        TType::Function {
            parameters: vec![TType::Any],
            return_type: Box::new(TType::Void),
        },
        None,
        SymbolKind::GenericFunction,
    );
    tc.environment.insert_symbol(
        "println",
        TType::Function {
            parameters: vec![TType::Any],
            return_type: Box::new(TType::Void),
        },
        None,
        SymbolKind::GenericFunction,
    );
    tc.environment.insert_symbol(
        "clone",
        TType::Function {
            parameters: vec![TType::Generic { name: "a".into() }],
            return_type: Box::new(TType::Generic { name: "a".into() }),
        },
        None,
        SymbolKind::GenericFunction,
    );
    tc
}

impl Parser {
    fn eof(&mut self) -> NovaResult<()> {
        if self.current_token().is_none() {
            // check if forward declarations are empty
            if !self.typechecker.environment.forward_declarations.is_empty() {
                let mut forward_decl = vec![];
                for (id, (_, ret, pos)) in self.typechecker.environment.forward_declarations.iter()
                {
                    forward_decl.push((
                        format!("{} -> {} forward declarations not resolved", id, ret),
                        pos.clone(),
                    ));
                }
                let pos = self.get_current_token_position();
                return Err(Box::new(NovaError::Parsing {
                    msg: "Reached end of file".into(),
                    note: "Make sure all forward declarations are resolved".into(),
                    position: pos,
                    extra: Some(forward_decl),
                }));
            }
            Ok(())
        } else {
            Err(Box::new(NovaError::Parsing {
                msg: "Parsing not completed, left over tokens unparsed".into(),
                note: "Make sure your statement ends with Semicolon.".into(),
                position: self.get_current_token_position(),
                extra: None,
            }))
        }
    }

    fn is_current_eof(&mut self) -> bool {
        self.current_token().is_none()
    }

    fn generate_error(
        &self,
        msg: impl Into<Cow<'static, str>>,
        note: impl Into<Cow<'static, str>>,
    ) -> Box<NovaError> {
        Box::new(NovaError::Parsing {
            msg: msg.into(),
            note: note.into(),
            position: self.get_current_token_position(),
            extra: None,
        })
    }

    fn generate_error_with_pos(
        &self,
        msg: impl Into<Cow<'static, str>>,
        note: impl Into<Cow<'static, str>>,
        pos: FilePosition,
    ) -> Box<NovaError> {
        Box::new(NovaError::Parsing {
            msg: msg.into(),
            note: note.into(),
            position: pos,
            extra: None,
        })
    }

    fn get_line_and_row(&self) -> (usize, usize) {
        let Some(t) = self.current_token() else {
            return (0, 0);
        };
        (t.line(), t.col())
    }

    fn get_current_token_position(&self) -> FilePosition {
        self.current_token()
            .map(|t| t.position())
            // unwrap or use previous token position
            .unwrap_or_else(|| {
                self.input
                    .get(self.index - 1)
                    .map_or_else(FilePosition::default, |t| t.position())
            })
    }

    fn consume_operator(&mut self, op: Operator) -> NovaResult<()> {
        match self.current_token() {
            Some(t) if t.is_op(op) => {
                self.advance();
                Ok(())
            }
            unexpected => Err(self.generate_error(
                format!("unexpected operator, got {unexpected:?}"),
                format!("expected {op:?}"),
            )),
        }
    }

    fn consume_symbol(&mut self, sym: StructuralSymbol) -> NovaResult<()> {
        match self.current_token() {
            Some(t) if t.is_symbol(sym) => {
                self.advance();
                Ok(())
            }
            unexpected => Err(self.generate_error(
                format!("unexpected symbol, got {unexpected:?}"),
                format!("expected {:?}", sym),
            )),
        }
    }

    // consume a keyword
    fn consume_keyword(&mut self, kw: KeyWord) -> NovaResult<()> {
        match self.current_token() {
            Some(t) if t.is_keyword(kw) => {
                self.advance();
                Ok(())
            }
            unexpected => Err(self.generate_error(
                format!("unexpected keyword, got {unexpected:?}"),
                format!("expected {kw:?}"),
            )),
        }
    }

    fn consume_identifier(&mut self, symbol: Option<&str>) -> NovaResult<()> {
        match self.current_token() {
            Some(t) if symbol.map_or_else(|| t.is_identifier(), |s| t.is_id(s)) => {
                self.advance();
                Ok(())
            }
            unexpected => Err(self.generate_error(
                format!("unexpected identifier, got {unexpected:?}"),
                match symbol {
                    Some(s) => format!("expecting {s}"),
                    None => "expecting an identifier".to_string(),
                },
            )),
        }
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn current_token(&self) -> Option<&Token> {
        self.input.get(self.index)
    }

    fn current_token_value(&self) -> Option<&TokenValue> {
        self.input.get(self.index).map(|t| &t.value)
    }

    // peek with offset
    fn peek_offset(&self, offset: usize) -> Option<&Token> {
        self.input.get(self.index + offset)
    }

    fn peek_offset_value(&self, offset: usize) -> Option<&TokenValue> {
        self.peek_offset(offset).map(|t| &t.value)
    }

    fn sign(&mut self) -> NovaResult<Option<Unary>> {
        match self.current_token_value() {
            Some(Operator(Operator::Addition)) => Ok(Some(Unary::Positive)),
            Some(Operator(Operator::Subtraction)) => Ok(Some(Unary::Negative)),
            Some(Operator(Operator::Not)) => Ok(Some(Unary::Not)),
            Some(Operator(_)) => Err(self.generate_error(
                format!("unexpected operation, got {:?}", self.current_token_value()),
                "expected unary sign ( + | - )",
            )),
            _ => Ok(None),
        }
    }

    fn expr_list(&mut self) -> NovaResult<Vec<Expr>> {
        let mut exprs = vec![];
        self.consume_symbol(LeftSquareBracket)?;

        if !self
            .current_token()
            .is_some_and(|t| t.is_symbol(RightSquareBracket))
        {
            self.process_expression(&mut exprs)?;
        }

        while self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
            self.advance();
            if self
                .current_token()
                .is_some_and(|t| t.is_symbol(RightSquareBracket))
            {
                break;
            }
            self.process_expression(&mut exprs)?;
        }

        self.consume_symbol(RightSquareBracket)?;
        Ok(exprs)
    }

    fn process_expression(&mut self, exprs: &mut Vec<Expr>) -> NovaResult<()> {
        let pos = self.get_current_token_position();
        let e = self.expr()?;
        if e.get_type() == TType::Void {
            return Err(self.generate_error_with_pos(
                "cannot insert a void expression",
                "expressions must not be void",
                pos,
            ));
        }
        exprs.push(e);
        Ok(())
    }

    fn argument_list(&mut self) -> NovaResult<Vec<Expr>> {
        let mut exprs = vec![];
        self.consume_symbol(LeftParen)?;
        if !self
            .current_token()
            .is_some_and(|t| t.is_symbol(RightParen))
        {
            exprs.push(self.expr()?);
        }
        while self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
            self.advance();
            if self
                .current_token()
                .is_some_and(|t| t.is_symbol(RightParen))
            {
                break;
            }
            exprs.push(self.expr()?);
        }
        self.consume_symbol(RightParen)?;
        Ok(exprs)
    }

    fn field_list(
        &mut self,
        constructor: &str,
        fields: Vec<(Rc<str>, TType)>,
        conpos: FilePosition,
    ) -> NovaResult<Vec<Expr>> {
        let mut field_exprs = HashMap::default();
        self.consume_symbol(LeftBrace)?;
        self.parse_field(&mut field_exprs)?;
        while self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
            self.advance();
            if self
                .current_token()
                .is_some_and(|t| t.is_symbol(RightBrace))
            {
                break;
            }
            self.parse_field(&mut field_exprs)?;
        }
        self.consume_symbol(RightBrace)?;
        self.validate_fields(constructor, &fields, conpos, &field_exprs)
    }

    fn parse_field(&mut self, field_exprs: &mut HashMap<Rc<str>, Expr>) -> NovaResult<()> {
        let (id, _) = self.get_identifier()?;
        self.consume_operator(Operator::Colon)?;
        field_exprs.insert(id, self.expr()?);
        Ok(())
    }

    fn validate_fields(
        &mut self,
        constructor: &str,
        fields: &[(impl AsRef<str>, TType)],
        conpos: FilePosition,
        field_exprs: &HashMap<Rc<str>, Expr>,
    ) -> NovaResult<Vec<Expr>> {
        let mut validated_exprs = vec![];
        for (field_name, field_type) in fields.iter() {
            if field_name.as_ref() == "type" {
                continue;
            }
            if let Some(expr) = field_exprs.get(field_name.as_ref()) {
                self.typechecker.check_and_map_types(
                    std::slice::from_ref(field_type),
                    &[expr.get_type()],
                    &mut HashMap::default(),
                    conpos.clone(),
                )?;
                validated_exprs.push(expr.clone());
            } else {
                return Err(Box::new(NovaError::Parsing {
                    msg: format!("{} is missing field {}", constructor, field_name.as_ref()).into(),
                    note: "".into(),
                    position: conpos,
                    extra: None,
                }));
            }
        }
        if field_exprs.len() != fields.len() - 1 {
            return Err(Box::new(NovaError::Parsing {
                msg: format!(
                    "{} has {} fields, you have {}",
                    constructor,
                    fields.len() - 1,
                    field_exprs.len()
                )
                .into(),
                note: "".into(),
                position: conpos.clone(),
                extra: None,
            }));
        }
        if validated_exprs.len() != fields.len() - 1 {
            return Err(Box::new(NovaError::Parsing {
                msg: format!(
                    "{} has {} fields, not all of them are covered",
                    constructor,
                    fields.len() - 1
                )
                .into(),
                note: "".into(),
                position: conpos,
                extra: None,
            }));
        }
        Ok(validated_exprs)
    }

    fn method(
        &mut self,
        mut identifier: Rc<str>,
        first_argument: Expr,
        pos: FilePosition,
    ) -> NovaResult<Expr> {
        let mut arguments = vec![first_argument];
        arguments.extend(self.argument_list()?);
        let mut argument_types: Vec<TType> = arguments.iter().map(|t| t.get_type()).collect();

        if self
            .current_token()
            .is_some_and(|t| t.is_op(Operator::Colon))
        {
            self.advance();
            // call get closure
            let (typeinput, input, output, statement, captured) = self.bar_closure()?;
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

        if argument_types.is_empty() {
            argument_types.push(TType::None)
        }
        // used last time for stuff like random.println() but removed for now
        // let old_identifier = identifier.clone();
        identifier = if let Some(TType::Custom { name, .. }) = argument_types.first() {
            if self
                .typechecker
                .environment
                .custom_types
                .contains_key(name.as_ref())
            {
                format!("{}::{}", name, identifier).into()
            } else {
                identifier
            }
        } else if let Some(ttype) = argument_types.first() {
            match ttype {
                TType::List { .. } => {
                    format!("List::{}", identifier)
                }
                TType::Option { .. } => {
                     format!("Option::{}", identifier)
                }
                TType::Function {  .. } => {
                     format!("Function::{}", identifier)
                }
                TType::Tuple { ..} => {
                     format!("Tuple::{}", identifier)
                }
                TType::Bool => {
                     format!("Bool::{}", identifier)
                }
                TType::Int => {
                     format!("Int::{}", identifier)
                }
                TType::Float => {
                     format!("Float::{}", identifier)
                }
                TType::Char => {
                     format!("Char::{}", identifier)
                }
                TType::String => {
                     format!("String::{}", identifier)
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
                            ttype,
                        ),
                        pos,
                    ))
                }
            }.into()
        } else {
            identifier
        };

        self.typechecker
            .varargs(&identifier, &mut argument_types, &mut arguments);

        if let Some((function_type, function_id, function_kind)) = self
            .typechecker
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
            self.typechecker.environment.get_type_capture(&identifier)
        {
            //println!("captured id {}", identifier);
            let pos = self.get_current_token_position();
            self.typechecker
                .environment
                .captured
                .last_mut()
                .unwrap()
                .insert(
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

    fn handle_function_call(
        &mut self,
        function_type: TType,
        mut function_id: Rc<str>,
        function_kind: SymbolKind,
        arguments: Vec<Expr>,
        argument_types: Vec<TType>,
        pos: FilePosition,
    ) -> NovaResult<Expr> {
        let (parameters, mut return_type) = match function_type {
            TType::Function {
                parameters,
                return_type,
            } => (parameters, return_type),
            _ => {
                return Err(self.generate_error_with_pos(
                    format!("E2 Not a valid function type: {}", function_type),
                    String::new(),
                    pos,
                ))
            }
        };

        let mut generic_list = TypeChecker::collect_generics(&[*return_type.clone()]);
        generic_list.extend(TypeChecker::collect_generics(&parameters));
        let mut type_map = HashMap::new();
        self.typechecker.check_and_map_types(
            &parameters,
            &argument_types,
            &mut type_map,
            pos.clone(),
        )?;

        if let SymbolKind::GenericFunction | SymbolKind::Constructor = function_kind {
            self.typechecker.map_generic_types(
                &parameters,
                &argument_types,
                &mut type_map,
                pos.clone(),
            )?;
        }
        // if current token is @ then parse [T: Type] and replace the generic type and inset that into the type_map
        self.modify_type_map(&mut type_map, pos.clone(), generic_list)?;
        return_type = Box::new(self.typechecker.get_output(
            *return_type,
            &mut type_map,
            pos.clone(),
        )?);

        if let Some(subtype) = self
            .typechecker
            .environment
            .generic_type_map
            .get(&function_id)
        {
            function_id = subtype.clone();
        }

        Ok(Expr::Literal {
            ttype: *return_type.clone(),
            value: Atom::Call {
                name: function_id,
                arguments,
                position: pos.clone(),
            },
        })
    }

    fn modify_type_map(
        &mut self,
        type_map: &mut HashMap<Rc<str>, TType>,
        pos: FilePosition,
        generics_list: table::Table<Rc<str>>,
    ) -> NovaResult<()> {
        if !self.current_token().is_some_and(|t| t.is_symbol(At)) {
            return Ok(());
        }
        self.advance();
        self.consume_symbol(LeftSquareBracket)?;
        let (generic_type, _) = self.get_identifier()?;
        if !generics_list.has(&generic_type) {
            return Err(Box::new(NovaError::SimpleTypeError {
                msg: format!("E2 Type '{}' is not a generic type", generic_type).into(),
                position: pos,
            }));
        }
        self.consume_operator(Operator::Colon)?;
        let ttype = self.ttype()?;
        // check to see if type is generic and then checkt to see if it is live and if it is not live, throw an error
        let generic_list = TypeChecker::collect_generics(std::slice::from_ref(&ttype));
        for generic in generic_list.items {
            if !self
                .typechecker
                .environment
                .live_generics
                .last()
                .unwrap()
                .has(&generic)
            {
                return Err(Box::new(NovaError::SimpleTypeError {
                    msg: format!("E1 Generic Type '{generic}' is not live").into(),
                    position: pos,
                }));
            }
        }
        if let Some(t) = type_map.get(&generic_type) {
            if t != &ttype {
                return Err(Box::new(NovaError::TypeError {
                    msg: format!("E1 Type '{generic_type}' is already inferred as {t}").into(),
                    expected: ttype.to_string().into(),
                    found: generic_type.to_string().into(),
                    position: pos,
                }));
            }
        }
        type_map.insert(generic_type.clone(), ttype.clone());

        while self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
            self.advance();
            let (generic_type, _) = self.get_identifier()?;
            if !generics_list.has(&generic_type) {
                return Err(Box::new(NovaError::SimpleTypeError {
                    msg: format!("E2 Type '{generic_type}' is not a generic type").into(),
                    position: pos,
                }));
            }
            self.consume_operator(Operator::Colon)?;
            let ttype = self.ttype()?;
            let generic_list = TypeChecker::collect_generics(std::slice::from_ref(&ttype));
            for generic in generic_list.items {
                if !self
                    .typechecker
                    .environment
                    .live_generics
                    .last()
                    .unwrap()
                    .has(&generic)
                {
                    return Err(Box::new(NovaError::SimpleTypeError {
                        msg: format!("E1 Generic Type '{}' is not live", generic).into(),
                        position: pos,
                    }));
                }
            }
            if let Some(t) = type_map.get(&generic_type) {
                if t != &ttype {
                    return Err(Box::new(NovaError::TypeError {
                        msg: format!("E2 Type '{generic_type}' is already inferred as {t}").into(),
                        expected: ttype.to_string().into(),
                        found: generic_type.to_string().into(),
                        position: pos,
                    }));
                }
            }
            type_map.insert(generic_type, ttype.clone());
        }
        self.consume_symbol(RightSquareBracket)?;
        Ok(())
    }

    fn call(
        &mut self,
        identifier: Rc<str>,
        pos: FilePosition,
        first: Option<Expr>,
    ) -> NovaResult<Expr> {
        let mut arguments = self.get_field_arguments(&identifier, pos.clone())?;
        if let Some(first) = first {
            arguments.insert(0, first);
        }
        let mut argument_types: Vec<TType> = arguments.iter().map(|t| t.get_type()).collect();

        if self
            .current_token()
            .is_some_and(|t| t.is_op(Operator::Colon))
        {
            self.advance();
            // call get closure
            let (typeinput, input, output, statement, captured) = self.bar_closure()?;
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

        if argument_types.is_empty() {
            argument_types.push(TType::None)
        }

        self.typechecker
            .varargs(&identifier, &mut argument_types, &mut arguments);

        if let Some((function_type, function_id, function_kind)) = self
            .typechecker
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
            self.typechecker.environment.get_type_capture(&identifier)
        {
            //println!("captured id: call {}", identifier);
            let pos = self.get_current_token_position();
            self.typechecker
                .environment
                .captured
                .last_mut()
                .unwrap()
                .insert(
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
    ) -> NovaResult<Vec<Expr>> {
        if let Some(fields) = self.typechecker.environment.custom_types.get(identifier) {
            if self.current_token().is_some_and(|t| t.is_symbol(LeftBrace)) {
                self.field_list(identifier, fields.to_vec(), pos)
            } else {
                self.argument_list()
            }
        } else {
            self.argument_list()
        }
    }

    fn field(&mut self, identifier: Rc<str>, mut lhs: Expr, pos: FilePosition) -> NovaResult<Expr> {
        if let Some(type_name) = lhs.get_type().custom_to_string() {
            if let Some(fields) = self.typechecker.environment.custom_types.get(type_name) {
                let new_fields = if let Some(x) = self
                    .typechecker
                    .environment
                    .generic_type_struct
                    .get(type_name)
                {
                    let TType::Custom { type_params, .. } = lhs.get_type() else {
                        panic!("not a custom type")
                    };
                    fields
                        .iter()
                        .map(|(name, ttype)| {
                            let new_ttype =
                                TypeChecker::replace_generic_types(ttype, x, &type_params);
                            (name.clone(), new_ttype)
                        })
                        .collect()
                } else {
                    fields.clone()
                };
                if let Some((index, field_type)) = self.find_field(&identifier, &new_fields) {
                    lhs = Expr::Field {
                        ttype: field_type.clone(),
                        name: type_name.into(),
                        index,
                        expr: Box::new(lhs),
                        position: pos.clone(),
                    };
                } else {
                    return self.generate_field_not_found_error(&identifier, type_name, pos);
                }
            } else {
                return self.generate_field_not_found_error(&identifier, type_name, pos);
            }
        } else {
            // check if dynamic type with fields in contract
            if let TType::Dyn { contract, .. } = lhs.get_type() {
                if let Some((name, field_type)) = contract
                    .iter()
                    .find(|(name, _)| name.as_ref() == identifier.as_ref())
                {
                    lhs = Expr::DynField {
                        ttype: field_type.clone(),
                        name: name.clone(),
                        expr: Box::new(lhs),
                        position: pos.clone(),
                    };
                } else {
                    return self.generate_field_not_found_error(&identifier, "Dyn", pos);
                }
            } else {
                return Err(self.generate_error_with_pos(
                    format!("E1 Not a valid field access: {}", identifier),
                    format!("{} is not a custom type", lhs.get_type()),
                    pos,
                ));
            }
        }
        Ok(lhs)
    }

    fn find_field<'a>(
        &self,
        identifier: &str,
        fields: &'a [(impl AsRef<str>, TType)],
    ) -> Option<(usize, &'a TType)> {
        fields
            .iter()
            .enumerate()
            .find_map(|(index, (field_name, field_type))| {
                if field_name.as_ref() == identifier {
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
        pos: FilePosition,
    ) -> NovaResult<Expr> {
        Err(self.generate_error_with_pos(
            format!("No field '{}' found for {}", identifier, type_name),
            "cannot retrieve field".to_string(),
            pos,
        ))
    }

    fn chain(&mut self, mut lhs: Expr) -> NovaResult<Expr> {
        let (identifier, pos) = self.get_identifier()?;
        match self.current_token_value() {
            Some(Operator(Operator::RightArrow)) => {
                self.advance();
                lhs = self.method(identifier, lhs, pos)?;
            }
            Some(Operator(Operator::DoubleColon)) => {
                let mut rhs = lhs.clone();
                while self
                    .current_token()
                    .is_some_and(|t| t.is_op(Operator::DoubleColon))
                {
                    self.consume_operator(Operator::DoubleColon)?;
                    let (field, pos) = self.get_identifier()?;
                    if let Some(custom_type) = self.typechecker.environment.get_type(&identifier) {
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
                            "Incorrect number of arguments",
                            format!("Got {}, expected {}", arguments.len(), parameters.len()),
                            pos,
                        ));
                    }
                    let input_types: Vec<_> = arguments.iter().map(|arg| arg.get_type()).collect();
                    let mut type_map = HashMap::default();
                    self.typechecker.check_and_map_types(
                        &parameters,
                        &input_types,
                        &mut type_map,
                        pos.clone(),
                    )?;
                    return_type = Box::new(self.typechecker.get_output(
                        *return_type.clone(),
                        &mut type_map,
                        pos,
                    )?);
                    lhs = Expr::Call {
                        ttype: *return_type,
                        name: "anon".into(),
                        function: Box::new(lhs),
                        args: arguments,
                    };
                } else {
                    return Err(self.generate_error_with_pos(
                        format!("Cannot call {}", lhs.get_type()),
                        "Not a function",
                        pos,
                    ));
                }
            }
            Some(StructuralSymbol(LeftParen)) => {
                lhs = self.method(identifier, lhs, pos)?;
            }
            Some(StructuralSymbol(LeftSquareBracket)) => {
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
    ) -> NovaResult<Expr> {
        Err(self.generate_error_with_pos(
            format!("'{}' does not exist", identifier),
            "Cannot retrieve field".to_string(),
            pos,
        ))
    }

    fn index(
        &mut self,
        identifier: Rc<str>,
        mut lhs: Expr,
        container_type: TType,
    ) -> NovaResult<Expr> {
        match container_type {
            TType::List {
                inner: element_type,
            } => {
                self.consume_symbol(LeftSquareBracket)?;

                let mut is_slice = false;
                let mut end_expr = None;
                let mut step = None;

                let position = self.get_current_token_position();
                let mut start_expr: Option<Box<Expr>> = None;
                if !self
                    .current_token()
                    .is_some_and(|t| t.is_op(Operator::Colon))
                {
                    start_expr = Some(Box::new(self.expr()?));
                }
                // do list slice if next token is a colon
                if self
                    .current_token()
                    .is_some_and(|t| t.is_op(Operator::Colon))
                {
                    self.advance();
                    if !self
                        .current_token()
                        .is_some_and(|t| t.is_symbol(RightSquareBracket))
                    {
                        if self
                            .current_token()
                            .is_some_and(|t| t.is_symbol(DollarSign))
                        {
                            self.advance();

                            step = Some(Box::new(self.expr()?));
                        } else {
                            end_expr = Some(Box::new(self.expr()?));
                            if self
                                .current_token()
                                .is_some_and(|t| t.is_symbol(DollarSign))
                            {
                                self.advance();
                                step = Some(Box::new(self.expr()?));
                            }
                        }
                    }
                    self.consume_symbol(RightSquareBracket)?;

                    if let Some(start_expr) = &start_expr {
                        if start_expr.get_type() != TType::Int {
                            return Err(self.generate_error_with_pos(
                                "Must index List with an int",
                                format!(
                                    "Cannot index into {} with {}",
                                    lhs.get_type(),
                                    start_expr.get_type()
                                ),
                                position,
                            ));
                        }
                    }

                    if let Some(step_expr) = &step {
                        if step_expr.get_type() != TType::Int {
                            return Err(self.generate_error_with_pos(
                                "Must index List with an int",
                                format!(
                                    "Cannot index into {} with {}",
                                    lhs.get_type(),
                                    step_expr.get_type()
                                ),
                                position,
                            ));
                        }
                    }

                    if let Some(end_expr) = &end_expr {
                        if end_expr.get_type() != TType::Int {
                            return Err(self.generate_error_with_pos(
                                "Must index List with an int",
                                format!(
                                    "Cannot index into {} with {}",
                                    lhs.get_type(),
                                    end_expr.get_type()
                                ),
                                position,
                            ));
                        }
                    }

                    is_slice = true;
                } else {
                    self.consume_symbol(RightSquareBracket)?;
                }

                if is_slice {
                    lhs = Expr::Sliced {
                        ttype: TType::List {
                            inner: element_type.clone(),
                        },
                        name: identifier.clone(),
                        start: start_expr,
                        end: end_expr,
                        step,
                        container: Box::new(lhs),
                        position,
                    };
                } else if let Some(start_expr) = start_expr {
                    // typecheck
                    if start_expr.get_type() != TType::Int {
                        return Err(self.generate_error_with_pos(
                            "Must index List with an int",
                            format!(
                                "Cannot index into {} with {}",
                                lhs.get_type(),
                                start_expr.get_type()
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
                if self
                    .current_token()
                    .is_some_and(|t| t.is_symbol(LeftSquareBracket))
                {
                    lhs = self.index(identifier.clone(), lhs, *element_type)?;
                }
            }
            TType::Tuple {
                elements: tuple_elements,
            } => {
                self.consume_symbol(LeftSquareBracket)?;
                let position = self.get_current_token_position();
                if let Some(&Integer(index)) = self.current_token_value() {
                    self.advance();
                    self.consume_symbol(RightSquareBracket)?;
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
                    if self
                        .current_token()
                        .is_some_and(|t| t.is_symbol(LeftSquareBracket))
                    {
                        lhs = self.index(identifier.clone(), lhs, element_type.clone())?;
                    }
                } else {
                    return Err(self.generate_error_with_pos(
                        "Must index Tuple with an int",
                        format!(
                            "Cannot index into {} with {:?}",
                            lhs.get_type(),
                            self.current_token()
                        ),
                        position,
                    ));
                }
            }
            _ => {
                return Err(self.generate_error(
                    "Cannot index into non-list or non-tuple",
                    "Must be of type list or tuple",
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
    ) -> NovaResult<Expr> {
        Err(self.generate_error_with_pos(
            format!("Tuple cannot index into {index}"),
            format!("Tuple has {} values", tuple_size),
            position,
        ))
    }

    fn anchor(&mut self, identifier: Rc<str>, pos: FilePosition) -> NovaResult<Expr> {
        let anchor = match self.current_token_value() {
            Some(Operator(Operator::RightArrow)) => {
                self.consume_operator(Operator::RightArrow)?;
                let (field, field_position) = self.get_identifier()?;
                if let Some(identifier_type) = self.typechecker.environment.get_type(&identifier) {
                    let mut arguments =
                        vec![self.create_literal_expr(identifier.clone(), identifier_type.clone())];
                    let left_expr = self.field(
                        field.clone(),
                        self.create_literal_expr(identifier.clone(), identifier_type.clone()),
                        field_position.clone(),
                    )?;
                    arguments.extend(self.argument_list()?);
                    if let TType::Function {
                        parameters,
                        mut return_type,
                    } = left_expr.get_type()
                    {
                        if arguments.len() != parameters.len() {
                            let msg = "E3 Incorrect number of arguments";
                            return Err(self.generate_error_with_pos(
                                msg,
                                format!("Got {}, expected {}", arguments.len(), parameters.len()),
                                field_position,
                            ));
                        }
                        let input_types: Vec<TType> =
                            arguments.iter().map(|arg| arg.get_type()).collect();
                        let mut type_map = HashMap::default();
                        self.typechecker.check_and_map_types(
                            &input_types,
                            &parameters,
                            &mut type_map,
                            field_position.clone(),
                        )?;
                        return_type = Box::new(self.typechecker.get_output(
                            *return_type.clone(),
                            &mut type_map,
                            pos,
                        )?);
                        // dbg!(arguments.clone(), return_type.clone(), left_expr.clone());

                        Expr::Call {
                            ttype: *return_type,
                            name: field,
                            function: Box::new(left_expr),
                            args: arguments,
                        }
                    } else {
                        return Err(self.generate_error_with_pos(
                            format!("Cannot call {}", left_expr.get_type()),
                            "Not a function",
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
            Some(StructuralSymbol(LeftSquareBracket)) => {
                self.handle_indexing(identifier.clone(), pos.clone())?
            }
            Some(StructuralSymbol(LeftParen)) => self.call(identifier.clone(), pos, None)?,
            _ => {
                if self.current_token().is_some_and(|t| t.is_symbol(LeftBrace))
                    && self
                        .typechecker
                        .environment
                        .custom_types
                        .contains_key(&identifier)
                {
                    self.call(identifier.clone(), pos.clone(), None)?
                } else {
                    self.handle_literal_or_capture(identifier.clone(), pos.clone())?
                }
            }
        };

        Ok(anchor)
    }

    fn create_literal_expr(&self, identifier: Rc<str>, ttype: TType) -> Expr {
        Expr::Literal {
            ttype,
            value: Atom::Id { name: identifier },
        }
    }

    fn handle_indexing(&mut self, identifier: Rc<str>, position: FilePosition) -> NovaResult<Expr> {
        if let Some(ttype) = self.typechecker.environment.get_type(&identifier) {
            self.index(
                identifier.clone(),
                self.create_literal_expr(identifier.clone(), ttype.clone()),
                ttype.clone(),
            )
        } else if let Some((ttype, _, kind)) =
            self.typechecker.environment.get_type_capture(&identifier)
        {
            self.typechecker
                .environment
                .captured
                .last_mut()
                .unwrap()
                .insert(
                    identifier.clone(),
                    Symbol {
                        id: identifier.clone(),
                        ttype: ttype.clone(),
                        pos: Some(position.clone()),
                        kind: SymbolKind::Captured,
                    },
                );
            self.typechecker.environment.insert_symbol(
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
            Err(self.generate_error_with_pos(
                format!("E1 Not a valid symbol: {}", identifier),
                "Unknown identifier".to_string(),
                position,
            ))
        }
    }

    fn handle_literal_or_capture(
        &mut self,
        identifier: Rc<str>,
        position: FilePosition,
    ) -> NovaResult<Expr> {
        if let Some(ttype) = self.typechecker.environment.get_type(&identifier) {
            //println!("identifier hloc-not-capture {}", identifier);
            Ok(self.create_literal_expr(identifier.clone(), ttype.clone()))
        } else if let Some((ttype, _, kind)) =
            self.typechecker.environment.get_type_capture(&identifier)
        {
            // println!("identifier hloc-capture {}", identifier);
            // println!(
            //     "environment {:?}",
            //     self.typechecker.environment.captured.last().unwrap()
            // );
            self.typechecker
                .environment
                .captured
                .last_mut()
                .unwrap()
                .insert(
                    identifier.clone(),
                    Symbol {
                        id: identifier.clone(),
                        ttype: ttype.clone(),
                        pos: Some(position.clone()),
                        kind: SymbolKind::Captured,
                    },
                );
            self.typechecker.environment.insert_symbol(
                &identifier,
                ttype.clone(),
                Some(position.clone()),
                kind,
            );
            Ok(self.create_literal_expr(identifier.clone(), ttype.clone()))
        } else {
            Err(self.generate_error_with_pos(
                format!("E2 Not a valid symbol: {}", identifier),
                "Unknown identifier".to_string(),
                position,
            ))
        }
    }

    fn factor(&mut self) -> NovaResult<Expr> {
        let mut left: Expr;
        if let Ok(Some(sign)) = self.sign() {
            self.advance();
            let factor = self.factor()?;
            // make sure not sign only works on bools
            if sign == Unary::Not {
                if factor.get_type() != TType::Bool {
                    return Err(self.generate_error(
                        "Cannot use ! on non-boolean",
                        format!("Got {}", factor.get_type()),
                    ));
                } else {
                    return Ok(Expr::Unary {
                        ttype: TType::Bool,
                        expr: Box::new(factor),
                        op: sign,
                    });
                }
            } else {
                return Ok(Expr::Unary {
                    ttype: factor.get_type(),
                    expr: Box::new(factor),
                    op: sign,
                });
            }
        }
        match self.current_token_value() {
            Some(StructuralSymbol(LeftBrace)) => {
                left = self.block_expr()?;
            }
            // if expression if test {} else {}, both branches must return the same type
            Some(Identifier(id)) if "if" == id.deref() => {
                let pos = self.get_current_token_position();
                self.advance();

                let condition = self.expr()?;
                // condition must be a boolean
                if condition.get_type() != TType::Bool {
                    return Err(self.generate_error(
                        "Condition must be a boolean",
                        format!("Got {}", condition.get_type()),
                    ));
                }
                let if_branch = self.block_expr()?;
                self.consume_identifier(Some("else"))?;
                let else_branch = self.block_expr()?;
                let if_type = if if_branch.get_type() == else_branch.get_type() {
                    if_branch.get_type()
                } else {
                    return Err(self.generate_error_with_pos(
                        "Both branches must return the same type",
                        format!(
                            "Got {} and {}",
                            if_branch.get_type(),
                            else_branch.get_type()
                        ),
                        pos,
                    ));
                };
                left = Expr::IfExpr {
                    ttype: if_type,
                    test: Box::new(condition),
                    body: Box::new(if_branch),
                    alternative: Box::new(else_branch),
                };
            }
            Some(Identifier(id)) if "return" == id.deref() => {
                self.advance();
                let ret = self.expr()?;
                left = Expr::Return {
                    ttype: TType::Void,
                    expr: Box::new(ret),
                };
            }
            Some(Identifier(id)) if "None" == id.deref() => {
                self.advance();
                self.consume_symbol(LeftParen)?;
                let option_type = self.ttype()?;
                left = Expr::Literal {
                    ttype: TType::Option {
                        inner: Box::new(option_type),
                    },
                    value: Atom::None,
                };
                self.consume_symbol(RightParen)?;
            }
            Some(&Char(value)) => {
                self.advance();
                left = Expr::Literal {
                    ttype: TType::Char,
                    value: Atom::Char { value },
                }
            }
            Some(Identifier(id)) if "fn" == id.deref() => {
                let pos = self.get_current_token_position();
                self.advance();
                // get parameters
                self.consume_symbol(LeftParen)?;
                let parameters = self.parameter_list()?;
                self.consume_symbol(RightParen)?;
                // get output type
                let mut output = TType::Void;
                if self.current_token().is_some_and(|t| t.is_symbol(LeftBrace)) {
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
                for (ttype, identifier) in parameters.iter().cloned() {
                    if let TType::Function { .. } = ttype.clone() {
                        // check if generic function exist
                        if self.typechecker.environment.has(&identifier) {
                            return Err(self.generate_error_with_pos(
                                format!("Generic Function {} already defined", &identifier),
                                "Cannot redefine a generic function",
                                pos.clone(),
                            ));
                        }
                        // check if normal function exist
                        if self.typechecker.environment.has(&identifier) {
                            return Err(self.generate_error_with_pos(
                                format!("Function {} already defined", &identifier,),
                                "Cannot redefine a generic function",
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
                let mut generic_list = TypeChecker::collect_generics(&typeinput);
                generic_list.extend(TypeChecker::collect_generics(&[output.clone()]));
                if let Some(livemap) = self.typechecker.environment.live_generics.last_mut() {
                    for generic in generic_list.items.iter() {
                        // add generics to live map
                        if !livemap.has(generic) {
                            livemap.insert(generic.clone());
                        }
                    }
                }
                self.typechecker.environment.push_scope();

                // insert params into scope
                for (ttype, id) in parameters.iter() {
                    match ttype.clone() {
                        TType::Function {
                            parameters: paraminput,
                            return_type: output,
                        } => {
                            self.typechecker.environment.insert_symbol(
                                id,
                                TType::Function {
                                    parameters: paraminput.clone(),
                                    return_type: Box::new(*output.clone()),
                                },
                                Some(pos.clone()),
                                SymbolKind::Parameter,
                            );
                        }
                        _ => self.typechecker.environment.insert_symbol(
                            id,
                            ttype.clone(),
                            Some(pos.clone()),
                            SymbolKind::Parameter,
                        ),
                    };
                }

                let mut statements = self.block()?;

                let mut captured: Vec<_> = self
                    .typechecker
                    .environment
                    .captured
                    .last()
                    .unwrap()
                    .iter()
                    .map(|v| v.0.clone())
                    .collect();

                self.typechecker.environment.pop_scope();

                for c in captured.iter() {
                    if let Some(mc) = self.typechecker.environment.get_type_capture(&c.clone()) {
                        let pos = self.get_current_token_position();
                        self.typechecker
                            .environment
                            .captured
                            .last_mut()
                            .unwrap()
                            .insert(
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
                    .typechecker
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
                        captured.retain(|x| x != &name);
                    }
                }

                // for dc in captured.iter() {
                //     if let Some(v) = self.typechecker.environment.values.last().unwrap().get(dc) {
                //         if let SymbolKind::Captured = v.kind {
                //         } else {
                //             self.typechecker.environment.captured.last_mut().unwrap().remove(dc);
                //         }
                //     }
                // }

                // check return types

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
                let will_return =
                    self.typechecker
                        .will_return(&statements, output.clone(), pos.clone())?;
                //dbg!(will_return);
                if !will_return {
                    return Err(self.generate_error_with_pos(
                        "E2 Function must return a value",
                        "Last statement is not a return",
                        pos,
                    ));
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
            Some(StructuralSymbol(Pipe) | Operator(Operator::Or)) => {
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
            Some(StructuralSymbol(LeftSquareBracket)) => {
                let pos = self.get_current_token_position();

                // add list comprehension using the for keyword
                // if symbol is colon operator then it is a list comprehension
                match self.peek_offset_value(2) {
                    Some(Keyword(KeyWord::In)) => {
                        let mut loops = vec![];
                        self.consume_symbol(LeftSquareBracket)?;
                        // first get ident, then in keyword, then expr, and then any guards
                        let (ident, mut pos) = self.get_identifier()?;
                        self.consume_keyword(KeyWord::In)?;
                        let listexpr = self.expr()?;

                        if let TType::List { inner } = listexpr.get_type() {
                            self.typechecker.environment.insert_symbol(
                                &ident,
                                *inner.clone(),
                                Some(pos.clone()),
                                SymbolKind::Variable,
                            );
                        } else {
                            return Err(self.generate_error_with_pos(
                                "List comprehension must be a list",
                                format!("{} is not a list", listexpr.get_type()),
                                pos,
                            ));
                        }

                        loops.push((ident.clone(), listexpr.clone()));
                        // while comma is present, get ident, in keyword, expr
                        while self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
                            self.consume_symbol(Comma)?;
                            let (ident, _) = self.get_identifier()?;
                            self.consume_keyword(KeyWord::In)?;
                            let listexpr = self.expr()?;
                            // insert identifer into scope for typechecking
                            if let TType::List { inner } = listexpr.get_type() {
                                self.typechecker.environment.insert_symbol(
                                    &ident,
                                    *inner.clone(),
                                    Some(pos.clone()),
                                    SymbolKind::Variable,
                                );
                            } else {
                                return Err(self.generate_error_with_pos(
                                    "List comprehension must be a list",
                                    format!("{} is not a list", listexpr.get_type()),
                                    pos,
                                ));
                            }
                            loops.push((ident.clone(), listexpr.clone()));
                        }
                        self.consume_symbol(Pipe)?;

                        self.typechecker.environment.push_block();
                        let mut outexpr = vec![self.expr()?];
                        // continue parsing expr if there is a comma after the outexpr
                        if self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
                            self.advance();
                            outexpr.push(self.expr()?);
                        }
                        // typecheck taht outexpr is not void
                        if outexpr.last().unwrap().get_type() == TType::Void {
                            return Err(self.generate_error_with_pos(
                                "List comprehension must return a value",
                                "Return expression is Void",
                                pos,
                            ));
                        }

                        let mut guards = vec![];
                        // now grab list of guards seprerated by bar
                        while self.current_token().is_some_and(|t| t.is_symbol(Pipe)) {
                            pos = self.get_current_token_position();
                            self.consume_symbol(Pipe)?;
                            guards.push(self.expr()?);
                        }

                        // check that all the guard types are bool
                        for guard in guards.iter() {
                            if guard.get_type() != TType::Bool {
                                return Err(self.generate_error_with_pos(
                                    "Guard must be a boolean",
                                    format!("{} is not a boolean", guard.get_type()),
                                    pos,
                                ));
                            }
                        }
                        self.typechecker.environment.pop_block();
                        self.consume_symbol(RightSquareBracket)?;
                        // remove ident from scope
                        for (ident, _) in loops.iter() {
                            if let Some(v) = self.typechecker.environment.values.last_mut() {
                                _ = v.remove(ident);
                            }
                        }
                        left = Expr::ListCompConstructor {
                            ttype: TType::List {
                                inner: Box::new(outexpr.last().unwrap().get_type()),
                            },
                            loops,
                            expr: outexpr,
                            guards,
                            position: pos,
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
                                return Err(Box::new(NovaError::TypeError {
                                    msg: "List must contain same type".into(),
                                    expected: ttype.to_string().into(),
                                    found: elem.get_type().to_string().into(),
                                    position: pos,
                                }));
                            }
                        }

                        if self
                            .current_token()
                            .is_some_and(|t| t.is_op(Operator::Colon))
                        {
                            self.consume_operator(Operator::Colon)?;
                            ttype = self.ttype()?;
                            if !expr_list.is_empty() && ttype != expr_list[0].get_type() {
                                return Err(Box::new(NovaError::TypeError {
                                    msg: "List must contain same type".into(),
                                    expected: ttype.to_string().into(),
                                    found: expr_list[0].get_type().to_string().into(),
                                    position: pos,
                                }));
                            }
                        }

                        let generic_list = TypeChecker::collect_generics(&[ttype.clone()]);
                        for generic in generic_list.items {
                            if !self
                                .typechecker
                                .environment
                                .live_generics
                                .last()
                                .unwrap()
                                .has(&generic)
                            {
                                return Err(Box::new(NovaError::SimpleTypeError {
                                    msg: format!("Generic Type '{}' is not live", generic).into(),
                                    position: pos,
                                }));
                            }
                        }
                        if ttype == TType::None {
                            return Err(self.generate_error_with_pos(
                                "List must have a type",
                                "use `[]: type` to annotate an empty list",
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
            Some(StructuralSymbol(LeftParen)) => {
                self.consume_symbol(LeftParen)?;
                if self
                    .current_token()
                    .is_some_and(|t| t.is_symbol(RightParen))
                {
                    self.consume_symbol(RightParen)?;
                    // error tuple must contain at least two elements
                    return Err(self.generate_error(
                        "Tuple must contain at least one elements",
                        "Add more elements to the tuple",
                    ));
                } else {
                    let expr = self.expr()?;
                    if expr.get_type() == TType::None {
                        return Err(self.generate_error(
                            "Tuple must not contain None",
                            "Add a comma after the element",
                        ));
                    }
                    // check if expr is a tuple
                    if self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
                        let mut tuple = vec![expr];
                        while self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
                            self.advance();
                            // if (5,) single element tuple
                            if self
                                .current_token()
                                .is_some_and(|t| t.is_symbol(RightParen))
                            {
                                break;
                            }
                            let expr = self.expr()?;
                            if expr.get_type() == TType::None {
                                return Err(self.generate_error(
                                    "Tuple must not contain None",
                                    "Add a comma after the element",
                                ));
                            }
                            tuple.push(expr);
                        }
                        self.consume_symbol(RightParen)?;
                        let typelist: Vec<_> = tuple.iter().map(|e| e.get_type()).collect();
                        left = Expr::ListConstructor {
                            ttype: TType::Tuple { elements: typelist },
                            elements: tuple,
                        };
                    } else {
                        self.consume_symbol(RightParen)?;
                        left = expr;
                    }
                }
            }
            Some(Identifier(_)) => {
                let (mut identifier, pos) = self.get_identifier()?;
                identifier = match self.current_token_value() {
                    Some(Operator(Operator::DoubleColon))
                        if matches!(
                            identifier.as_ref(),
                            "Int" | "String" | "Float" | "Bool" | "List" | "Char" | "Option"
                        ) =>
                    {
                        self.advance();
                        let (name, _) = self.get_identifier()?;
                        format!("{}::{}", identifier, name).into()
                    }
                    Some(Operator(Operator::DoubleColon))
                        if self
                            .typechecker
                            .environment
                            .custom_types
                            .contains_key(&identifier) =>
                    {
                        self.advance();
                        let (name, _) = self.get_identifier()?;
                        format!("{}::{}", identifier, name).into()
                    }
                    Some(Operator(Operator::DoubleColon)) if self.modules.has(&identifier) => {
                        self.advance();
                        let (name, _) = self.get_identifier()?;
                        format!("{}::{}", identifier, name).into()
                    }
                    Some(Operator(Operator::DoubleColon)) => identifier,
                    Some(StructuralSymbol(At)) => {
                        self.consume_symbol(At)?;
                        self.consume_symbol(LeftParen)?;
                        let mut type_annotation = vec![];
                        // if ) then push none and return
                        if self
                            .current_token()
                            .is_some_and(|t| t.is_symbol(RightParen))
                        {
                            self.advance();
                            type_annotation.push(TType::None);
                        } else {
                            let ta = self.ttype()?;
                            type_annotation.push(ta);
                            while self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
                                self.advance();
                                let ta = self.ttype()?;
                                type_annotation.push(ta);
                            }
                            self.consume_symbol(RightParen)?;
                        }
                        generate_unique_string(&identifier, &type_annotation).into()
                    }
                    _ => identifier,
                };

                identifier = match self.current_token_value() {
                    Some(Operator(Operator::DoubleColon))
                        if matches!(
                            identifier.as_ref(),
                            "Int" | "String" | "Float" | "Bool" | "List" | "Char" | "Option"
                        ) =>
                    {
                        self.advance();
                        let (name, _) = self.get_identifier()?;
                        format!("{}::{}", identifier, name).into()
                    }
                    Some(Operator(Operator::DoubleColon))
                        if self
                            .typechecker
                            .environment
                            .custom_types
                            .contains_key(&identifier) =>
                    {
                        self.advance();
                        let (name, _) = self.get_identifier()?;
                        format!("{}::{}", identifier, name).into()
                    }
                    Some(Operator(Operator::DoubleColon)) if self.modules.has(&identifier) => {
                        self.advance();
                        let (name, _) = self.get_identifier()?;
                        format!("{}::{}", identifier, name).into()
                    }
                    Some(Operator(Operator::DoubleColon)) => identifier,
                    Some(StructuralSymbol(At)) => {
                        self.consume_symbol(At)?;
                        self.consume_symbol(LeftParen)?;
                        let mut type_annotation = vec![];
                        // if ) then push none and return
                        if self
                            .current_token()
                            .is_some_and(|t| t.is_symbol(RightParen))
                        {
                            self.advance();
                            type_annotation.push(TType::None);
                        } else {
                            let ta = self.ttype()?;
                            type_annotation.push(ta);
                            while self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
                                self.advance();
                                let ta = self.ttype()?;
                                type_annotation.push(ta);
                            }
                            self.consume_symbol(RightParen)?;
                        }
                        let t = generate_unique_string(&identifier, &type_annotation).into();
                        dbg!(&t);
                        t
                    }
                    _ => identifier,
                };

                let leftt = self.anchor(identifier.clone(), pos)?;
                left = leftt;

                // dbg!(self.current_token(), identifier.clone());
            }
            Some(&Integer(value)) => {
                self.advance();
                left = Expr::Literal {
                    ttype: TType::Int,
                    value: Atom::Integer { value },
                };
            }
            Some(&Float(value)) => {
                self.advance();
                left = Expr::Literal {
                    ttype: TType::Float,
                    value: Atom::Float { value },
                };
            }
            Some(StringLiteral(s)) => {
                left = Expr::Literal {
                    ttype: TType::String,
                    value: Atom::String { value: s.clone() },
                };
                self.advance();
            }
            Some(&Bool(b)) => {
                self.advance();
                left = Expr::Literal {
                    ttype: TType::Bool,
                    value: Atom::Bool { value: b },
                };
            }
            None => {
                return Err(self.generate_error("End of file error", "expected expression"));
            }
            _ => left = Expr::Void,
        }
        loop {
            match self.current_token_value() {
                Some(Operator(Operator::RightArrow)) => {
                    self.consume_operator(Operator::RightArrow)?;
                    left = self.handle_inner_function_call(left)?;
                }
                Some(Operator(Operator::DoubleColon)) => {
                    self.consume_operator(Operator::DoubleColon)?;
                    left = self.handle_field_access(left)?;
                }
                Some(StructuralSymbol(Dot)) => {
                    self.consume_symbol(Dot)?;
                    left = self.handle_method_chain(left)?;
                }
                Some(StructuralSymbol(LeftParen)) => {
                    left = self.handle_function_pointer_call(left)?;
                }
                Some(StructuralSymbol(LeftSquareBracket)) => {
                    left = self.handle_chain_indexint(left)?;
                }
                Some(Operator(Operator::PipeArrow)) => {
                    self.consume_operator(Operator::PipeArrow)?;
                    let (mut identifier, pos) = self.get_identifier()?;
                    identifier = match self.current_token_value() {
                        Some(Operator(Operator::DoubleColon))
                            if matches!(
                                identifier.as_ref(),
                                "Int" | "String" | "Float" | "Bool" | "List" | "Char" | "Option"
                            ) =>
                        {
                            self.advance();
                            let (name, _) = self.get_identifier()?;
                            format!("{}::{}", identifier, name).into()
                        }
                        Some(Operator(Operator::DoubleColon))
                            if self
                                .typechecker
                                .environment
                                .custom_types
                                .contains_key(&identifier) =>
                        {
                            self.advance();
                            let (name, _) = self.get_identifier()?;
                            format!("{}::{}", identifier, name).into()
                        }
                        Some(Operator(Operator::DoubleColon)) if self.modules.has(&identifier) => {
                            self.advance();
                            let (name, _) = self.get_identifier()?;
                            format!("{}::{}", identifier, name).into()
                        }
                        Some(Operator(Operator::DoubleColon)) => identifier,
                        Some(StructuralSymbol(At)) => {
                            self.consume_symbol(At)?;
                            self.consume_symbol(LeftParen)?;
                            let mut type_annotation = vec![];
                            let ta = self.ttype()?;
                            type_annotation.push(ta);
                            while self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
                                self.advance();
                                let ta = self.ttype()?;
                                type_annotation.push(ta);
                            }
                            self.consume_symbol(RightParen)?;
                            generate_unique_string(&identifier, &type_annotation).into()
                        }
                        _ => identifier,
                    };
                    left = self.call(identifier, pos, Some(left))?;
                }
                _ => {
                    break;
                }
            }
        }

        Ok(left)
    }

    #[allow(clippy::type_complexity)]
    fn bar_closure(
        &mut self,
    ) -> NovaResult<(Vec<TType>, Vec<Arg>, TType, Vec<Statement>, Vec<Rc<str>>)> {
        let pos = self.get_current_token_position();
        let parameters = match self.consume_symbol(Pipe) {
            Ok(_) => {
                let p = self.parameter_list()?;
                self.consume_symbol(Pipe)?;
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
                if self.typechecker.environment.has(&identifier) {
                    return Err(self.generate_error_with_pos(
                        format!("Generic Function {} already defined", &identifier),
                        "Cannot redefine a generic function",
                        pos.clone(),
                    ));
                }
                // check if normal function exist
                if self.typechecker.environment.has(&identifier) {
                    return Err(self.generate_error_with_pos(
                        format!("Function {} already defined", &identifier,),
                        "Cannot redefine a generic function",
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
        let generic_list = TypeChecker::collect_generics(&typeinput);
        self.typechecker
            .environment
            .live_generics
            .push(generic_list.clone());
        self.typechecker.environment.push_scope();
        for (ttype, id) in parameters.iter() {
            match ttype.clone() {
                TType::Function {
                    parameters: paraminput,
                    return_type: output,
                } => {
                    self.typechecker.environment.insert_symbol(
                        id,
                        TType::Function {
                            parameters: paraminput.clone(),
                            return_type: Box::new(*output.clone()),
                        },
                        Some(pos.clone()),
                        SymbolKind::Parameter,
                    );
                }
                _ => self.typechecker.environment.insert_symbol(
                    id,
                    ttype.clone(),
                    Some(pos.clone()),
                    SymbolKind::Parameter,
                ),
            };
        }
        let output: TType;
        let statement = if let Some(StructuralSymbol(LeftBrace)) = self.current_token_value() {
            //println!("its a block");
            let expression = self.block_expr()?;
            output = expression.clone().get_type();
            let statement = vec![Statement::Return {
                ttype: expression.get_type(),
                expr: expression.clone(),
            }];
            statement
        } else {
            //println!("its an expression");
            let expression = self.expr()?;
            output = expression.clone().get_type();
            let statement = vec![Statement::Return {
                ttype: expression.get_type(),
                expr: expression.clone(),
            }];
            statement
        };
        let mut captured: Vec<_> = self
            .typechecker
            .environment
            .captured
            .last()
            .unwrap()
            .iter()
            .map(|v| v.0.clone())
            .collect();

        self.typechecker.environment.pop_scope();
        self.typechecker.environment.live_generics.pop();
        for c in captured.iter() {
            if let Some(mc) = self.typechecker.environment.get_type_capture(&c.clone()) {
                let pos = self.get_current_token_position();

                self.typechecker
                    .environment
                    .captured
                    .last_mut()
                    .unwrap()
                    .insert(
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
            .typechecker
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
                captured.retain(|x| x != &name);
            }
        }

        // for dc in captured.iter() {
        //     if let Some(v) = self.typechecker.environment.values.last().unwrap().get(dc) {
        //         if let SymbolKind::Captured = v.kind {
        //         } else {
        //             self.typechecker.environment.captured.last_mut().unwrap().remove(dc);
        //         }
        //     }
        // }

        Ok((typeinput, input, output, statement, captured))
    }

    fn handle_inner_function_call(&mut self, left: Expr) -> NovaResult<Expr> {
        let (target_field, pos) = self.get_identifier()?;
        let mut arguments = vec![left.clone()];
        let function_expr = self.field(target_field.clone(), left.clone(), pos.clone())?;
        arguments.extend(self.argument_list()?);
        self.create_call_expression(function_expr, target_field, arguments, pos)
    }

    fn handle_field_access(&mut self, left: Expr) -> NovaResult<Expr> {
        let (field, pos) = self.get_identifier()?;
        self.field(field.clone(), left, pos)
    }

    fn handle_method_chain(&mut self, left: Expr) -> NovaResult<Expr> {
        self.chain(left)
    }

    fn handle_function_pointer_call(&mut self, left: Expr) -> NovaResult<Expr> {
        let pos = self.get_current_token_position();
        let mut arguments = self.argument_list()?;
        if arguments.is_empty() {
            arguments.push(Expr::None)
        }
        self.create_call_expression(left, "anon".into(), arguments, pos)
    }

    fn handle_chain_indexint(&mut self, left: Expr) -> NovaResult<Expr> {
        self.index("anon".into(), left.clone(), left.get_type().clone())
    }

    fn create_call_expression(
        &mut self,
        function_expr: Expr,
        function_name: Rc<str>,
        arguments: Vec<Expr>,
        pos: FilePosition,
    ) -> NovaResult<Expr> {
        if let TType::Function {
            parameters,
            mut return_type,
        } = function_expr.get_type()
        {
            if arguments.len() != parameters.len() {
                return Err(self.generate_error_with_pos(
                    "Incorrect number of arguments",
                    format!("Got {}, expected {}", arguments.len(), parameters.len()),
                    pos.clone(),
                ));
            }
            let mut input_types = vec![];
            for arg in arguments.iter() {
                input_types.push(arg.get_type())
            }
            let mut type_map = HashMap::new();
            self.typechecker.check_and_map_types(
                &parameters,
                &input_types,
                &mut type_map,
                pos.clone(),
            )?;
            return_type = Box::new(self.typechecker.get_output(
                *return_type.clone(),
                &mut type_map,
                pos,
            )?);
            Ok(Expr::Call {
                ttype: *return_type,
                name: function_name,
                function: Box::new(function_expr),
                args: arguments,
            })
        } else {
            Err(self.generate_error_with_pos(
                format!("Cannot call {}", function_expr.get_type()),
                "Not a function",
                pos.clone(),
            ))
        }
    }

    fn term(&mut self) -> NovaResult<Expr> {
        let mut left_expr = self.factor()?;
        let current_pos = self.get_current_token_position();
        while self.current_token().is_some_and(|t| t.is_multi_op()) {
            if let Some(operation) = self.current_token().and_then(|t| t.get_operator()) {
                self.advance();
                let right_expr = self.factor()?;
                match (left_expr.clone().get_type(), right_expr.clone().get_type()) {
                    (TType::Int, TType::Int) | (TType::Float, TType::Float) => {
                        // if module only works with int
                        if operation == Operator::Modulo {
                            match (left_expr.clone().get_type(), right_expr.clone().get_type()) {
                                (TType::Int, TType::Int) => {}
                                (_, _) => {
                                    return Err(self.typechecker.create_type_error(
                                        left_expr.clone(),
                                        right_expr.clone(),
                                        operation,
                                        current_pos.clone(),
                                    ));
                                }
                            }
                        }
                        left_expr = self.create_binop_expr(
                            left_expr.clone(),
                            right_expr,
                            operation,
                            left_expr.get_type(),
                        );
                    }
                    (_, _) => {
                        // check dunder methods for operation
                        let function_id: String = match operation {
                            Operator::Multiplication => {
                                if let Some(custom) = left_expr.get_type().custom_to_string() {
                                    format!("{}::__mul__", custom)
                                } else {
                                    // error if no custom method
                                    return Err(self.typechecker.create_type_error(
                                        left_expr.clone(),
                                        right_expr.clone(),
                                        operation,
                                        current_pos.clone(),
                                    ));
                                }
                            }
                            Operator::Division => {
                                if let Some(custom) = left_expr.get_type().custom_to_string() {
                                    format!("{}::__div__", custom)
                                } else {
                                    // error if no custom method
                                    return Err(self.typechecker.create_type_error(
                                        left_expr.clone(),
                                        right_expr.clone(),
                                        operation,
                                        current_pos.clone(),
                                    ));
                                }
                            }
                            Operator::Modulo => {
                                if let Some(custom) = left_expr.get_type().custom_to_string() {
                                    format!("{}::__mod__", custom)
                                } else {
                                    // error if no custom method
                                    return Err(self.typechecker.create_type_error(
                                        left_expr.clone(),
                                        right_expr.clone(),
                                        operation,
                                        current_pos.clone(),
                                    ));
                                }
                            }
                            _ => {
                                return Err(self.generate_error_with_pos(
                                    "Invalid operation",
                                    "Operation not supported",
                                    current_pos.clone(),
                                ));
                            }
                        };
                        if let Some(overload) =
                            self.typechecker.environment.get(&generate_unique_string(
                                &function_id,
                                &[left_expr.get_type(), right_expr.get_type()],
                            ))
                        {
                            // get return type of function call
                            let pos = self.get_current_token_position();
                            let arguments = vec![left_expr.clone(), right_expr.clone()];
                            let typelist = vec![left_expr.get_type(), right_expr.get_type()];
                            let returntype = match overload.ttype {
                                TType::Function {
                                    return_type,
                                    parameters,
                                } => {
                                    match self.typechecker.check_and_map_types(
                                        &parameters,
                                        &typelist,
                                        &mut HashMap::default(),
                                        pos.clone(),
                                    ) {
                                        Ok(_) => *return_type,
                                        Err(_) => {
                                            match (
                                                left_expr.clone().get_type(),
                                                right_expr.clone().get_type(),
                                            ) {
                                                (TType::Int, TType::Int)
                                                | (TType::Float, TType::Float) => {
                                                    left_expr = self.create_binop_expr(
                                                        left_expr.clone(),
                                                        right_expr,
                                                        operation,
                                                        left_expr.get_type(),
                                                    );
                                                }
                                                (_, _) => {
                                                    return Err(self
                                                        .typechecker
                                                        .create_type_error(
                                                            left_expr.clone(),
                                                            right_expr.clone(),
                                                            operation,
                                                            current_pos.clone(),
                                                        ));
                                                }
                                            }
                                            return Ok(left_expr);
                                        }
                                    }
                                }
                                _ => {
                                    return Err(self.generate_error(
                                        "Expected function",
                                        "Make sure function is defined",
                                    ))
                                }
                            };
                            // return function call expression
                            left_expr = Expr::Literal {
                                ttype: returntype,
                                value: Atom::Call {
                                    name: generate_unique_string(
                                        &function_id,
                                        &[left_expr.get_type(), right_expr.get_type()],
                                    )
                                    .into(),
                                    arguments,
                                    position: pos.clone(),
                                },
                            };
                        } else if let Some(overload) =
                            self.typechecker.environment.get(&function_id)
                        {
                            // get return type of function call
                            let pos = self.get_current_token_position();
                            let arguments = vec![left_expr.clone(), right_expr.clone()];
                            let typelist =
                                vec![left_expr.clone().get_type(), right_expr.get_type()];
                            let returntype = match overload.ttype {
                                TType::Function {
                                    return_type,
                                    parameters,
                                } => {
                                    match self.typechecker.check_and_map_types(
                                        &parameters,
                                        &typelist,
                                        &mut HashMap::default(),
                                        pos.clone(),
                                    ) {
                                        Ok(_) => *return_type,
                                        Err(_) => {
                                            match (
                                                left_expr.clone().get_type(),
                                                right_expr.clone().get_type(),
                                            ) {
                                                (TType::Int, TType::Int)
                                                | (TType::Float, TType::Float) => {
                                                    left_expr = self.create_binop_expr(
                                                        left_expr.clone(),
                                                        right_expr,
                                                        operation,
                                                        left_expr.get_type(),
                                                    );
                                                }
                                                (_, _) => {
                                                    return Err(self
                                                        .typechecker
                                                        .create_type_error(
                                                            left_expr.clone(),
                                                            right_expr.clone(),
                                                            operation,
                                                            current_pos.clone(),
                                                        ));
                                                }
                                            }
                                            return Ok(left_expr);
                                        }
                                    }
                                }
                                _ => {
                                    return Err(self.generate_error(
                                        "Expected function",
                                        "Make sure function is defined",
                                    ))
                                }
                            };
                            // return function call expression
                            left_expr = Expr::Literal {
                                ttype: returntype,
                                value: Atom::Call {
                                    name: function_id.into(),
                                    arguments,
                                    position: pos.clone(),
                                },
                            };
                        } else {
                            match (left_expr.clone().get_type(), right_expr.clone().get_type()) {
                                (TType::Int, TType::Int) | (TType::Float, TType::Float) => {
                                    left_expr = self.create_binop_expr(
                                        left_expr.clone(),
                                        right_expr,
                                        operation,
                                        left_expr.get_type(),
                                    );
                                }
                                (_, _) => {
                                    return Err(self.typechecker.create_type_error(
                                        left_expr.clone(),
                                        right_expr.clone(),
                                        operation,
                                        current_pos.clone(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(left_expr)
    }

    fn expr(&mut self) -> NovaResult<Expr> {
        match self.current_token_value() {
            Some(Identifier(id)) if "let" == id.deref() => {
                return self.let_expr();
            }
            _ => {}
        }
        let mut left_expr = self.logical_top_expr()?;
        let current_pos = self.get_current_token_position();
        while self.current_token().is_some_and(|t| t.is_assign()) {
            if let Some(operation) = self.current_token().and_then(|t| t.get_operator()) {
                self.advance();
                let right_expr = self.logical_top_expr()?;
                match left_expr.clone() {
                    Expr::ListConstructor { .. }
                    | Expr::Binop { .. }
                    | Expr::Call { .. }
                    | Expr::Unary { .. }
                    | Expr::Closure { .. }
                    | Expr::None => {
                        return Err(self.generate_error_with_pos(
                            "Error: left hand side of `=` must be assignable",
                            format!("{:?} is not assignable", left_expr),
                            current_pos.clone(),
                        ));
                    }
                    Expr::Literal { value: v, .. } => match v {
                        Atom::Id { .. } => {
                            self.typechecker.check_and_map_types(
                                &[left_expr.get_type()],
                                &[right_expr.get_type()],
                                &mut HashMap::default(),
                                current_pos.clone(),
                            )?;
                        }
                        _ => {
                            return Err(self.generate_error_with_pos(
                                format!(
                                    "cannot assign {} to {}",
                                    right_expr.get_type(),
                                    left_expr.get_type()
                                ),
                                "Cannot assign a value to a literal value",
                                current_pos.clone(),
                            ));
                        }
                    },
                    _ => {
                        if right_expr.get_type() != left_expr.get_type() {
                            return Err(self.generate_error_with_pos(
                                format!(
                                    "cannot assign {} to {}",
                                    right_expr.get_type(),
                                    left_expr.get_type()
                                ),
                                "Cannot assign a value to a literal value",
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

        if let Some(Operator(Operator::RightTilde)) = self.current_token_value() {
            // the syntax is expr ~> id { statements }
            self.consume_operator(Operator::RightTilde)?;
            let (identifier, pos) = self.get_identifier()?;

            // if current token is { else its expr,
            match self.current_token_value() {
                Some(StructuralSymbol(LeftBrace)) => {
                    // cant assing a void
                    if left_expr.get_type() == TType::Void {
                        return Err(self.generate_error_with_pos(
                            format!("Variable '{}' cannot be assinged to void", identifier),
                            "Make sure the expression returns a value",
                            pos.clone(),
                        ));
                    }

                    if self.typechecker.environment.has(&identifier) {
                        return Err(self.generate_error_with_pos(
                            format!("Variable '{}' has already been created", identifier),
                            "",
                            pos.clone(),
                        ));
                    } else {
                        self.typechecker.environment.push_block();
                        self.typechecker.environment.insert_symbol(
                            &identifier,
                            left_expr.get_type(),
                            Some(pos.clone()),
                            SymbolKind::Variable,
                        );
                        let expr_block = self.block()?;
                        self.typechecker.environment.pop_block();

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
                        "Expected block after `~>`",
                        "Make sure to use a block after `~>`",
                        pos.clone(),
                    ));
                }
            }
        }
        Ok(left_expr)
    }

    fn top_expr(&mut self) -> NovaResult<Expr> {
        let mut left_expr = self.mid_expr()?;
        let current_pos = self.get_current_token_position();
        while self.current_token().is_some_and(|t| t.is_relop()) {
            if let Some(operation) = self.current_token().and_then(|t| t.get_operator()) {
                self.advance();
                let right_expr = self.mid_expr()?;
                // check if void
                if left_expr.get_type() == TType::Void || right_expr.get_type() == TType::Void {
                    return Err(self.generate_error_with_pos(
                        "Cannot compare void",
                        "Make sure expression returns a value",
                        current_pos.clone(),
                    ));
                }
                match operation {
                    Operator::Greater
                    | Operator::GreaterOrEqual
                    | Operator::LessOrEqual
                    | Operator::Less => {
                        match (left_expr.get_type(), right_expr.get_type()) {
                            (TType::Int, TType::Int) => {
                                left_expr = self.create_binop_expr(
                                    left_expr,
                                    right_expr,
                                    operation,
                                    TType::Bool,
                                );
                            }
                            (TType::Float, TType::Float) => {
                                left_expr = self.create_binop_expr(
                                    left_expr,
                                    right_expr,
                                    operation,
                                    TType::Bool,
                                );
                            }
                            _ => {
                                // check dunder method
                                let function_id: String = match operation {
                                    Operator::Greater => {
                                        if let Some(custom) =
                                            left_expr.get_type().custom_to_string()
                                        {
                                            format!("{}::__gt__", custom)
                                        } else {
                                            return Err(self.typechecker.create_type_error(
                                                left_expr.clone(),
                                                right_expr.clone(),
                                                operation,
                                                current_pos.clone(),
                                            ));
                                        }
                                    }
                                    Operator::GreaterOrEqual => {
                                        if let Some(custom) =
                                            left_expr.get_type().custom_to_string()
                                        {
                                            format!("{}::__ge__", custom)
                                        } else {
                                            return Err(self.typechecker.create_type_error(
                                                left_expr.clone(),
                                                right_expr.clone(),
                                                operation,
                                                current_pos.clone(),
                                            ));
                                        }
                                    }
                                    Operator::Less => {
                                        if let Some(custom) =
                                            left_expr.get_type().custom_to_string()
                                        {
                                            format!("{}::__lt__", custom)
                                        } else {
                                            return Err(self.typechecker.create_type_error(
                                                left_expr.clone(),
                                                right_expr.clone(),
                                                operation,
                                                current_pos.clone(),
                                            ));
                                        }
                                    }
                                    Operator::LessOrEqual => {
                                        if let Some(custom) =
                                            left_expr.get_type().custom_to_string()
                                        {
                                            format!("{}::__le__", custom)
                                        } else {
                                            return Err(self.typechecker.create_type_error(
                                                left_expr.clone(),
                                                right_expr.clone(),
                                                operation,
                                                current_pos.clone(),
                                            ));
                                        }
                                    }
                                    _ => {
                                        return Err(self.generate_error(
                                            "Expected function",
                                            "Make sure function is defined",
                                        ))
                                    }
                                };

                                if let Some(overload) =
                                    self.typechecker.environment.get(&generate_unique_string(
                                        &function_id,
                                        &[left_expr.get_type(), right_expr.get_type()],
                                    ))
                                {
                                    // get return type of function call
                                    let pos = self.get_current_token_position();
                                    let arguments = vec![left_expr.clone(), right_expr.clone()];
                                    let typelist =
                                        vec![left_expr.get_type(), right_expr.get_type()];
                                    let returntype = match overload.ttype {
                                        TType::Function {
                                            return_type,
                                            parameters,
                                        } => {
                                            match self.typechecker.check_and_map_types(
                                                &parameters,
                                                &typelist,
                                                &mut HashMap::default(),
                                                pos.clone(),
                                            ) {
                                                Ok(_) => *return_type,
                                                Err(_) => {
                                                    return Ok(self.create_binop_expr(
                                                        left_expr,
                                                        right_expr,
                                                        operation,
                                                        TType::Bool,
                                                    ))
                                                }
                                            }
                                        }
                                        _ => {
                                            return Err(self.generate_error(
                                                "Expected function",
                                                "Make sure function is defined",
                                            ))
                                        }
                                    };
                                    // check if return type is bool
                                    if returntype != TType::Bool {
                                        return Err(self.generate_error_with_pos(
                                            "Comparison operation expects bool",
                                            format!(
                                                "expected {} , but found {}",
                                                left_expr.get_type(),
                                                right_expr.get_type(),
                                            ),
                                            current_pos.clone(),
                                        ));
                                    }
                                    // return function call expression
                                    left_expr = Expr::Literal {
                                        ttype: TType::Bool,
                                        value: Atom::Call {
                                            name: generate_unique_string(
                                                &function_id,
                                                &[left_expr.get_type(), right_expr.get_type()],
                                            )
                                            .into(),
                                            arguments,
                                            position: pos.clone(),
                                        },
                                    };
                                } else if let Some(overload) =
                                    self.typechecker.environment.get(&function_id)
                                {
                                    // get return type of function call
                                    let pos = self.get_current_token_position();
                                    let arguments = vec![left_expr.clone(), right_expr.clone()];
                                    let typelist =
                                        vec![left_expr.get_type(), right_expr.get_type()];
                                    let returntype = match overload.ttype {
                                        TType::Function {
                                            return_type,
                                            parameters,
                                        } => {
                                            match self.typechecker.check_and_map_types(
                                                &parameters,
                                                &typelist,
                                                &mut HashMap::default(),
                                                pos.clone(),
                                            ) {
                                                Ok(_) => *return_type,
                                                Err(_) => {
                                                    return Ok(self.create_binop_expr(
                                                        left_expr,
                                                        right_expr,
                                                        operation,
                                                        TType::Bool,
                                                    ))
                                                }
                                            }
                                        }
                                        _ => {
                                            return Err(self.generate_error(
                                                "Expected function",
                                                "Make sure function is defined",
                                            ))
                                        }
                                    };
                                    // check if return type is bool
                                    if returntype != TType::Bool {
                                        return Err(self.generate_error_with_pos(
                                            "Comparison operation expects bool",
                                            format!(
                                                "expected {} , but found {}",
                                                left_expr.get_type(),
                                                right_expr.get_type(),
                                            ),
                                            current_pos.clone(),
                                        ));
                                    }
                                    // return function call expression
                                    left_expr = Expr::Literal {
                                        ttype: TType::Bool,
                                        value: Atom::Call {
                                            name: function_id.into(),
                                            arguments,
                                            position: pos.clone(),
                                        },
                                    };
                                } else {
                                    // return error
                                    return Err(self.typechecker.create_type_error(
                                        left_expr.clone(),
                                        right_expr.clone(),
                                        operation,
                                        current_pos.clone(),
                                    ));
                                }
                            }
                        }
                    }
                    _ => {
                        // check dunder method
                        let function_id: String = match operation {
                            Operator::Equal => {
                                if let Some(custom) = left_expr.get_type().custom_to_string() {
                                    format!("{}::__eq__", custom)
                                } else {
                                    left_expr = self.create_binop_expr(
                                        left_expr,
                                        right_expr,
                                        operation,
                                        TType::Bool,
                                    );
                                    return Ok(left_expr);
                                }
                            }
                            Operator::NotEqual => {
                                if let Some(custom) = left_expr.get_type().custom_to_string() {
                                    format!("{}::__ne__", custom)
                                } else {
                                    left_expr = self.create_binop_expr(
                                        left_expr,
                                        right_expr,
                                        operation,
                                        TType::Bool,
                                    );
                                    return Ok(left_expr);
                                }
                            }
                            _ => "".into(),
                        };
                        if let Some(overload) =
                            self.typechecker.environment.get(&generate_unique_string(
                                &function_id,
                                &[left_expr.get_type(), right_expr.get_type()],
                            ))
                        {
                            // get return type of function call
                            let pos = self.get_current_token_position();
                            let arguments = vec![left_expr.clone(), right_expr.clone()];
                            let typelist = vec![left_expr.get_type(), right_expr.get_type()];
                            let returntype = match overload.ttype {
                                TType::Function {
                                    return_type,
                                    parameters,
                                } => {
                                    match self.typechecker.check_and_map_types(
                                        &parameters,
                                        &typelist,
                                        &mut HashMap::default(),
                                        pos.clone(),
                                    ) {
                                        Ok(_) => *return_type,
                                        Err(_) => {
                                            return Ok(self.create_binop_expr(
                                                left_expr,
                                                right_expr,
                                                operation,
                                                TType::Bool,
                                            ))
                                        }
                                    }
                                }
                                _ => {
                                    return Err(self.generate_error(
                                        "Expected function",
                                        "Make sure function is defined",
                                    ))
                                }
                            };
                            // check if return type is bool
                            if returntype != TType::Bool {
                                return Err(self.generate_error_with_pos(
                                    "Comparison operation expects bool",
                                    format!(
                                        "expected {} , but found {}",
                                        left_expr.get_type(),
                                        right_expr.get_type(),
                                    ),
                                    current_pos.clone(),
                                ));
                            }
                            // return function call expression
                            left_expr = Expr::Literal {
                                ttype: TType::Bool,
                                value: Atom::Call {
                                    name: generate_unique_string(
                                        &function_id,
                                        &[left_expr.get_type(), right_expr.get_type()],
                                    )
                                    .into(),
                                    arguments,
                                    position: pos.clone(),
                                },
                            };
                        } else if let Some(overload) =
                            self.typechecker.environment.get(&function_id)
                        {
                            // get return type of function call
                            let pos = self.get_current_token_position();
                            let arguments = vec![left_expr.clone(), right_expr.clone()];
                            let typelist = vec![left_expr.get_type(), right_expr.get_type()];
                            let returntype = match overload.ttype {
                                TType::Function {
                                    return_type,
                                    parameters,
                                } => {
                                    match self.typechecker.check_and_map_types(
                                        &parameters,
                                        &typelist,
                                        &mut HashMap::default(),
                                        pos.clone(),
                                    ) {
                                        Ok(_) => *return_type,
                                        Err(_) => {
                                            return Ok(self.create_binop_expr(
                                                left_expr,
                                                right_expr,
                                                operation,
                                                TType::Bool,
                                            ))
                                        }
                                    }
                                }
                                _ => {
                                    return Err(self.generate_error(
                                        "Expected function",
                                        "Make sure function is defined",
                                    ))
                                }
                            };
                            // check if return type is bool
                            if returntype != TType::Bool {
                                return Err(self.generate_error_with_pos(
                                    "Comparison operation expects bool",
                                    format!(
                                        "expected {} , but found {}",
                                        left_expr.get_type(),
                                        right_expr.get_type(),
                                    ),
                                    current_pos.clone(),
                                ));
                            }
                            // return function call expression
                            left_expr = Expr::Literal {
                                ttype: TType::Bool,
                                value: Atom::Call {
                                    name: function_id.into(),
                                    arguments,
                                    position: pos.clone(),
                                },
                            };
                        } else {
                            left_expr = self.create_binop_expr(
                                left_expr,
                                right_expr,
                                operation,
                                TType::Bool,
                            );
                        }
                    }
                }
            }
        }
        Ok(left_expr)
    }

    fn logical_top_expr(&mut self) -> NovaResult<Expr> {
        let mut left_expr = self.top_expr()?;
        let current_pos = self.get_current_token_position();
        while self.current_token().is_some_and(|t| t.is_logical_op()) {
            if let Some(operation) = self.current_token().and_then(|t| t.get_operator()) {
                self.advance();
                let right_expr = self.top_expr()?;
                // check if void
                if left_expr.get_type() == TType::Void || right_expr.get_type() == TType::Void {
                    return Err(self.generate_error_with_pos(
                        "Cannot compare void",
                        "Make sure expression returns a value",
                        current_pos.clone(),
                    ));
                }
                match operation {
                    Operator::And | Operator::Or => {
                        if (left_expr.get_type() != TType::Bool)
                            || (right_expr.get_type() != TType::Bool)
                        {
                            // check dunder method
                            let function_id: String = match operation {
                                Operator::And => {
                                    if let Some(custom) = left_expr.get_type().custom_to_string() {
                                        format!("{}::__and__", custom)
                                    } else {
                                        // error if no custom method
                                        return Err(self.typechecker.create_type_error(
                                            left_expr.clone(),
                                            right_expr.clone(),
                                            operation,
                                            current_pos.clone(),
                                        ));
                                    }
                                }
                                Operator::Or => {
                                    if let Some(custom) = left_expr.get_type().custom_to_string() {
                                        format!("{}::__or__", custom)
                                    } else {
                                        // error if no custom method
                                        return Err(self.typechecker.create_type_error(
                                            left_expr.clone(),
                                            right_expr.clone(),
                                            operation,
                                            current_pos.clone(),
                                        ));
                                    }
                                }
                                _ => {
                                    return Err(self.generate_error(
                                        "Expected function",
                                        "Make sure function is defined",
                                    ))
                                }
                            };

                            if let Some(overload) =
                                self.typechecker.environment.get(&generate_unique_string(
                                    &function_id,
                                    &[left_expr.get_type(), right_expr.get_type()],
                                ))
                            {
                                // get return type of function call
                                let pos = self.get_current_token_position();
                                let arguments = vec![left_expr.clone(), right_expr.clone()];
                                let typelist = vec![left_expr.get_type(), right_expr.get_type()];
                                let returntype = match overload.ttype {
                                    TType::Function {
                                        return_type,
                                        parameters,
                                    } => {
                                        match self.typechecker.check_and_map_types(
                                            &parameters,
                                            &typelist,
                                            &mut HashMap::default(),
                                            pos.clone(),
                                        ) {
                                            Ok(_) => *return_type,
                                            Err(_) => {
                                                return Ok(self.create_binop_expr(
                                                    left_expr,
                                                    right_expr,
                                                    operation,
                                                    TType::Bool,
                                                ))
                                            }
                                        }
                                    }
                                    _ => {
                                        return Err(self.generate_error(
                                            "Expected function",
                                            "Make sure function is defined",
                                        ))
                                    }
                                };
                                // check if return type is bool
                                if returntype != TType::Bool {
                                    return Err(self.generate_error_with_pos(
                                        "Comparison operation expects bool",
                                        format!(
                                            "expected {} , but found {}",
                                            left_expr.get_type(),
                                            right_expr.get_type(),
                                        ),
                                        current_pos.clone(),
                                    ));
                                }
                                // return function call expression
                                left_expr = Expr::Literal {
                                    ttype: TType::Bool,
                                    value: Atom::Call {
                                        name: generate_unique_string(
                                            &function_id,
                                            &[left_expr.get_type(), right_expr.get_type()],
                                        )
                                        .into(),
                                        arguments,
                                        position: pos.clone(),
                                    },
                                };
                            } else if let Some(overload) =
                                self.typechecker.environment.get(&function_id)
                            {
                                // get return type of function call
                                let pos = self.get_current_token_position();
                                let arguments = vec![left_expr.clone(), right_expr.clone()];
                                let typelist = vec![left_expr.get_type(), right_expr.get_type()];
                                let returntype = match overload.ttype {
                                    TType::Function {
                                        return_type,
                                        parameters,
                                    } => {
                                        match self.typechecker.check_and_map_types(
                                            &parameters,
                                            &typelist,
                                            &mut HashMap::default(),
                                            pos.clone(),
                                        ) {
                                            Ok(_) => *return_type,
                                            Err(_) => {
                                                return Ok(self.create_binop_expr(
                                                    left_expr,
                                                    right_expr,
                                                    operation,
                                                    TType::Bool,
                                                ))
                                            }
                                        }
                                    }
                                    _ => {
                                        return Err(self.generate_error(
                                            "Expected function",
                                            "Make sure function is defined",
                                        ))
                                    }
                                };
                                // check if return type is bool
                                if returntype != TType::Bool {
                                    return Err(self.generate_error_with_pos(
                                        "Comparison operation expects bool",
                                        format!(
                                            "expected {} , but found {}",
                                            left_expr.get_type(),
                                            right_expr.get_type(),
                                        ),
                                        current_pos.clone(),
                                    ));
                                }
                                // return function call expression
                                left_expr = Expr::Literal {
                                    ttype: TType::Bool,
                                    value: Atom::Call {
                                        name: function_id.into(),
                                        arguments,
                                        position: pos.clone(),
                                    },
                                };
                            } else {
                                left_expr = self.create_binop_expr(
                                    left_expr,
                                    right_expr,
                                    operation,
                                    TType::Bool,
                                );
                            }
                        } else {
                            left_expr = self.create_binop_expr(
                                left_expr,
                                right_expr,
                                operation,
                                TType::Bool,
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(left_expr)
    }

    fn mid_expr(&mut self) -> NovaResult<Expr> {
        let mut left_expr = self.term()?;
        let current_pos = self.get_current_token_position();
        while self.current_token().is_some_and(|t| t.is_adding_op()) {
            if let Some(operation) = self.current_token().and_then(|t| t.get_operator()) {
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
                            return Err(self.typechecker.create_type_error(
                                left_expr.clone(),
                                right_expr.clone(),
                                operation,
                                current_pos.clone(),
                            ));
                        }
                    }
                    (_, _) => {
                        let function_id: String = match operation {
                            Operator::Addition => {
                                if let Some(custom) = left_expr.get_type().custom_to_string() {
                                    format!("{}::__add__", custom)
                                } else {
                                    // error if no custom method
                                    return Err(self.typechecker.create_type_error(
                                        left_expr.clone(),
                                        right_expr.clone(),
                                        operation,
                                        current_pos.clone(),
                                    ));
                                }
                            }
                            Operator::Subtraction => {
                                if let Some(custom) = left_expr.get_type().custom_to_string() {
                                    format!("{}::__sub__", custom)
                                } else {
                                    // error if no custom method
                                    return Err(self.typechecker.create_type_error(
                                        left_expr.clone(),
                                        right_expr.clone(),
                                        operation,
                                        current_pos.clone(),
                                    ));
                                }
                            }
                            _ => {
                                return Err(self.typechecker.create_type_error(
                                    left_expr.clone(),
                                    right_expr.clone(),
                                    operation,
                                    current_pos.clone(),
                                ))
                            }
                        };

                        //dbg!(function_id.clone());
                        if let Some(overload) =
                            self.typechecker.environment.get(&generate_unique_string(
                                &function_id,
                                &[left_expr.get_type(), right_expr.get_type()],
                            ))
                        {
                            // get return type of function call
                            let pos = self.get_current_token_position();
                            let arguments = vec![left_expr.clone(), right_expr.clone()];
                            let returntype = match overload.ttype {
                                TType::Function { return_type, .. } => *return_type,
                                _ => {
                                    return Err(self.generate_error(
                                        "Expected function",
                                        "Make sure function is defined",
                                    ))
                                }
                            };
                            // return function call expression
                            left_expr = Expr::Literal {
                                ttype: returntype,
                                value: Atom::Call {
                                    name: generate_unique_string(
                                        &function_id,
                                        &[left_expr.get_type(), right_expr.get_type()],
                                    )
                                    .into(),
                                    arguments,
                                    position: pos.clone(),
                                },
                            };
                        } else if let Some(overload) =
                            self.typechecker.environment.get(&function_id)
                        {
                            // get return type of function call
                            let pos = self.get_current_token_position();
                            let arguments = vec![left_expr.clone(), right_expr.clone()];
                            let returntype = match overload.ttype {
                                TType::Function { return_type, .. } => *return_type,
                                _ => {
                                    return Err(self.generate_error(
                                        "Expected function",
                                        "Make sure function is defined",
                                    ))
                                }
                            };
                            // return function call expression
                            left_expr = Expr::Literal {
                                ttype: returntype,
                                value: Atom::Call {
                                    name: function_id.into(),
                                    arguments,
                                    position: pos.clone(),
                                },
                            };
                        } else {
                            // error if no custom method, let user know that the operation is not supported
                            return Err(self.generate_error_with_pos(
                                "Operation not supported",
                                format!("Try implementing the method {}", function_id),
                                current_pos.clone(),
                            ));
                        }
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

    fn ttype(&mut self) -> NovaResult<TType> {
        match self.current_token_value() {
            Some(StructuralSymbol(LeftParen)) => {
                let mut typelist = vec![];
                self.consume_symbol(LeftParen)?;
                // return error if there is no type in the tuple
                if self
                    .current_token()
                    .is_some_and(|t| t.is_symbol(RightParen))
                {
                    return Err(self.generate_error(
                        "Tuple must contain at least two elements",
                        "Add more elements to the tuple",
                    ));
                }
                typelist.push(self.ttype()?);
                while self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
                    self.consume_symbol(Comma)?;
                    // if (5,) single element tuple
                    if self
                        .current_token()
                        .is_some_and(|t| t.is_symbol(RightParen))
                    {
                        self.consume_symbol(RightParen)?;
                        return Ok(TType::Tuple { elements: typelist });
                    }
                    typelist.push(self.ttype()?);
                }
                self.consume_symbol(RightParen)?;
                // if there is only one type in the tuple, return that type
                if typelist.len() == 1 {
                    return Ok(typelist[0].clone());
                }
                Ok(TType::Tuple { elements: typelist })
            }
            Some(Identifier(id)) if "fn" == id.deref() => {
                self.advance();
                self.consume_symbol(LeftParen)?;
                let mut input = vec![];
                if !self
                    .current_token()
                    .is_some_and(|t| t.is_symbol(RightParen))
                {
                    let inner = self.ttype()?;
                    input.push(inner);
                    while self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
                        self.consume_symbol(Comma)?;
                        let inner = self.ttype()?;
                        input.push(inner);
                    }
                    self.consume_symbol(RightParen)?;
                    let mut output = TType::Void;
                    if self
                        .current_token()
                        .is_some_and(|t| t.is_op(Operator::RightArrow))
                    {
                        self.consume_operator(Operator::RightArrow)?;
                        output = self.ttype()?;
                    }
                    Ok(TType::Function {
                        parameters: *Box::new(input),
                        return_type: Box::new(output),
                    })
                } else {
                    self.consume_symbol(RightParen)?;
                    let mut output = TType::Void;
                    if self
                        .current_token()
                        .is_some_and(|t| t.is_op(Operator::RightArrow))
                    {
                        self.consume_operator(Operator::RightArrow)?;
                        output = self.ttype()?;
                    }
                    Ok(TType::Function {
                        parameters: *Box::new(vec![TType::None]),
                        return_type: Box::new(output),
                    })
                }
            }
            Some(StructuralSymbol(DollarSign)) => {
                self.consume_symbol(DollarSign)?;
                let (generictype, _) = self.get_identifier()?;
                Ok(TType::Generic { name: generictype })
            }
            Some(Identifier(id)) if "Option" == id.deref() => {
                self.advance();
                self.consume_symbol(LeftParen)?;
                let ttype = self.ttype()?;
                self.consume_symbol(RightParen)?;
                if let TType::Option { .. } = ttype {
                    return Err(self.generate_error(
                        "Cannot have option directly inside an option",
                        "Type Error: Try removing the extra `?`",
                    ));
                }
                Ok(TType::Option {
                    inner: Box::new(ttype),
                })
            }
            Some(StructuralSymbol(LeftSquareBracket)) => {
                self.consume_symbol(LeftSquareBracket)?;
                let mut inner = TType::None;
                if !self
                    .current_token()
                    .is_some_and(|t| t.is_symbol(RightSquareBracket))
                {
                    inner = self.ttype()?;
                }
                self.consume_symbol(RightSquareBracket)?;
                Ok(TType::List {
                    inner: Box::new(inner),
                })
            }
            Some(Identifier(id)) if "Dyn" == id.deref() => {
                self.advance();
                self.consume_symbol(LeftParen)?;
                let (owned, _) = self.get_identifier()?;
                let mut fields = vec![];
                self.consume_operator(Operator::Assignment)?;
                loop {
                    let (field, _) = self.get_identifier()?;
                    self.consume_operator(Operator::Colon)?;
                    let field_type = self.ttype()?;
                    fields.push((field, field_type));
                    if !self
                        .current_token()
                        .is_some_and(|t| t.is_op(Operator::Addition))
                    {
                        break;
                    }
                    self.advance();
                }
                self.consume_symbol(RightParen)?;
                Ok(TType::Dyn {
                    own: owned,
                    contract: fields,
                })
            }
            Some(Identifier(_)) => {
                let (identifier, pos) = self.get_identifier()?;

                let builtin = 'builtin: {
                    Some(match identifier.as_ref() {
                        "Int" => TType::Int,
                        "Float" => TType::Float,
                        "Bool" => TType::Bool,
                        "String" => TType::String,
                        "Any" => TType::Any,
                        "Char" => TType::Char,
                        "None" => TType::None,
                        _ => break 'builtin None,
                    })
                };
                if let Some(builtin) = builtin {
                    Ok(builtin)
                } else if self
                    .typechecker
                    .environment
                    .custom_types
                    .contains_key(&identifier)
                {
                    let mut type_annotation = vec![];
                    if let Some(StructuralSymbol(LeftParen)) = self.current_token_value() {
                        self.consume_symbol(LeftParen)?;

                        let ta = self.ttype()?;
                        type_annotation.push(ta);
                        while self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
                            self.advance();
                            let ta = self.ttype()?;
                            type_annotation.push(ta);
                        }
                        self.consume_symbol(RightParen)?;
                    }
                    if let Some(generic_len) = self
                        .typechecker
                        .environment
                        .generic_type_struct
                        .get(&identifier)
                    {
                        if generic_len.len() != type_annotation.len() {
                            return Err(self.generate_error_with_pos(
                                format!("Expected {} type parameters", generic_len.len()),
                                format!("Got {} type parameters", type_annotation.len()),
                                pos,
                            ));
                        }
                    }

                    Ok(TType::Custom {
                        name: identifier,
                        type_params: type_annotation,
                    })
                } else {
                    let Some(alias) = self.typechecker.environment.type_alias.get(&identifier)
                    else {
                        return Err(self.generate_error_with_pos(
                            "Unknown type",
                            format!("Unknown type '{identifier}' "),
                            pos,
                        ));
                    };
                    Ok(alias.clone())
                }
            }
            _ => Err(self.generate_error(
                "Expected type annotation",
                format!("Unknown type value {:?}", self.current_token()),
            )),
        }
    }

    fn get_identifier(&mut self) -> NovaResult<(Rc<str>, FilePosition)> {
        let identifier = match self.current_token_value() {
            Some(Identifier(id)) => id.clone(),
            _ => {
                return Err(self.generate_error(
                    "Expected identifier",
                    format!("Cannot assign a value to {:?}", self.current_token(),),
                ));
            }
        };
        let (line, row) = self.get_line_and_row();
        self.advance();
        Ok((
            identifier,
            FilePosition {
                line,
                col: row,
                filepath: self.filepath.clone(),
            },
        ))
    }

    fn parameter_list(&mut self) -> NovaResult<Vec<(TType, Rc<str>)>> {
        let mut parameters: Table<Rc<str>> = Table::new();
        let mut arguments = vec![];

        while self.current_token().is_some_and(|t| t.is_identifier()) {
            let (identifier, pos) = self.get_identifier()?;
            if parameters.has(&identifier) {
                return Err(self.generate_error_with_pos(
                    "parameter identifier already defined",
                    "try using another name",
                    pos,
                ));
            }
            parameters.insert(identifier.clone());
            self.consume_operator(Operator::Colon)?;
            let ttype = self.ttype()?;
            arguments.push((ttype, identifier));

            if !self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
                break;
            }
            self.advance();
        }

        Ok(arguments)
    }

    fn enum_list(&mut self) -> NovaResult<Vec<(TType, Rc<str>)>> {
        let mut parameters = Table::new();
        let mut arguments = vec![];

        while self.current_token().is_some_and(|t| t.is_identifier()) {
            let (identifier, pos) = self.get_identifier()?;
            if parameters.has(&identifier) {
                return Err(self.generate_error_with_pos(
                    "parameter identifier already defined",
                    "try using another name",
                    pos,
                ));
            }
            parameters.insert(identifier.clone());
            // if no colon, then its a unit variant
            if !self
                .current_token()
                .is_some_and(|t| t.is_op(Operator::Colon))
            {
                arguments.push((TType::None, identifier));
                if !self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
                    break;
                }
                self.advance();
                continue;
            }
            self.consume_operator(Operator::Colon)?;
            let ttype = self.ttype()?;
            arguments.push((ttype, identifier));

            if !self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
                break;
            }
            self.advance();
        }

        Ok(arguments)
    }

    fn alternative(&mut self) -> NovaResult<Vec<Statement>> {
        let test = self.top_expr()?;
        let pos = self.get_current_token_position();
        if test.get_type() != TType::Bool {
            return Err(self.generate_error_with_pos(
                "If statement expression must return a bool",
                format!("got {}", test.get_type()),
                pos,
            ));
        }
        self.typechecker.environment.push_block();
        let statements = self.block()?;
        self.typechecker.environment.pop_block();
        let mut alternative: Option<Vec<Statement>> = None;
        if self.current_token().is_some_and(|t| t.is_id("elif")) {
            self.advance();
            alternative = Some(self.alternative()?);
        } else if self.current_token().is_some_and(|t| t.is_id("else")) {
            self.advance();
            self.typechecker.environment.push_block();
            alternative = Some(self.block()?);
            self.typechecker.environment.pop_block();
        };
        Ok(vec![Statement::If {
            ttype: TType::Void,
            test,
            body: statements,
            alternative,
        }])
    }

    fn import_file(&mut self) -> NovaResult<Option<Statement>> {
        self.consume_identifier(Some("import"))?;
        let pos = self.get_current_token_position();
        let import_filepath: PathBuf = match self.current_token_value() {
            Some(StringLiteral(path)) => {
                let path = PathBuf::from_str(path).unwrap();
                self.advance();
                path
            }
            Some(Identifier(name)) => {
                let import_filepath = if name.as_ref() == "super" { ".." } else { name };
                let mut import_filepath = PathBuf::from(import_filepath);
                self.advance();
                while self.current_token().is_some_and(|t| t.is_symbol(Dot)) {
                    self.advance();
                    let (identifier, _) = self.get_identifier()?;
                    if &*identifier == "super" {
                        import_filepath.push("..");
                    } else {
                        import_filepath.push(&*identifier);
                    }
                }
                import_filepath.set_extension("nv");
                import_filepath
            }
            _ => panic!(),
        };
        let resolved_filepath = match self
            .filepath
            .as_ref()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
        {
            Some(mut current_dir) => {
                current_dir.push(import_filepath);
                current_dir
            }
            None => import_filepath,
        };
        let resolved_filepath: Rc<Path> = resolved_filepath.into();
        let tokens = Lexer::read_file(&resolved_filepath);
        let tokens = match tokens {
            Ok(tokens) => tokens,
            Err(_) => {
                return Err(self.generate_error_with_pos(
                    "Error Importing file",
                    format!("Could not import file: {}", resolved_filepath.display()),
                    pos,
                ));
            }
        };
        let tokens = tokens.collect::<NovaResult<Vec<_>>>()?;
        let mut parser = self.clone();
        parser.index = 0;
        parser.filepath = Some(resolved_filepath.clone());
        parser.input = tokens;
        parser.parse()?;
        self.typechecker.environment = parser.typechecker.environment.clone();
        self.modules = parser.modules.clone();
        Ok(Some(Statement::Block {
            body: parser.ast.program.clone(),
            filepath: Some(resolved_filepath),
        }))
    }

    fn match_statement(&mut self) -> NovaResult<Option<Statement>> {
        self.consume_identifier(Some("match"))?;
        let expr = self.expr()?;

        if expr.get_type().custom_to_string().is_some() {
        } else {
            return Err(self.generate_error_with_pos(
                "Match statement expects an enum type",
                format!("got {}", expr.get_type()),
                self.get_current_token_position(),
            ));
        }

        let pos = self.get_current_token_position();
        let mut branches = vec![];
        self.consume_symbol(LeftBrace)?;
        let mut default_branch = None;
        while !self
            .current_token()
            .is_some_and(|t| t.is_symbol(RightBrace))
        {
            let (variant, pos) = self.get_identifier()?;
            if &*variant == "_" {
                // check to see if default branch is already defined
                if default_branch.is_some() {
                    return Err(self.generate_error_with_pos(
                        "default branch already defined",
                        "make sure only one default branch is defined",
                        pos,
                    ));
                }
                self.consume_operator(Operator::FatArrow)?;
                if self.current_token().is_some_and(|t| t.is_symbol(LeftBrace)) {
                    default_branch = Some(self.block()?);
                } else {
                    let body = self.expr()?;
                    default_branch = Some(vec![Statement::Expression {
                        ttype: body.clone().get_type(),
                        expr: body,
                    }])
                };
                continue;
            }
            // collect identifiers
            let mut enum_id = None;
            if self.current_token().is_some_and(|t| t.is_symbol(LeftParen)) {
                self.consume_symbol(LeftParen)?;
                if !self
                    .current_token()
                    .is_some_and(|t| t.is_symbol(RightParen))
                {
                    enum_id = Some(self.get_identifier()?.0);
                }
                self.consume_symbol(RightParen)?;
            }
            self.consume_operator(Operator::FatArrow)?;

            if let Some(fields) = self
                .typechecker
                .environment
                .custom_types
                .get(expr.get_type().custom_to_string().unwrap())
            {
                let new_fields = if let Some(x) = self
                    .typechecker
                    .environment
                    .generic_type_struct
                    .get(expr.get_type().custom_to_string().unwrap())
                {
                    let TType::Custom { type_params, .. } = expr.get_type() else {
                        return Err(self.generate_error_with_pos(
                            "Expected custom type",
                            format!("got {}", expr.get_type()),
                            pos,
                        ));
                    };
                    fields
                        .iter()
                        .map(|(name, ttype)| {
                            let new_ttype =
                                TypeChecker::replace_generic_types(ttype, x, &type_params);
                            (name.clone(), new_ttype)
                        })
                        .collect()
                } else {
                    fields.clone()
                };
                let mut tag = 0;

                // mark if the variant is found
                let mut found = false;
                let mut vtype = TType::None;

                for (i, field) in new_fields.iter().enumerate() {
                    if variant == field.0 {
                        tag = i;
                        vtype = field.1.clone();
                        found = true;
                    }
                }

                if vtype != TType::None && enum_id.is_none() {
                    return Err(self.generate_error_with_pos(
                        format!("variant '{}' is missing Identifier", variant),
                        "Variant(id), id is missing",
                        pos,
                    ));
                }

                if !found {
                    return Err(self.generate_error_with_pos(
                        format!("variant '{}' not found in type", variant),
                        "make sure the variant is in the type",
                        pos,
                    ));
                }

                self.typechecker.environment.push_block();
                self.typechecker.environment.insert_symbol(
                    enum_id.as_deref().unwrap_or_default(),
                    vtype,
                    None,
                    SymbolKind::Variable,
                );
                // get expression if no { }

                //let enum_id = enum_id.unwrap_or_default();
                if self.current_token().is_some_and(|t| t.is_symbol(LeftBrace)) {
                    let body = self.block()?;
                    branches.push((tag, enum_id.clone(), body.clone()));
                    body.clone()
                } else {
                    let body = self.expr()?;
                    branches.push((
                        tag,
                        enum_id.clone(),
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

                self.typechecker.environment.pop_block();
            }
        }
        self.consume_symbol(RightBrace)?;

        if default_branch.is_none() {
            // check to see if all variants are covered
            let mut covered = vec![];
            for (tag, _, _) in branches.clone() {
                covered.push(tag);
            }
            if let Some(fields) = self
                .typechecker
                .environment
                .custom_types
                .get(expr.get_type().custom_to_string().unwrap())
            {
                let new_fields = if let Some(x) = self
                    .typechecker
                    .environment
                    .generic_type_struct
                    .get(expr.get_type().custom_to_string().unwrap())
                {
                    let TType::Custom { type_params, .. } = expr.get_type() else {
                        return Err(self.generate_error_with_pos(
                            "not a custom type",
                            "make sure the type is a custom type",
                            pos,
                        ));
                    };
                    fields
                        .iter()
                        .map(|(name, ttype)| {
                            let new_ttype =
                                TypeChecker::replace_generic_types(ttype, x, &type_params);
                            (name.clone(), new_ttype)
                        })
                        .collect()
                } else {
                    fields.clone()
                };
                for (i, field) in new_fields.iter().enumerate() {
                    if field.0.deref() != "type" && !covered.contains(&i) {
                        return Err(self.generate_error_with_pos(
                            format!("variant '{}' is not covered", field.0),
                            "make sure all variants are covered",
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
            position: pos,
        }))
    }

    // new statement for making type aliases
    // alias identifer = <type>
    fn type_alias(&mut self) -> NovaResult<Option<Statement>> {
        self.consume_identifier(Some("type"))?;
        let (alias, _) = self.get_identifier()?;
        if self
            .typechecker
            .environment
            .custom_types
            .contains_key(&alias)
        {
            return Err(self.generate_error_with_pos(
                format!("type '{}' already defined", alias),
                "try using another name",
                self.get_current_token_position(),
            ));
        }
        self.consume_operator(Operator::Assignment)?;
        let ttype = self.ttype()?;
        self.typechecker
            .environment
            .type_alias
            .insert(alias, ttype.clone());
        Ok(None)
    }

    fn statement(&mut self) -> NovaResult<Option<Statement>> {
        match self.current_token_value() {
            Some(Identifier(id)) => match id.as_ref() {
                "match" => self.match_statement(),
                "type" => self.type_alias(),
                "import" => self.import_file(),
                "pass" => self.pass_statement(),
                "struct" => self.struct_declaration(),
                "if" => self.if_statement(),
                "while" => self.while_statement(),
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
            None => Ok(None),
            _ => self.expression_statement(),
        }
    }

    fn pass_statement(&mut self) -> NovaResult<Option<Statement>> {
        self.consume_identifier(Some("pass"))?;
        Ok(Some(Statement::Pass))
    }

    fn get_id_list(&mut self) -> NovaResult<Vec<Rc<str>>> {
        let mut idlist = vec![];
        self.consume_symbol(LeftParen)?;
        if !self
            .current_token()
            .is_some_and(|t| t.is_symbol(RightParen))
        {
            idlist.push(self.get_identifier()?.0);
        }
        while self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
            self.advance();
            if self
                .current_token()
                .is_some_and(|t| t.is_symbol(RightParen))
            {
                break;
            }
            idlist.push(self.get_identifier()?.0);
        }
        self.consume_symbol(RightParen)?;
        Ok(idlist)
    }

    fn enum_declaration(&mut self) -> NovaResult<Option<Statement>> {
        self.consume_identifier(Some("enum"))?;
        let (enum_name, position) = self.get_identifier()?;

        // Initialize the struct in the environment for recursive types
        self.typechecker
            .environment
            .custom_types
            .insert(enum_name.clone(), vec![]);

        self.typechecker.environment.enums.insert(enum_name.clone());

        let mut generic_field_names = vec![];
        if self.current_token().is_some_and(|t| t.is_symbol(LeftParen)) {
            generic_field_names = self.get_id_list()?;
            self.typechecker
                .environment
                .generic_type_struct
                .insert(enum_name.clone(), generic_field_names.clone());
        }

        self.consume_symbol(LeftBrace)?;
        let parameter_list = self.enum_list()?;
        self.consume_symbol(RightBrace)?;
        let mut fields = vec![];
        let mut type_parameters = vec![];
        let mut generics_table = Table::new();

        for (field_type, field_name) in parameter_list.clone() {
            generics_table.extend(TypeChecker::collect_generics(std::slice::from_ref(
                &field_type,
            )));
            type_parameters.push(field_type.clone());
            fields.push((field_name, field_type));
        }
        fields.push(("type".into(), TType::String));

        for generic_type in generics_table.items.iter() {
            if !generic_field_names.contains(generic_type) {
                return Err(self.generate_error_with_pos(
                    format!(
                        "enum '{}' is missing generic type {}",
                        enum_name, generic_type
                    ),
                    "You must include generic types in enum name(...generictypes)",
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
            if generics_table.is_empty() {
                self.typechecker.environment.insert_symbol(
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
                let genericmap = generic_field_names
                    .iter()
                    .map(|x| TType::Generic { name: x.clone() })
                    .collect::<Vec<TType>>();

                self.typechecker.environment.insert_symbol(
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

        self.typechecker
            .environment
            .custom_types
            .insert(enum_name.clone(), fields);

        if !self.typechecker.environment.has(&enum_name) {
            self.typechecker
                .environment
                .no_override
                .insert(enum_name.clone());
        } else {
            return Err(self.generate_error_with_pos(
                format!("Enum '{}' is already instantiated", enum_name),
                "Cannot reinstantiate the same type",
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

    fn struct_declaration(&mut self) -> NovaResult<Option<Statement>> {
        self.consume_identifier(Some("struct"))?;
        let (struct_name, position) = self.get_identifier()?;

        // Initialize the struct in the environment for recursive types
        self.typechecker
            .environment
            .custom_types
            .insert(struct_name.clone(), vec![]);

        let mut generic_field_names = vec![];
        if self.current_token().is_some_and(|t| t.is_symbol(LeftParen)) {
            generic_field_names = self.get_id_list()?;
            self.typechecker
                .environment
                .generic_type_struct
                .insert(struct_name.clone(), generic_field_names.clone());
        }

        self.consume_symbol(LeftBrace)?;
        let parameter_list = self.parameter_list()?;
        self.consume_symbol(RightBrace)?;

        let mut fields = vec![];
        let mut type_parameters = vec![];
        let mut generics_table = Table::new();

        for (field_type, field_name) in parameter_list.clone() {
            generics_table.extend(TypeChecker::collect_generics(std::slice::from_ref(
                &field_type,
            )));
            type_parameters.push(field_type.clone());
            fields.push((field_name, field_type));
        }
        fields.push(("type".into(), TType::String));

        for generic_type in generics_table.items.iter() {
            if !generic_field_names.contains(generic_type) {
                return Err(self.generate_error_with_pos(
                    format!(
                        "Struct '{}' is missing generic type {}",
                        struct_name, generic_type
                    ),
                    "You must include generic types in struct name(...generictypes)",
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

        if !self.typechecker.environment.has(&struct_name) {
            self.typechecker
                .environment
                .no_override
                .insert(struct_name.clone());
            if generics_table.is_empty() {
                self.typechecker.environment.insert_symbol(
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

                self.typechecker.environment.insert_symbol(
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
            self.typechecker
                .environment
                .custom_types
                .insert(struct_name.clone(), fields);
        } else {
            return Err(self.generate_error_with_pos(
                format!("Struct '{}' is already instantiated", struct_name),
                "Cannot reinstantiate the same type",
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

    fn for_statement(&mut self) -> NovaResult<Option<Statement>> {
        self.consume_identifier(Some("for"))?;

        if let Some(Keyword(KeyWord::In)) = self.peek_offset_value(1) {
            // Handle foreach statement

            let (identifier, pos) = self.get_identifier()?;
            if self.typechecker.environment.has(&identifier) {
                return Err(self.generate_error_with_pos(
                    "identifier already used",
                    format!("identifier '{identifier}' is already used within this scope"),
                    pos.clone(),
                ));
            }
            self.consume_keyword(KeyWord::In)?;
            let arraypos = self.get_current_token_position();
            let array = self.expr()?;
            // check for inclusiverange operator
            match self.current_token_value() {
                Some(Operator(Operator::ExclusiveRange)) => {
                    let start_range = array;
                    self.consume_operator(Operator::ExclusiveRange)?;
                    let end_range = self.expr()?;
                    self.typechecker.environment.push_block();
                    self.typechecker.environment.insert_symbol(
                        &identifier,
                        TType::Int,
                        Some(pos),
                        SymbolKind::Variable,
                    );
                    let body = self.block()?;
                    self.typechecker.environment.pop_block();
                    Ok(Some(Statement::ForRange {
                        identifier,
                        body,
                        start: start_range,
                        end: end_range,
                        inclusive: true,
                        step: None,
                    }))
                }
                Some(Operator(Operator::InclusiveRange)) => {
                    let start_range = array;
                    self.consume_operator(Operator::InclusiveRange)?;
                    let end_range = self.expr()?;
                    self.typechecker.environment.push_block();
                    self.typechecker.environment.insert_symbol(
                        &identifier,
                        TType::Int,
                        Some(pos),
                        SymbolKind::Variable,
                    );
                    let body = self.block()?;
                    self.typechecker.environment.pop_block();
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
                    self.typechecker.environment.push_block();
                    // check if array has type array and then assign identifier to that type
                    if let TType::List { inner } = array.get_type() {
                        self.typechecker.environment.insert_symbol(
                            &identifier,
                            *inner,
                            Some(pos),
                            SymbolKind::Variable,
                        )
                    } else {
                        return Err(self.generate_error_with_pos(
                            "foreach can only iterate over arrays",
                            format!("got {}", array.get_type()),
                            arraypos.clone(),
                        ));
                    }
                    let body = self.block()?;
                    self.typechecker.environment.pop_block();

                    Ok(Some(Statement::Foreach {
                        identifier,
                        expr: array,
                        body,
                        position: arraypos,
                    }))
                }
            }
        } else {
            // Handle regular for statement
            self.typechecker.environment.push_block();
            let init = self.expr()?;
            self.consume_symbol(Semicolon)?;
            let testpos = self.get_current_token_position();
            let test = self.expr()?;
            self.consume_symbol(Semicolon)?;
            let inc = self.expr()?;
            if test.get_type() != TType::Bool && test.get_type() != TType::Void {
                return Err(self.generate_error_with_pos(
                    "test expression must return a bool",
                    format!("got {}", test.get_type()),
                    testpos,
                ));
            }
            let body = self.block()?;
            self.typechecker.environment.pop_block();
            Ok(Some(Statement::For {
                init,
                test,
                inc,
                body,
            }))
        }
    }

    fn while_statement(&mut self) -> NovaResult<Option<Statement>> {
        self.consume_identifier(Some("while"))?;
        // check for let keyword
        if self.current_token().is_some_and(|t| t.is_id("let")) {
            self.advance();
            let (identifier, pos) = self.get_identifier()?;
            self.consume_operator(Operator::Assignment)?;
            let expr = self.expr()?;
            let inner = if let TType::Option { inner } = expr.get_type() {
                inner
            } else {
                return Err(self.generate_error_with_pos(
                    "unwrap expects an option type",
                    format!("got {}", expr.get_type()),
                    pos.clone(),
                ));
            };

            // make sure symbol doesn't already exist
            if self.typechecker.environment.has(&identifier) {
                Err(self.generate_error_with_pos(
                    format!("Symbol '{}' is already instantiated", identifier),
                    "Cannot reinstantiate the same symbol in the same scope",
                    pos.clone(),
                ))
            } else {
                self.typechecker.environment.push_block();
                self.typechecker.environment.insert_symbol(
                    &identifier,
                    *inner.clone(),
                    Some(pos),
                    SymbolKind::Variable,
                );
                let statements = self.block()?;
                self.typechecker.environment.pop_block();

                Ok(Some(Statement::WhileLet {
                    identifier,
                    expr,
                    body: statements,
                }))
            }
        } else {
            let testpos = self.get_current_token_position();
            let test = self.top_expr()?;
            if test.get_type() != TType::Bool && test.get_type() != TType::Void {
                return Err(self.generate_error_with_pos(
                    "test expression must return a bool",
                    format!("got {}", test.get_type()),
                    testpos,
                ));
            }
            self.typechecker.environment.push_block();
            let statements = self.block()?;
            self.typechecker.environment.pop_block();

            Ok(Some(Statement::While {
                test,
                body: statements,
            }))
        }
    }

    fn if_statement(&mut self) -> NovaResult<Option<Statement>> {
        self.consume_identifier(Some("if"))?;

        if self.current_token().is_some_and(|t| t.is_id("let")) {
            // Handle if let statement
            self.advance(); // consume 'let'
            let mut global = false;
            let (mut identifier, mut pos) = self.get_identifier()?;
            if identifier.deref() == "global" {
                (identifier, pos) = self.get_identifier()?;
                global = true
            }
            self.consume_operator(Operator::Assignment)?;
            let expr = self.expr()?;
            let inner = if let TType::Option { inner } = expr.get_type() {
                inner
            } else {
                return Err(self.generate_error_with_pos(
                    "unwrap expects an option type",
                    format!("got {}", expr.get_type()),
                    pos.clone(),
                ));
            };

            // make sure symbol doesn't already exist
            if self.typechecker.environment.has(&identifier) {
                Err(self.generate_error_with_pos(
                    format!("Symbol '{}' is already instantiated", identifier),
                    "Cannot reinstantiate the same symbol in the same scope",
                    pos.clone(),
                ))
            } else {
                self.typechecker.environment.push_block();
                self.typechecker.environment.insert_symbol(
                    &identifier,
                    *inner.clone(),
                    Some(pos),
                    SymbolKind::Variable,
                );
                let body = self.block()?;
                self.typechecker.environment.pop_block();

                let mut alternative: Option<Vec<Statement>> = None;
                if self.current_token().is_some_and(|t| t.is_id("elif")) {
                    self.advance();
                    alternative = Some(self.alternative()?);
                } else if self.current_token().is_some_and(|t| t.is_id("else")) {
                    self.advance();
                    self.typechecker.environment.push_block();
                    alternative = Some(self.block()?);
                    self.typechecker.environment.pop_block();
                };

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
            let test = self.expr()?;
            if test.get_type() != TType::Bool {
                return Err(self.generate_error_with_pos(
                    "If statement's expression must return a bool",
                    format!("got {}", test.get_type()),
                    testpos.clone(),
                ));
            }
            self.typechecker.environment.push_block();
            let body = self.block()?;
            self.typechecker.environment.pop_block();
            let mut alternative: Option<Vec<Statement>> = None;

            if self.current_token().is_some_and(|t| t.is_id("elif")) {
                self.advance();
                alternative = Some(self.alternative()?);
            } else if self.current_token().is_some_and(|t| t.is_id("else")) {
                self.advance();
                self.typechecker.environment.push_block();
                alternative = Some(self.block()?);
                self.typechecker.environment.pop_block();
            };

            Ok(Some(Statement::If {
                ttype: TType::Void,
                test,
                body,
                alternative,
            }))
        }
    }

    fn let_expr(&mut self) -> NovaResult<Expr> {
        self.consume_identifier(Some("let"))?;
        let mut global = false;
        // refactor out into two parsing ways for ident. one with module and one without
        let (mut identifier, mut pos) = self.get_identifier()?;
        if self.modules.has(&identifier) {
            // throw error
            return Err(self.generate_error_with_pos(
                "Cannot use module as identifier",
                format!("got {}", identifier),
                pos.clone(),
            ));
        }
        if identifier.deref() == "global" {
            (identifier, pos) = self.get_identifier()?;
            global = true
        }
        let ttype;
        let expr;
        if self
            .current_token()
            .is_some_and(|t| t.is_op(Operator::Colon))
        {
            self.consume_operator(Operator::Colon)?;
            ttype = self.ttype()?;
            self.consume_operator(Operator::Assignment)?;
            expr = self.expr()?;
            match (
                self.typechecker.check_and_map_types(
                    std::slice::from_ref(&ttype),
                    &[expr.get_type()],
                    &mut HashMap::default(),
                    pos.clone(),
                ),
                self.typechecker.check_and_map_types(
                    &[expr.get_type()],
                    std::slice::from_ref(&ttype),
                    &mut HashMap::default(),
                    pos.clone(),
                ),
            ) {
                (Ok(_), Ok(_)) => {}
                _ => {
                    return Err(self.generate_error_with_pos(
                        format!("Cannot assign {} to {}", expr.get_type(), ttype),
                        "Make sure the expression returns the givin type",
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
                "Make sure the expression returns a value",
                pos.clone(),
            ));
        }
        // make sure symbol doesnt already exist
        if self.typechecker.environment.has(&identifier) {
            Err(self.generate_error_with_pos(
                format!("Symbol '{}' is already instantiated", identifier),
                "Cannot reinstantiate the same symbol in the same scope",
                pos.clone(),
            ))
        } else {
            self.typechecker.environment.insert_symbol(
                &identifier,
                ttype.clone(),
                Some(pos.clone()),
                SymbolKind::Variable,
            );
            Ok(Expr::Let {
                ttype: TType::Void,
                identifier,
                expr: Box::new(expr),
                global,
            })
        }
    }

    fn return_statement(&mut self) -> NovaResult<Option<Statement>> {
        self.consume_identifier(Some("return"))?;
        let expr = self.expr()?;
        Ok(Some(Statement::Return {
            ttype: expr.get_type(),
            expr,
        }))
    }

    fn function_declaration(&mut self) -> NovaResult<Option<Statement>> {
        self.consume_identifier(Some("fn"))?;
        let builtin_types = [
            "List", "Option", "Function", "Tuple", "Bool", "Int", "Float", "String", "Char",
        ];
        let mut is_extended = false;
        let mut is_mod = false;
        let mut get_first = false;
        // check if dunder method
        // check to see if next is the extends keyword with a custom type name and get the custom type name
        let mut custom_type = Rc::default();
        if self.current_token().is_some_and(|t| t.is_id("extends")) {
            self.advance();
            // if current token is ( then get the custom type name , otherwise extend from first argument
            if self.current_token().is_some_and(|t| t.is_symbol(LeftParen)) {
                self.consume_symbol(LeftParen)?;
                (custom_type, _) = self.get_identifier()?;
                // check to see if its a valid custom type
                if !self
                    .typechecker
                    .environment
                    .custom_types
                    .contains_key(&custom_type)
                    && !builtin_types.contains(&&*custom_type)
                {
                    return Err(self.generate_error_with_pos(
                        format!("Custom type {} does not exist", custom_type),
                        "Cannot extend a non existent custom type",
                        self.get_current_token_position(),
                    ));
                }
                self.consume_symbol(RightParen)?;
                is_extended = true;
            } else {
                get_first = true;
            }
        } else if self.current_token().is_some_and(|t| t.is_id("mod")) {
            self.advance();
            self.consume_symbol(LeftParen)?;
            (custom_type, _) = self.get_identifier()?;
            // check to see if its a valid custom type
            if !self.modules.has(&custom_type) {
                return Err(self.generate_error_with_pos(
                    format!("Module {} does not exist", custom_type),
                    "Cannot extend a non existent module",
                    self.get_current_token_position(),
                ));
            }
            self.consume_symbol(RightParen)?;
            is_mod = true;
        }

        let (mut identifier, pos) = self.get_identifier()?;

        // get parameters
        self.consume_symbol(LeftParen)?;
        let parameters = self.parameter_list()?;
        self.consume_symbol(RightParen)?;
        // get output type

        let mut output = TType::Void;
        if self.current_token().is_some_and(|t| t.is_symbol(LeftBrace)) {
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
        let generic = TypeChecker::is_generic(&typeinput);

        // check if dunder method
        match identifier.as_ref() {
            id @ "__add__"
            | id @ "__and__"
            | id @ "__or__"
            | id @ "__sub__"
            | id @ "__mul__"
            | id @ "__div__"
            | id @ "__mod__"
            | id @ "__eq__"
            | id @ "__ne__"
            | id @ "__lt__"
            | id @ "__le__"
            | id @ "__gt__"
            | id @ "__ge__" => {
                if parameters.len() != 2 {
                    return Err(self.generate_error_with_pos(
                        format!("Dunder method {id} expects Two parameters"),
                        format!("got {}", parameters.len()),
                        pos.clone(),
                    ));
                }
                // if is_extended {
                //     // return error
                //     return Err(self.generate_error_with_pos(
                //         format!("Cannot extend from {id} "),
                //         "Cannot extend from dunder methods",
                //         pos.clone(),
                //     ));
                // }
                // if generic {
                //     return Err(self.generate_error_with_pos(
                //         format!("Cannot create generic function for {id}"),
                //         "Cannot create generic function for dunder methods",
                //         pos.clone(),
                //     ));
                // }
                if is_mod {
                    return Err(self.generate_error_with_pos(
                        format!("Cannot create module function for {id}"),
                        "Cannot create module function for dunder methods",
                        pos.clone(),
                    ));
                }
                if !get_first {
                    return Err(self.generate_error_with_pos(
                        format!("Must extend from {id}"),
                        "dunder methods must extends from a custom type",
                        pos.clone(),
                    ));
                }
            }
            _ => {}
        }

        if is_extended || is_mod {
            identifier = format!("{}::{}", custom_type, identifier).into();
        }

        if !is_extended && get_first {
            //println!("{} {}", identifier, parameters.len());
            if let Some((ttype, _)) = parameters.first() {
                identifier = match ttype {
                    TType::Custom { name, .. } => {
                        format!("{}::{}", name, identifier)
                    }
                    TType::List { .. } => {
                        format!("List::{}", identifier)
                    }
                    TType::Option { .. } => {
                        format!("Option::{}", identifier)
                    }
                    TType::Function { .. } => {
                        format!("Function::{}", identifier)
                    }
                    TType::Tuple { .. } => {
                        format!("Tuple::{}", identifier)
                    }
                    TType::Bool => {
                        format!("Bool::{}", identifier)
                    }
                    TType::Int => {
                        format!("Int::{}", identifier)
                    }
                    TType::Float => {
                        format!("Float::{}", identifier)
                    }
                    TType::String => {
                        format!("String::{}", identifier)
                    }
                    TType::Char => {
                        format!("Char::{}", identifier)
                    }
                    _ => {
                        // error
                        return Err(self.generate_error_with_pos(
                            "Cannot extend from type",
                            "Cannot extend from this type",
                            pos.clone(),
                        ));
                    }
                }
                .into();
            }
        }
        //dbg!(identifier.clone());
        // build helper vecs
        let mut input = vec![];
        for (ttype, identifier) in parameters.clone() {
            if let TType::Function { .. } = ttype.clone() {
                // check if generic function exist
                if self.typechecker.environment.has(&identifier) {
                    return Err(self.generate_error_with_pos(
                        format!("Generic Function {} already defined", &identifier),
                        "Cannot redefine a generic function",
                        pos.clone(),
                    ));
                }
                // check if normal function exist
                if self.typechecker.environment.has(&identifier) {
                    return Err(self.generate_error_with_pos(
                        format!("Function {} already defined", &identifier,),
                        "Cannot redefine a generic function",
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

        // insert function into environment
        if !generic {
            // check if normal function exist
            if self
                .typechecker
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
                    "Cannot redefine a function with the same signature",
                    pos.clone(),
                ));
            }
            self.typechecker.environment.insert_symbol(
                &identifier,
                TType::Function {
                    parameters: typeinput.clone(),
                    return_type: Box::new(output.clone()),
                },
                Some(pos.clone()),
                SymbolKind::Function,
            );
            identifier = generate_unique_string(&identifier, &typeinput).into();
        } else {
            if self.typechecker.environment.no_override.has(&identifier) {
                return Err(self.generate_error_with_pos(
                    format!(
                        "Cannot create generic functon since, {} is already defined",
                        &identifier
                    ),
                    "Cannot create generic function since this function is overload-able",
                    pos.clone(),
                ));
            }
            self.typechecker.environment.insert_symbol(
                &identifier,
                TType::Function {
                    parameters: typeinput.clone(),
                    return_type: Box::new(output.clone()),
                },
                Some(pos.clone()),
                SymbolKind::GenericFunction,
            );
        }
        //println!("{} {}", identifier, parameters.len());
        // check for no rightbrace
        if self
            .current_token()
            .is_some_and(|t| !t.is_symbol(LeftBrace))
        {
            //dbg!(&identifier);
            self.typechecker.environment.forward_declarations.insert(
                identifier.clone(),
                (typeinput.clone(), output.clone(), pos.clone()),
            );
            return Ok(Some(Statement::ForwardDec { identifier }));
        }

        //dbg!(identifier.clone());
        self.typechecker
            .environment
            .no_override
            .insert(identifier.clone());
        let mut generic_list = TypeChecker::collect_generics(&typeinput);
        generic_list.extend(TypeChecker::collect_generics(&[output.clone()]));
        self.typechecker
            .environment
            .live_generics
            .push(generic_list.clone());
        // parse body with scope
        self.typechecker.environment.push_scope();
        // insert params into scope
        for (ttype, id) in parameters.iter() {
            match ttype {
                TType::Function {
                    parameters,
                    return_type,
                } => {
                    self.typechecker.environment.insert_symbol(
                        id,
                        TType::Function {
                            parameters: parameters.clone(),
                            return_type: return_type.clone(),
                        },
                        Some(pos.clone()),
                        SymbolKind::Parameter,
                    );
                }
                _ => self.typechecker.environment.insert_symbol(
                    id,
                    ttype.clone(),
                    Some(pos.clone()),
                    SymbolKind::Parameter,
                ),
            }
        }

        let mut statements = self.block()?;

        // capture variables -----------------------------------
        let mut captured: Vec<Rc<str>> = self
            .typechecker
            .environment
            .captured
            .last()
            .unwrap()
            .iter()
            .map(|v| v.0.clone())
            .collect();

        self.typechecker.environment.pop_scope();
        self.typechecker.environment.live_generics.pop();
        for c in captured.iter() {
            if let Some(mc) = self.typechecker.environment.get_type_capture(&c.clone()) {
                let pos = self.get_current_token_position();

                self.typechecker
                    .environment
                    .captured
                    .last_mut()
                    .unwrap()
                    .insert(
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
            .typechecker
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
                captured.retain(|x| x != &name);
            }
        }

        for dc in captured.iter() {
            if let Some(v) = self.typechecker.environment.values.last().unwrap().get(dc) {
                if let SymbolKind::Captured = v.kind {
                } else {
                    self.typechecker
                        .environment
                        .captured
                        .last_mut()
                        .unwrap()
                        .remove(dc);
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
        let will_return = self
            .typechecker
            .will_return(&statements, output.clone(), pos.clone())?;
        if !will_return && output != TType::Void {
            if let Some(Statement::Pass) = statements.last() {
                // do nothing
            } else if !will_return {
                return Err(self.generate_error_with_pos(
                    "Function is missing a return statement in a branch",
                    "Function missing return",
                    pos.clone(),
                ));
            }
        }

        // dbg!(identifier.clone());
        Ok(Some(Statement::Function {
            ttype: output,
            identifier,
            parameters: input,
            body: statements,
            captures: captured,
        }))
    }

    fn expression_statement(&mut self) -> NovaResult<Option<Statement>> {
        // check for return statement
        self.expr().map(|expr| {
            Some(Statement::Expression {
                ttype: expr.get_type(),
                expr,
            })
        })
    }

    fn block(&mut self) -> NovaResult<Vec<Statement>> {
        self.consume_symbol(LeftBrace)?;
        if self
            .current_token()
            .is_some_and(|t| t.is_symbol(RightBrace))
        {
            self.advance();
            return Ok(vec![]);
        }
        let statements = self.compound_statement()?;
        self.consume_symbol(RightBrace)?;
        Ok(statements)
    }

    fn block_expr(&mut self) -> NovaResult<Expr> {
        self.consume_symbol(LeftBrace)?;
        self.typechecker.environment.push_block();
        let statements = self.compound_statement()?;
        self.typechecker.environment.pop_block();
        self.consume_symbol(RightBrace)?;
        // check if last statement is a statement expression
        let mut ttype = match statements.last().cloned() {
            Some(Statement::Expression { ttype, .. }) => ttype,
            _ => TType::Void,
        };
        // check if type is None
        if ttype == TType::None {
            ttype = TType::Void;
        }
        Ok(Expr::Block {
            body: statements,
            ttype: ttype.clone(),
        })
    }

    fn compound_statement(&mut self) -> NovaResult<Vec<Statement>> {
        let mut initial_statements = vec![];
        if let Some(statement) = self.statement()? {
            initial_statements.push(statement)
        };
        let statements = {
            let mut statements = initial_statements;

            while self.current_token().is_some_and(|t| t.is_symbol(Semicolon))
                || !self.is_current_eof()
            {
                let index_change = self.index;
                if self.current_token().is_some_and(|t| t.is_symbol(Semicolon)) {
                    self.advance()
                }
                if self
                    .current_token()
                    .is_some_and(|t| t.is_symbol(RightBrace))
                {
                    break;
                }
                if let Some(statement) = self.statement()? {
                    statements.push(statement);
                }
                if self.index == index_change {
                    return Err(self.generate_error("Expected statement", "Expected statement"));
                }
            }
            statements
        };
        Ok(statements)
    }

    pub fn parse(&mut self) -> NovaResult<()> {
        // if repl mode no need to parse module
        if self.filepath.is_none() {
            self.ast.program = self.compound_statement()?;
            return self.eof();
        }

        if self.current_token().is_some_and(|t| t.is_id("module")) {
            self.consume_identifier(Some("module"))?;
            let (module_name, _) = self.get_identifier()?;
            if self.modules.has(&module_name) {
                return Ok(());
            }
            self.modules.insert(module_name);
        } else {
            return Err(self.generate_error(
                "Expected module declaration",
                "Module declaration must be the first statement",
            ));
        }

        self.ast.program = self.compound_statement()?;
        self.eof()
    }
}
