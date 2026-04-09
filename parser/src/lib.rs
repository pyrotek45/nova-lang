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
    nodes::{Arg, Ast, Atom, Expr, Field, Pattern, Statement, Symbol, SymbolKind},
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
    depth: usize,
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
        depth: 0,
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
        depth: 0,
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
                inner: Box::new(TType::Generic { name: "T".into() }),
            }],
            return_type: Box::new(TType::Generic { name: "T".into() }),
        },
        None,
        SymbolKind::GenericFunction,
    );
    tc.environment.insert_symbol(
        "Some",
        TType::Function {
            parameters: vec![TType::Generic { name: "T".into() }],
            return_type: Box::new(TType::Option {
                inner: Box::new(TType::Generic { name: "T".into() }),
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
            parameters: vec![TType::Generic { name: "T".into() }],
            return_type: Box::new(TType::Generic { name: "T".into() }),
        },
        None,
        SymbolKind::GenericFunction,
    );
    tc
}

/// Format an `Option<&Token>` for clean error messages.
fn fmt_token_opt(token: Option<&Token>) -> String {
    match token {
        Some(t) => format!("{}", t),
        None => "end of file".to_string(),
    }
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
            let leftover = fmt_token_opt(self.current_token());
            Err(Box::new(NovaError::Parsing {
                msg: format!("Unexpected token {} after end of statement", leftover).into(),
                note: "The parser finished a statement but found extra tokens.\n  Common causes:\n  - Missing semicolon between statements\n  - Extra closing brace `}`\n  - Stray token or typo\n  Statements in Nova are separated by newlines or semicolons.".into(),
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
                format!(
                    "unexpected token {}, expected `{op}`",
                    fmt_token_opt(unexpected)
                ),
                format!("expected `{op}`"),
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
                format!(
                    "unexpected token {}, expected `{sym}`",
                    fmt_token_opt(unexpected)
                ),
                format!("expected `{sym}`"),
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
                format!(
                    "unexpected token {}, expected `{kw}`",
                    fmt_token_opt(unexpected)
                ),
                format!("expected `{kw}`"),
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
                format!(
                    "unexpected token {}, expected an identifier",
                    fmt_token_opt(unexpected)
                ),
                match symbol {
                    Some(s) => format!("expected `{s}`"),
                    None => "expected an identifier".to_string(),
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
                format!(
                    "unexpected operator {}, expected unary sign",
                    fmt_token_opt(self.current_token())
                ),
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
                        format!("Cannot call method `.{}()` on type `{}`", identifier, ttype),
                        format!(
                            "The type `{}` does not support method calls.\n  Only struct/enum types and built-in types (List, Option, String, Int, Float, Bool, Char, Tuple) support methods.\n  To define a method on a type, use:\n    `fn extends method_name(self: MyType, ...) -> ReturnType {{ ... }}`\n  Then call it as: `value.method_name()`",
                            ttype
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
            if let Some(cap) = self.typechecker.environment.captured.last_mut() {
                cap.insert(
                    identifier.clone(),
                    Symbol {
                        id: identifier.clone(),
                        ttype: function_type.clone(),
                        pos: Some(pos.clone()),
                        kind: SymbolKind::Captured,
                    },
                );
            }
            self.handle_function_call(
                function_type,
                function_id,
                function_kind,
                arguments,
                argument_types,
                pos,
            )
        } else {
            let arg_types_str = argument_types
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            Err(self.generate_error_with_pos(
                format!("No matching method `{}` for argument types [{}]", identifier, arg_types_str),
                format!(
                    "No method `{}` accepts [{}] as arguments.\n  Check that:\n  - The method is defined for this type (using `fn extends`)\n  - The argument types match the method's parameter types\n  Example: `fn extends {}(self: TypeName, ...) -> ReturnType {{ ... }}`",
                    identifier, arg_types_str, identifier
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
                    format!("Expected a function type, found `{}`", function_type),
                    "This identifier does not refer to a callable function. In Nova, functions are declared with `fn name(param: Type) -> ReturnType { ... }`".to_string(),
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
        *return_type = self.typechecker.get_output(
            *return_type,
            &mut type_map,
            pos.clone(),
        )?;

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
            let is_live = self
                .typechecker
                .environment
                .live_generics
                .last()
                .is_some_and(|lg| lg.has(&generic));
            if !is_live {
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
                let is_live = self
                    .typechecker
                    .environment
                    .live_generics
                    .last()
                    .is_some_and(|lg| lg.has(&generic));
                if !is_live {
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
            if let Some(cap) = self.typechecker.environment.captured.last_mut() {
                cap.insert(
                    identifier.clone(),
                    Symbol {
                        id: identifier.clone(),
                        ttype: function_type.clone(),
                        pos: Some(pos.clone()),
                        kind: SymbolKind::Captured,
                    },
                );
            }
            self.handle_function_call(
                function_type,
                function_id,
                function_kind,
                arguments,
                argument_types,
                pos,
            )
        } else {
            let arg_types_str = argument_types
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            Err(self.generate_error_with_pos(
                format!("No matching function `{}` for argument types [{}]", identifier, arg_types_str),
                format!(
                    "No function signature `{}` accepts [{}] as arguments.\n  Check that:\n  - The function exists and is defined before this call\n  - The argument types match the function's parameter types\n  - If calling a method, use `value.method(args)` syntax",
                    identifier, arg_types_str
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
                        return Err(self.generate_error_with_pos(
                            format!("Expected a generic custom type, found `{}`", lhs.get_type()),
                            "This type has generic type parameters but the value does not carry type parameter information.\n  This is an internal type error — please report it.",
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
                // Type is known (custom_to_string returned Some) but not in custom_types
                // This means it's a built-in type like Tuple, List, Int, etc.
                match lhs.get_type() {
                    TType::Tuple { ref elements } => {
                        return Err(self.generate_error_with_pos(
                            format!("Cannot use dot syntax `.{}` on a Tuple", identifier),
                            format!(
                                "Tuples use bracket indexing, not dot access. Use `my_tuple[0]`, `my_tuple[1]`, etc. This tuple has {} element(s) (indices 0..{})",
                                elements.len(),
                                elements.len().saturating_sub(1)
                            ),
                            pos,
                        ));
                    }
                    _ => return self.generate_field_not_found_error(&identifier, type_name, pos),
                }
            }
        } else {
            // check if dynamic type with fields in contract
            if let TType::Dyn { contract, .. } = lhs.get_type() {
                if identifier.as_ref() == "type" {
                    // Every struct has an implicit "type" field (String) at runtime.
                    // Allow it on Dyn even though it's not in the contract.
                    lhs = Expr::DynField {
                        ttype: TType::String,
                        name: "type".into(),
                        expr: Box::new(lhs),
                        position: pos.clone(),
                    };
                } else if let Some((name, field_type)) = contract
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
            } else if let TType::Tuple { elements } = lhs.get_type() {
                return Err(self.generate_error_with_pos(
                    format!("Cannot use dot syntax `.{}` on a Tuple", identifier),
                    format!(
                        "Tuples use bracket indexing, not dot access. Use `my_tuple[0]`, `my_tuple[1]`, etc. This tuple has {} element(s) (indices 0..{})",
                        elements.len(),
                        elements.len().saturating_sub(1)
                    ),
                    pos,
                ));
            } else {
                let hint = match lhs.get_type() {
                    TType::List { .. } => "Lists use methods like `.len()`, `.push(val)`, `.map(|x: T| ...)`, `.filter(|x: T| ...)`. Use `list[index]` for element access.",
                    TType::Option { .. } => "Options use `if let val = opt_expr { ... }` to unwrap, or `.unwrap()` to get the inner value.",
                    TType::Int | TType::Float => "Numeric types use methods like `.to(end)` for ranges. Use `Cast::string(val)` to convert to String.",
                    TType::String => "Strings use methods like `.len()`, `.split(delim)`, `.trim()`. Use `+` for concatenation.",
                    TType::Bool => "Bool does not have fields. Use `Cast::string(val)` to convert to String.",
                    _ => "Only struct/enum types support dot field access. Use bracket indexing `[index]` for tuples and lists.",
                };
                return Err(self.generate_error_with_pos(
                    format!("Cannot access field `.{}` on type `{}`", identifier, lhs.get_type()),
                    hint.to_string(),
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
        let mut available_fields = String::new();
        if let Some(fields) = self.typechecker.environment.custom_types.get(type_name) {
            let field_names: Vec<&str> = fields
                .iter()
                .filter(|(name, _)| name.as_ref() != "type")
                .map(|(name, _)| name.as_ref())
                .collect();
            if !field_names.is_empty() {
                available_fields = format!(" Available fields: {}", field_names.join(", "));
            }
        }
        Err(self.generate_error_with_pos(
            format!("No field '{}' found on type `{}`", identifier, type_name),
            format!(
                "The type `{}` does not have a field named `{}`.{}",
                type_name, identifier, available_fields
            ),
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
                            format!("Incorrect number of arguments: expected {}, got {}", parameters.len(), arguments.len()),
                            format!("This function expects {} argument(s) but {} were provided.", parameters.len(), arguments.len()),
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
                    *return_type = self.typechecker.get_output(
                        *return_type.clone(),
                        &mut type_map,
                        pos,
                    )?;
                    lhs = Expr::Call {
                        ttype: *return_type,
                        name: "anon".into(),
                        function: Box::new(lhs),
                        args: arguments,
                    };
                } else {
                    return Err(self.generate_error_with_pos(
                        format!("Cannot call `{}` — it is not a function", lhs.get_type()),
                        "Expected a callable function type, but found a non-function value.\n  Only functions and closures can be called with `(...)`.",
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
            format!("'{}' is not defined", identifier),
            format!(
                "The identifier `{}` was not found in the current scope. Make sure it is declared with `let` before use, or check for typos. If this is a type, it must be declared with `struct` or `enum` before this point.",
                identifier
            ),
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
                                "List index must be an Int",
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
                                "List index must be an Int",
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
                                "List index must be an Int",
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
                            "List index must be an Int",
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
                        "Tuple must be indexed with an integer literal",
                        format!(
                            "Cannot index into `{}` with `{}`.\n  Use a constant integer: `my_tuple[0]`, `my_tuple[1]`, etc.",
                            lhs.get_type(),
                            fmt_token_opt(self.current_token())
                        ),
                        position,
                    ));
                }
            }
            TType::String => {
                self.consume_symbol(LeftSquareBracket)?;
                let position = self.get_current_token_position();
                let start_expr = Box::new(self.expr()?);
                self.consume_symbol(RightSquareBracket)?;

                if start_expr.get_type() != TType::Int {
                    return Err(self.generate_error_with_pos(
                        "String index must be an Int",
                        format!(
                            "Cannot index into String with {}",
                            start_expr.get_type()
                        ),
                        position,
                    ));
                }
                lhs = Expr::Indexed {
                    ttype: TType::Char,
                    name: identifier.clone(),
                    index: start_expr,
                    container: Box::new(lhs),
                    position,
                };
            }
            _ => {
                return Err(self.generate_error(
                    "Cannot index into this type",
                    "Only lists, tuples, and strings can be indexed with `[...]`.\n  Lists: `my_list[i]`\n  Tuples: `my_tuple[0]`, `my_tuple[1]`\n  Strings: `my_string[i]`".to_string(),
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
            format!("Tuple index `{}` is out of bounds", index),
            format!("This tuple has {} element(s), valid indices are 0..{}.", tuple_size, tuple_size.saturating_sub(1)),
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
                            return Err(self.generate_error_with_pos(
                                format!("Incorrect number of arguments: expected {}, got {}", parameters.len(), arguments.len()),
                                format!("This function expects {} argument(s) but {} were provided.", parameters.len(), arguments.len()),
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
                        *return_type = self.typechecker.get_output(
                            *return_type.clone(),
                            &mut type_map,
                            pos,
                        )?;
                        // dbg!(arguments.clone(), return_type.clone(), left_expr.clone());

                        Expr::Call {
                            ttype: *return_type,
                            name: field,
                            function: Box::new(left_expr),
                            args: arguments,
                        }
                    } else {
                        return Err(self.generate_error_with_pos(
                            format!("Cannot call `{}` — it is not a function", left_expr.get_type()),
                            "Expected a callable function type, but found a non-function value.\n  Only functions and closures can be called with `(...)`.",
                            field_position,
                        ));
                    }
                } else {
                    return Err(self.generate_error_with_pos(
                        format!("Cannot get `{}` from `{}`", field, identifier),
                        format!("The identifier `{}` is not defined in the current scope. Check spelling or make sure it is declared before use.", identifier),
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
            if let Some(cap) = self.typechecker.environment.captured.last_mut() {
                cap.insert(
                    identifier.clone(),
                    Symbol {
                        id: identifier.clone(),
                        ttype: ttype.clone(),
                        pos: Some(position.clone()),
                        kind: SymbolKind::Captured,
                    },
                );
            }
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
            let note = if self.typechecker.environment.is_known_function(&identifier) {
                format!(
                    "`{}` is a function, not a variable.\n  \
                     To pass a named function as a value, use the @(Type) syntax:\n    \
                     {}@(ParamType)\n  \
                     For example: {}@(Int) or {}@(Int, String)",
                    identifier, identifier, identifier, identifier,
                )
            } else {
                format!(
                    "The identifier `{}` is not defined.\n  \
                     Check spelling, or make sure it is declared before this point.",
                    identifier,
                )
            };
            Err(self.generate_error_with_pos(
                format!("Undefined symbol `{}`", identifier),
                note,
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
            if let Some(cap) = self.typechecker.environment.captured.last_mut() {
                cap.insert(
                    identifier.clone(),
                    Symbol {
                        id: identifier.clone(),
                        ttype: ttype.clone(),
                        pos: Some(position.clone()),
                        kind: SymbolKind::Captured,
                    },
                );
            }
            self.typechecker.environment.insert_symbol(
                &identifier,
                ttype.clone(),
                Some(position.clone()),
                kind,
            );
            Ok(self.create_literal_expr(identifier.clone(), ttype.clone()))
        } else {
            let note = if self.typechecker.environment.is_known_function(&identifier) {
                format!(
                    "`{}` is a function, not a variable.\n  \
                     To pass a named function as a value, use the @(Type) syntax:\n    \
                     {}@(ParamType)\n  \
                     For example: {}@(Int) or {}@(Int, String)",
                    identifier, identifier, identifier, identifier,
                )
            } else {
                format!(
                    "The identifier `{}` is not defined in the current or enclosing scope.\n  \
                     Check spelling, or make sure it is declared before this point.",
                    identifier,
                )
            };
            Err(self.generate_error_with_pos(
                format!("Undefined symbol `{}`", identifier),
                note,
                position,
            ))
        }
    }

    fn check_depth(&mut self) -> NovaResult<()> {
        const MAX_DEPTH: usize = 64;
        if self.depth > MAX_DEPTH {
            return Err(self.generate_error(
                "Expression too deeply nested",
                format!("Exceeded maximum nesting depth of {MAX_DEPTH}.\n  Simplify the expression or break it into smaller parts."),
            ));
        }
        Ok(())
    }

    fn factor(&mut self) -> NovaResult<Expr> {
        self.depth += 1;
        self.check_depth()?;
        let result = self.factor_inner();
        self.depth -= 1;
        result
    }

    fn factor_inner(&mut self) -> NovaResult<Expr> {
        let mut left: Expr;
        if let Ok(Some(sign)) = self.sign() {
            self.advance();
            let factor = self.factor()?;
            // make sure not sign only works on bools
            if sign == Unary::Not {
                if factor.get_type() != TType::Bool {
                    return Err(self.generate_error(
                        format!("Cannot use `!` on type `{}`", factor.get_type()),
                        "The `!` (not) operator only works on Bool values.\n  Example: `!true`, `!(x > 5)`",
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
                        format!("Condition must be a Bool, found `{}`", condition.get_type()),
                        "The condition in an inline `if` expression must be a Bool.\n  Example: `if x > 0 { x } else { -x }`",
                    ));
                }
                let if_branch = self.block_expr()?;
                self.consume_identifier(Some("else"))?;
                let else_branch = self.block_expr()?;
                let if_type = if if_branch.get_type() == else_branch.get_type() {
                    if_branch.get_type()
                } else {
                    return Err(self.generate_error_with_pos(
                        "Both branches of inline `if` must return the same type".to_string(),
                        format!(
                            "The `if` branch returns `{}` but the `else` branch returns `{}`.\n  Both must be the same type since this is used as an expression.",
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
            // match expression: match expr { Variant() => expr, ... }
            Some(Identifier(id)) if "match" == id.deref() => {
                left = self.match_expr()?;
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
                                format!("Parameter name `{}` conflicts with an existing function", &identifier),
                                format!("A function named `{}` already exists in scope.\n  Use a different parameter name to avoid shadowing.", &identifier),
                                pos.clone(),
                            ));
                        }
                        // check if normal function exist
                        if self.typechecker.environment.has(&identifier) {
                            return Err(self.generate_error_with_pos(
                                format!("Parameter name `{}` conflicts with an existing function", &identifier),
                                format!("A function named `{}` already exists in scope.\n  Use a different parameter name to avoid shadowing.", &identifier),
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
                    .map(|cap| cap.iter().map(|v| v.0.clone()).collect())
                    .unwrap_or_default();

                self.typechecker.environment.pop_scope();

                for c in captured.iter() {
                    if let Some(mc) = self.typechecker.environment.get_type_capture(&c.clone()) {
                        let pos = self.get_current_token_position();
                        if let Some(cap) = self.typechecker.environment.captured.last_mut() {
                            cap.insert(
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
                }

                captured = self
                    .typechecker
                    .environment
                    .captured
                    .last()
                    .map(|cap| cap.iter().map(|v| v.0.clone()).collect())
                    .unwrap_or_default();

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
                        "Closure must return a value",
                        "The last statement in this closure is not a return.\n  Make sure the closure body returns a value matching the declared return type.",
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
                                format!("List comprehension source must be a list, found `{}`", listexpr.get_type()),
                                "The expression after `in` must be a list.\n  Example: `[x in my_list | x * 2]`",
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
                                    format!("List comprehension source must be a list, found `{}`", listexpr.get_type()),
                                    "The expression after `in` must be a list.\n  Example: `[x in my_list | x * 2]`",
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
                                "List comprehension body must return a value",
                                "The output expression returns Void.\n  Make sure the expression after `|` produces a value.\n  Example: `[x in my_list | x * 2]`",
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
                                    format!("List comprehension guard must be a Bool, found `{}`", guard.get_type()),
                                    "Guard expressions filter elements and must return a Bool.\n  Example: `[x in my_list | x * 2 | x > 0]`",
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
                        "Tuple must contain at least one element",
                        "An empty `()` is not a valid expression.\n  For a single-element tuple, use: `(value,)`\n  For a multi-element tuple, use: `(a, b, c)`",
                    ));
                } else {
                    let expr = self.expr()?;
                    if expr.get_type() == TType::None {
                        return Err(self.generate_error(
                            "Tuple element cannot be None/Void",
                            "Each element in a tuple must have a concrete type.\n  Make sure every expression inside `(...)` returns a value.",
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
                                    "Tuple element cannot be None/Void",
                                    "Each element in a tuple must have a concrete type.\n  Make sure every expression inside `(...)` returns a value.",
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
                // ── Detect common wrong-language identifiers used as expressions ──
                match identifier.as_ref() {
                    "True" | "False" => {
                        let correct = if identifier.as_ref() == "True" { "true" } else { "false" };
                        return Err(self.generate_error_with_pos(
                            format!("Unknown identifier `{}`", identifier),
                            format!(
                                "Boolean literals in Nova are lowercase: `true` and `false` (not `{}`).\n  \
                                 Did you mean `{}`?",
                                identifier, correct
                            ),
                            pos,
                        ));
                    }
                    "null" | "nil" | "none" | "undefined" => {
                        return Err(self.generate_error_with_pos(
                            format!("Unknown identifier `{}`", identifier),
                            format!(
                                "Nova uses `None(T)` to represent the absence of a value (there is no `{}`).\n  \
                                 Example: `let x: Option(Int) = None(Int)`\n  \
                                 Note: `None` (capital N) requires a type parameter in parentheses.",
                                identifier
                            ),
                            pos,
                        ));
                    }
                    "this" => {
                        return Err(self.generate_error_with_pos(
                            "Unknown identifier `this`",
                            "Nova does not use `this`. Methods receive the instance as an explicit first parameter.\n  \
                             Example: `fn extends greet(p: Person) -> String { return p.name }`\n  \
                             Use the first parameter name instead of `this`.",
                            pos,
                        ));
                    }
                    _ => {}
                }
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
                return Err(self.generate_error("Unexpected end of file", "An expression was expected but the file ended.\n  Check for missing closing braces `}`, brackets `]`, or parentheses `)`."));
            }
            Some(Operator(Operator::Assignment)) => {
                return Err(self.generate_error(
                    "Unexpected `=` — did you mean `==`?",
                    "A single `=` is assignment, not comparison.\n  \
                     For equality comparison, use `==`.\n  \
                     Example: `if x == 5 { ... }`",
                ));
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
                    // Check that pipe target is actually called with ()
                    if !self.current_token().is_some_and(|t| t.is_symbol(LeftParen)) {
                        return Err(self.generate_error_with_pos(
                            format!("Pipe `|>` requires a function call — add `()` after `{}`", identifier),
                            format!(
                                "The pipe operator passes the left-hand value as the first argument\n  \
                                 to the function on the right, but the function must be called with `()`.\n  \
                                 Example: `value |> {}()`\n  \
                                 Not:     `value |> {}`",
                                identifier, identifier
                            ),
                            pos.clone(),
                        ));
                    }
                    left = self.call(identifier, pos, Some(left))?;
                }
                Some(StructuralSymbol(QuestionMark)) => {
                    let pos = self.get_current_token_position();
                    self.consume_symbol(QuestionMark)?;
                    // ? is syntax sugar for Option::unwrap
                    if let TType::Option { inner } = left.get_type() {
                        left = Expr::Literal {
                            ttype: *inner,
                            value: Atom::Call {
                                name: "Option::unwrap".into(),
                                arguments: vec![left],
                                position: pos,
                            },
                        };
                    } else {
                        return Err(self.generate_error_with_pos(
                            format!(
                                "The `?` operator requires an Option type, found `{}`",
                                left.get_type()
                            ),
                            "The `?` operator is shorthand for `.unwrap()` and can only be used on Option values.\n  \
                             Example: `readFile(\"data.txt\")?` unwraps the Option(String) to String.",
                            pos,
                        ));
                    }
                }
                _ => {
                    // Detect `tuple.0` pattern: `.N` is lexed as float (e.g. .0→0.0, .1→0.1),
                    // not as dot + int
                    if let TType::Tuple { .. } = left.get_type() {
                        if let Some(Float(f)) = self.current_token_value() {
                            // Any float following a tuple is likely `tuple.N` syntax
                            // since `.N` is lexed as a float literal
                            if *f >= 0.0 && *f < 100.0 {
                                // Try to recover the intended index
                                // .0 → 0.0, .1 → 0.1, .2 → 0.2, etc.
                                let idx = if *f < 1.0 {
                                    // .0 → 0.0, .1 → 0.1, .2 → 0.2
                                    (*f * 10.0).round() as usize
                                } else {
                                    // Whole floats like when user somehow gets 1.0, 2.0
                                    *f as usize
                                };
                                let pos = self.get_current_token_position();
                                return Err(self.generate_error_with_pos(
                                    format!("Cannot use dot-index on a tuple — use `[{idx}]` instead"),
                                    format!(
                                        "Nova does not support dot-index syntax for tuples.\n  \
                                         Use bracket indexing: `my_tuple[{idx}]`"
                                    ),
                                    pos,
                                ));
                            }
                        }
                    }
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
                        format!("Parameter name `{}` conflicts with an existing function", &identifier),
                        format!("A function named `{}` already exists in scope.\n  Use a different parameter name to avoid shadowing.", &identifier),
                        pos.clone(),
                    ));
                }
                // check if normal function exist
                if self.typechecker.environment.has(&identifier) {
                    return Err(self.generate_error_with_pos(
                        format!("Parameter name `{}` conflicts with an existing function", &identifier),
                        format!("A function named `{}` already exists in scope.\n  Use a different parameter name to avoid shadowing.", &identifier),
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
        // Detect `|| -> Type { ... }` pattern and give helpful error
        if self.current_token().is_some_and(|t| t.is_op(Operator::RightArrow)) {
            return Err(self.generate_error_with_pos(
                "Closure with `||` cannot have a return type annotation",
                "The `||` closure syntax does not support `-> Type` annotations.\n  \
                 Use the `fn()` form for typed zero-argument closures:\n  \
                 Example: `fn() -> Int { return 42 }`\n  \
                 Or for closures with arguments: `fn(x: Int) -> Int { return x + 1 }`",
                pos.clone(),
            ));
        }
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
                    format!("Incorrect number of arguments: expected {}, got {}", parameters.len(), arguments.len()),
                    format!("This function expects {} argument(s) but {} were provided.", parameters.len(), arguments.len()),
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
            *return_type = self.typechecker.get_output(
                *return_type.clone(),
                &mut type_map,
                pos,
            )?;
            Ok(Expr::Call {
                ttype: *return_type,
                name: function_name,
                function: Box::new(function_expr),
                args: arguments,
            })
        } else {
            Err(self.generate_error_with_pos(
                format!("Cannot call `{}` — it is not a function", function_expr.get_type()),
                format!("Expected a callable function type, but found `{}`.\n  Only function values can be called with `(...)`.", function_expr.get_type()),
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
                                    format!("Unsupported operator `{}` for types `{}` and `{}`", operation, left_expr.get_type(), right_expr.get_type()),
                                    format!("The operator `{}` is not defined for `{}` and `{}`.\n  For custom types, define a dunder method like `fn extends __mul__(a: MyType, b: MyType) -> MyType {{ ... }}`", operation, left_expr.get_type(), right_expr.get_type()),
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
                                        "No matching operator overload found",
                                        "The operator is not defined for these types.\n  Define a matching dunder method (e.g. __add__, __eq__) for the types involved.",
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
                                        "No matching operator overload found",
                                        "The operator is not defined for these types.\n  Define a matching dunder method (e.g. __add__, __eq__) for the types involved.",
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
        self.depth += 1;
        self.check_depth()?;
        let result = self.expr_inner();
        self.depth -= 1;
        result
    }

    fn expr_inner(&mut self) -> NovaResult<Expr> {
        match self.current_token_value() {
            Some(Identifier(id)) if "let" == id.deref() => {
                return self.let_expr();
            }
            _ => {}
        }

        // Bare `_` is not a valid identifier — it's only valid in `let _ = expr`.
        // Reject it early with a clear message.
        if let Some(Identifier(id)) = self.current_token_value() {
            if id.deref() == "_" {
                return Err(self.generate_error(
                    "`_` can only be used with `let` to discard a value",
                    "`_` is not a variable — it is a discard pattern.\n  \
                     Use `let _ = expr` to evaluate an expression and throw away the result.",
                ));
            }
        }

        let mut left_expr = self.logical_or_expr()?;
        let current_pos = self.get_current_token_position();
        while self.current_token().is_some_and(|t| t.is_assign()) {
            if let Some(operation) = self.current_token().and_then(|t| t.get_operator()) {
                self.advance();
                let right_expr = self.logical_or_expr()?;
                match left_expr.clone() {
                    Expr::ListConstructor { .. }
                    | Expr::Binop { .. }
                    | Expr::Call { .. }
                    | Expr::Unary { .. }
                    | Expr::Closure { .. }
                    | Expr::None => {
                        let kind = match &left_expr {
                            Expr::ListConstructor { .. } => "a list constructor",
                            Expr::Binop { .. } => "a binary expression",
                            Expr::Call { .. } => "a function call",
                            Expr::Unary { .. } => "a unary expression",
                            Expr::Closure { .. } => "a closure",
                            Expr::None => "None",
                            _ => "this expression",
                        };
                        return Err(self.generate_error_with_pos(
                            "left hand side of `=` must be assignable",
                            format!("{kind} is not assignable"),
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
                                    "Cannot assign `{}` to `{}`",
                                    right_expr.get_type(),
                                    left_expr.get_type()
                                ),
                                "The left-hand side of `=` must be a variable, not a literal value.\n  Use `let name = value` to create a new variable.",
                                current_pos.clone(),
                            ));
                        }
                    },
                    _ => {
                        if right_expr.get_type() != left_expr.get_type() {
                            return Err(self.generate_error_with_pos(
                                format!(
                                    "Cannot assign `{}` to `{}`",
                                    right_expr.get_type(),
                                    left_expr.get_type()
                                ),
                                format!("Type mismatch: the variable has type `{}` but the expression returns `{}`.", left_expr.get_type(), right_expr.get_type()),
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

        while let Some(Operator(Operator::RightTilde)) = self.current_token_value() {
            // the syntax is expr ~> id { statements }
            self.consume_operator(Operator::RightTilde)?;
            let (identifier, pos) = self.get_identifier()?;

            // if current token is { else its expr,
            match self.current_token_value() {
                Some(StructuralSymbol(LeftBrace)) => {
                    // cant assign a void
                    if left_expr.get_type() == TType::Void {
                        return Err(self.generate_error_with_pos(
                            format!("Variable `{}` cannot be assigned to Void", identifier),
                            "The expression on the left of `~>` does not return a value (returns Void).\n  Make sure the expression produces a value.\n  Syntax: `expr ~> name { ... }`",
                            pos.clone(),
                        ));
                    }

                    if self.typechecker.environment.has(&identifier) {
                        return Err(self.generate_error_with_pos(
                            format!("Variable `{}` is already defined in this scope", identifier),
                            format!("`{}` already exists. Choose a different name for the `~>` binding.", identifier),
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
                        "Cannot compare Void values",
                        "One or both sides of the comparison do not return a value (Void).\n  Make sure both sides are expressions that produce a value.",
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
                                            "No matching operator overload found",
                                            "The operator is not defined for these types.\n  Define a matching dunder method (e.g. __add__, __eq__) for the types involved.",
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
                                                "No matching operator overload found",
                                                "The operator is not defined for these types.\n  Define a matching dunder method (e.g. __add__, __eq__) for the types involved.",
                                            ))
                                        }
                                    };
                                    // check if return type is bool
                                    if returntype != TType::Bool {
                                        return Err(self.generate_error_with_pos(
                                            "Comparison operator must return Bool",
                                            format!(
                                                "The dunder method for this comparison returned `{}` instead of `Bool`. Make sure the operator overload returns Bool.",
                                                returntype,
                                                
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
                                                "No matching operator overload found",
                                                "The operator is not defined for these types.\n  Define a matching dunder method (e.g. __add__, __eq__) for the types involved.",
                                            ))
                                        }
                                    };
                                    // check if return type is bool
                                    if returntype != TType::Bool {
                                        return Err(self.generate_error_with_pos(
                                            "Comparison operator must return Bool",
                                            format!(
                                                "The dunder method for this comparison returned `{}` instead of `Bool`. Make sure the operator overload returns Bool.",
                                                returntype,
                                                
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
                                        "No matching operator overload found",
                                        "The operator is not defined for these types.\n  Define a matching dunder method (e.g. __add__, __eq__) for the types involved.",
                                    ))
                                }
                            };
                            // check if return type is bool
                            if returntype != TType::Bool {
                                return Err(self.generate_error_with_pos(
                                    "Comparison operator must return Bool",
                                    format!(
                                        "The dunder method for this comparison returned `{}` instead of `Bool`. Make sure the operator overload returns Bool.",
                                        returntype,
                                        
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
                                        "No matching operator overload found",
                                        "The operator is not defined for these types.\n  Define a matching dunder method (e.g. __add__, __eq__) for the types involved.",
                                    ))
                                }
                            };
                            // check if return type is bool
                            if returntype != TType::Bool {
                                return Err(self.generate_error_with_pos(
                                    "Comparison operator must return Bool",
                                    format!(
                                        "The dunder method for this comparison returned `{}` instead of `Bool`. Make sure the operator overload returns Bool.",
                                        returntype,
                                        
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

    // ── logical_or_expr: handles `||` (lower precedence) ──
    fn logical_or_expr(&mut self) -> NovaResult<Expr> {
        let mut left_expr = self.logical_and_expr()?;
        let current_pos = self.get_current_token_position();
        while self.current_token().is_some_and(|t| t.is_logical_or()) {
            if let Some(operation) = self.current_token().and_then(|t| t.get_operator()) {
                self.advance();
                let right_expr = self.logical_and_expr()?;
                left_expr =
                    self.build_logical_expr(left_expr, right_expr, operation, &current_pos)?;
            }
        }
        Ok(left_expr)
    }

    // ── logical_and_expr: handles `&&` (higher precedence than `||`) ──
    fn logical_and_expr(&mut self) -> NovaResult<Expr> {
        let mut left_expr = self.top_expr()?;
        let current_pos = self.get_current_token_position();
        while self.current_token().is_some_and(|t| t.is_logical_and()) {
            if let Some(operation) = self.current_token().and_then(|t| t.get_operator()) {
                self.advance();
                let right_expr = self.top_expr()?;
                left_expr =
                    self.build_logical_expr(left_expr, right_expr, operation, &current_pos)?;
            }
        }
        Ok(left_expr)
    }

    // ── shared logic for && and || (dunder overloads, type checking) ──
    fn build_logical_expr(
        &mut self,
        left_expr: Expr,
        right_expr: Expr,
        operation: Operator,
        current_pos: &FilePosition,
    ) -> NovaResult<Expr> {
        // check if void
        if left_expr.get_type() == TType::Void || right_expr.get_type() == TType::Void {
            return Err(self.generate_error_with_pos(
                "Cannot use logical operators on Void values",
                "One or both sides of `&&`/`||` do not return a value (Void).\n  Make sure both sides are expressions that produce a Bool.",
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
                                "No matching operator overload found",
                                "The operator is not defined for these types.\n  Define a matching dunder method (e.g. __add__, __eq__) for the types involved.",
                            ))
                        }
                    };

                    if let Some(overload) =
                        self.typechecker.environment.get(&generate_unique_string(
                            &function_id,
                            &[left_expr.get_type(), right_expr.get_type()],
                        ))
                    {
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
                                    "No matching operator overload found",
                                    "The operator is not defined for these types.\n  Define a matching dunder method (e.g. __add__, __eq__) for the types involved.",
                                ))
                            }
                        };
                        if returntype != TType::Bool {
                            return Err(self.generate_error_with_pos(
                                "Comparison operator must return Bool",
                                format!(
                                    "The dunder method for this comparison returned `{}` instead of `Bool`. Make sure the operator overload returns Bool.",
                                    returntype,
                                ),
                                current_pos.clone(),
                            ));
                        }
                        Ok(Expr::Literal {
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
                        })
                    } else if let Some(overload) =
                        self.typechecker.environment.get(&function_id)
                    {
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
                                    "No matching operator overload found",
                                    "The operator is not defined for these types.\n  Define a matching dunder method (e.g. __add__, __eq__) for the types involved.",
                                ))
                            }
                        };
                        if returntype != TType::Bool {
                            return Err(self.generate_error_with_pos(
                                "Comparison operator must return Bool",
                                format!(
                                    "The dunder method for this comparison returned `{}` instead of `Bool`. Make sure the operator overload returns Bool.",
                                    returntype,
                                ),
                                current_pos.clone(),
                            ));
                        }
                        Ok(Expr::Literal {
                            ttype: TType::Bool,
                            value: Atom::Call {
                                name: function_id.into(),
                                arguments,
                                position: pos.clone(),
                            },
                        })
                    } else {
                        Ok(self.create_binop_expr(
                            left_expr,
                            right_expr,
                            operation,
                            TType::Bool,
                        ))
                    }
                } else {
                    Ok(self.create_binop_expr(
                        left_expr,
                        right_expr,
                        operation,
                        TType::Bool,
                    ))
                }
            }
            _ => Ok(left_expr),
        }
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
                                        "No matching operator overload found",
                                        "The operator is not defined for these types.\n  Define a matching dunder method (e.g. __add__, __eq__) for the types involved.",
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
                                        "No matching operator overload found",
                                        "The operator is not defined for these types.\n  Define a matching dunder method (e.g. __add__, __eq__) for the types involved.",
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
                                format!("No operator overload `{}` found for `{}` and `{}`", operation, left_expr.get_type(), right_expr.get_type()),
                                format!("Define a dunder method to support this operation.\n  Example: `fn extends {}(a: {}, b: {}) -> {} {{ ... }}`", function_id.split("::").last().unwrap_or(&function_id), left_expr.get_type(), right_expr.get_type(), left_expr.get_type()),
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
                        "Tuple type must contain at least two elements",
                        "A tuple type requires at least two elements, e.g. `(Int, String)`.\n  A single-element parenthesized type like `(Int)` is just `Int`.",
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
                        "Cannot nest Option directly inside Option",
                        "Nested `Option(Option(...))` is not allowed.\n  Use a single `Option(T)` instead.",
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
            Some(Identifier(id)) if "List" == id.deref() => {
                self.advance();
                self.consume_symbol(LeftParen)?;
                let inner = self.ttype()?;
                self.consume_symbol(RightParen)?;
                Ok(TType::List {
                    inner: Box::new(inner),
                })
            }
            Some(Identifier(id)) if "Tuple" == id.deref() => {
                self.advance();
                self.consume_symbol(LeftParen)?;
                let mut elements = vec![self.ttype()?];
                while self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
                    self.consume_symbol(Comma)?;
                    if self
                        .current_token()
                        .is_some_and(|t| t.is_symbol(RightParen))
                    {
                        break;
                    }
                    elements.push(self.ttype()?);
                }
                self.consume_symbol(RightParen)?;
                if elements.len() < 2 {
                    return Err(self.generate_error(
                        "Tuple type must contain at least two elements",
                        "A tuple type requires at least two elements, e.g. `Tuple(Int, String)`.\n  For a single-element tuple, use: `(Int,)`",
                    ));
                }
                Ok(TType::Tuple { elements })
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
                                format!("Expected {} type parameter(s) for `{}`", generic_len.len(), identifier),
                                format!("Got {} type parameter(s), but `{}` requires {}.\n  Example: `{}({})`",
                                    type_annotation.len(),
                                    identifier,
                                    generic_len.len(),
                                    identifier,
                                    generic_len.iter().map(|g| g.to_string()).collect::<Vec<_>>().join(", ")
                                ),
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
                            format!("Unknown type `{}`", identifier),
                            format!("The type `{identifier}` is not defined.\n  Check spelling and make sure it is declared before use.\n  Built-in types: Int, Float, Bool, String, Char, Any, None\n  Custom types must be declared with `enum` or `struct` before use."),
                            pos,
                        ));
                    };
                    Ok(alias.clone())
                }
            }
            _ => Err(self.generate_error(
                "Expected type annotation",
                format!(
                    "Got `{}` but expected a type name.\n  Valid types: Int, Float, Bool, String, Char, Option(T), [T], (T1, T2), fn(T) -> R, or a custom type name",
                    fmt_token_opt(self.current_token())
                ),
            )),
        }
    }

    fn get_identifier(&mut self) -> NovaResult<(Rc<str>, FilePosition)> {
        let identifier = match self.current_token_value() {
            Some(Identifier(id)) => id.clone(),
            Some(Keyword(kw)) => {
                return Err(self.generate_error(
                    format!("Expected identifier, found keyword `{}`", kw),
                    format!(
                        "`{}` is a reserved keyword and cannot be used as a name.\n  \
                         Choose a different name for this identifier.",
                        kw
                    ),
                ));
            }
            Some(Integer(_)) => {
                return Err(self.generate_error(
                    "Expected identifier, found a number",
                    "Identifiers must start with a letter or underscore, not a digit.\n  \
                     Example: `my_var`, `count`, `_temp`",
                ));
            }
            Some(StructuralSymbol(sym)) => {
                return Err(self.generate_error(
                    format!("Expected identifier, found `{}`", sym),
                    format!(
                        "A name was expected here but got `{}`.\n  \
                         Check for missing identifiers, extra punctuation, or unclosed brackets.",
                        sym
                    ),
                ));
            }
            Some(Operator(op)) => {
                return Err(self.generate_error(
                    format!("Expected identifier, found `{}`", op),
                    format!(
                        "A name was expected here but got `{}`.\n  \
                         Check for missing identifiers or extra operators.",
                        op
                    ),
                ));
            }
            None => {
                return Err(self.generate_error(
                    "Expected identifier, but reached end of file",
                    "The file ended unexpectedly. Check for missing closing braces `}}`, brackets `]`, or parentheses `)`.",
                ));
            }
            _ => {
                return Err(self.generate_error(
                    "Expected identifier",
                    format!(
                        "got {} but expected an identifier (a name like `x`, `my_func`, `Point`, etc.)",
                        fmt_token_opt(self.current_token())
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
                    "Duplicate parameter name",
                    "Each parameter must have a unique name. Choose a different name for this parameter.",
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
                    "Duplicate parameter name",
                    "Each parameter must have a unique name. Choose a different name for this parameter.",
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
        let test = self.logical_or_expr()?;
        let pos = self.get_current_token_position();
        if test.get_type() != TType::Bool {
            return Err(self.generate_error_with_pos(
                format!("Condition must be a Bool, found `{}`", test.get_type()),
                format!(
                    "The condition expression returned `{}` but `if`/`elif` requires a Bool.\n  Use a comparison like `x > 0`, `x == 0`, `x != \"\"`, etc.",
                    test.get_type()
                ),
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
            let else_pos = self.get_current_token_position();
            self.advance();
            if self.current_token().is_some_and(|t| t.is_id("if")) {
                return Err(self.generate_error_with_pos(
                    "Unexpected `else if` — Nova uses `elif`",
                    "Nova does not support `else if`. Use `elif` instead.\n  Example: `elif condition { ... }`",
                    else_pos,
                ));
            }
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

        // ── GitHub import: `import @ "owner/repo/path/file.nv" ! "commit"` ──
        if self.current_token().is_some_and(|t| t.is_symbol(At)) {
            self.advance(); // consume @
            let repo_path = match self.current_token_value() {
                Some(StringLiteral(s)) => s.to_string(),
                _ => {
                    return Err(self.generate_error_with_pos(
                        "Expected GitHub path after `@`",
                        "expected a string literal like `\"owner/repo/path/to/file.nv\"`\n  \
                         Example: import @ \"pyrotek45/nova-lang/std/core.nv\"",
                        self.get_current_token_position(),
                    ));
                }
            };
            self.advance();

            // Optional commit lock: ! "commithash"
            let commit = if self.current_token().is_some_and(|t| t.is_op(Operator::Not)) {
                self.advance(); // consume !
                let hash = match self.current_token_value() {
                    Some(StringLiteral(s)) => s.to_string(),
                    _ => {
                        return Err(self.generate_error_with_pos(
                            "Expected commit hash after `!`",
                            "expected a string literal like `\"abc123\"` for version locking\n  \
                             Example: import @ \"pyrotek45/nova-lang/std/core.nv\" ! \"a1b2c3d\"",
                            self.get_current_token_position(),
                        ));
                    }
                };
                self.advance();
                Some(hash)
            } else {
                None
            };

            // Parse the repo path: "owner/repo/path/to/file.nv"
            let parts: Vec<&str> = repo_path.splitn(3, '/').collect();
            if parts.len() < 3 {
                return Err(self.generate_error_with_pos(
                    "Invalid GitHub path",
                    format!(
                        "expected `\"owner/repo/path/to/file.nv\"`, got `\"{}\"`\n  \
                         The path must have at least three segments: owner, repository, and file path.\n  \
                         Example: `\"pyrotek45/nova-lang/std/core.nv\"`",
                        repo_path
                    ),
                    pos,
                ));
            }
            let owner = parts[0];
            let repo = parts[1];
            let file_path = parts[2];
            let branch = commit.as_deref().unwrap_or("main");

            let url = format!(
                "https://raw.githubusercontent.com/{}/{}/{}/{}",
                owner, repo, branch, file_path
            );

            let source = match ureq::get(&url).call() {
                Ok(resp) => match resp.into_string() {
                    Ok(body) => body,
                    Err(e) => {
                        return Err(self.generate_error_with_pos(
                            "GitHub import: failed to read response",
                            format!(
                                "Could not read the response body from:\n  {}\n  Error: {}",
                                url, e
                            ),
                            pos,
                        ));
                    }
                },
                Err(e) => {
                    let mut hint = if commit.is_some() {
                        format!(
                            "\n  Check that the commit hash `{}` is correct and the file exists at that revision.",
                            branch
                        )
                    } else {
                        "\n  Check that the repository is public and the file path is correct.\n  \
                         Tip: you can lock to a specific commit with `! \"commithash\"`."
                            .to_string()
                    };

                    // Detect common mistake: user included the branch name in the path
                    let common_branches = ["main/", "master/", "dev/", "develop/"];
                    for prefix in common_branches {
                        if let Some(corrected) = file_path.strip_prefix(prefix) {
                            hint.push_str(&format!(
                                "\n\n  It looks like the path contains the branch name `{}`.\n  \
                                 The branch is added automatically — try removing it:\n  \
                                 import @ \"{}/{}/{}\"",
                                &prefix[..prefix.len() - 1],
                                owner,
                                repo,
                                corrected
                            ));
                            break;
                        }
                    }

                    return Err(self.generate_error_with_pos(
                        "GitHub import: could not fetch file",
                        format!(
                            "Failed to fetch from GitHub:\n  {}\n  Error: {}{}",
                            url, e, hint
                        ),
                        pos,
                    ));
                }
            };

            // Use a virtual filepath for error reporting
            let virtual_path: Rc<Path> =
                PathBuf::from(format!("github://{}/{}/{}", owner, repo, file_path)).into();

            let mut lexer = Lexer::new(source.as_str(), Some(&virtual_path));
            let tokens = lexer.tokenize()?;
            let mut parser = self.clone();
            parser.index = 0;
            parser.filepath = Some(virtual_path.clone());
            parser.input = tokens;
            parser.parse()?;
            self.typechecker.environment = parser.typechecker.environment.clone();
            self.modules = parser.modules.clone();
            return Ok(Some(Statement::Block {
                body: parser.ast.program.clone(),
                filepath: Some(virtual_path),
            }));
        }

        // ── Local import: dot-path or string literal ──
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
            _ => {
                return Err(self.generate_error_with_pos(
                    "Expected import path",
                    "expected a module path, string literal, or `@` for GitHub import after 'import'\n  \
                     Examples:\n  \
                     import utils.math          // local: ./utils/math.nv\n  \
                     import super.std.core      // local: ../std/core.nv\n  \
                     import @ \"owner/repo/path/file.nv\"  // GitHub",
                    pos,
                ));
            }
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

        // ── If the resolved path is a github:// virtual path, fetch from GitHub ──
        let resolved_str = resolved_filepath.to_string_lossy();
        if let Some(gh_path) = resolved_str.strip_prefix("github://") {
            // Parse github://owner/repo/path/to/file.nv
            let gh_parts: Vec<&str> = gh_path.splitn(3, '/').collect();
            if gh_parts.len() >= 3 {
                let gh_owner = gh_parts[0];
                let gh_repo = gh_parts[1];
                let gh_file = gh_parts[2];
                let gh_url = format!(
                    "https://raw.githubusercontent.com/{}/{}/main/{}",
                    gh_owner, gh_repo, gh_file
                );

                let gh_source = match ureq::get(&gh_url).call() {
                    Ok(resp) => match resp.into_string() {
                        Ok(body) => body,
                        Err(e) => {
                            return Err(self.generate_error_with_pos(
                                "GitHub import: failed to read response",
                                format!(
                                    "Could not read the response body from:\n  {}\n  Error: {}",
                                    gh_url, e
                                ),
                                pos,
                            ));
                        }
                    },
                    Err(e) => {
                        return Err(self.generate_error_with_pos(
                            "GitHub import: could not fetch file",
                            format!(
                                "Failed to fetch nested import from GitHub:\n  {}\n  Error: {}\n  \
                                 This import was triggered by a GitHub-fetched file.\n  \
                                 Check that the file exists in the repository.",
                                gh_url, e
                            ),
                            pos,
                        ));
                    }
                };

                let mut lexer = Lexer::new(gh_source.as_str(), Some(&resolved_filepath));
                let tokens = lexer.tokenize()?;
                let mut parser = self.clone();
                parser.index = 0;
                parser.filepath = Some(resolved_filepath.clone());
                parser.input = tokens;
                parser.parse()?;
                self.typechecker.environment = parser.typechecker.environment.clone();
                self.modules = parser.modules.clone();
                return Ok(Some(Statement::Block {
                    body: parser.ast.program.clone(),
                    filepath: Some(resolved_filepath),
                }));
            }
        }

        let tokens = Lexer::read_file(&resolved_filepath);
        let tokens = match tokens {
            Ok(tokens) => tokens,
            Err(_) => {
                return Err(self.generate_error_with_pos(
                    "Error Importing file",
                    format!(
                        "Could not find file: {}\n  \
                         Import paths are relative to the current file's directory.\n  \
                         Use `super` to go up a directory: import super.folder.module\n  \
                         Check that the file exists and the path is spelled correctly.",
                        resolved_filepath.display()
                    ),
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

    // ─────────────────────────────────────────────────────────────────
    //  Generalized pattern parsing for non-enum match
    // ─────────────────────────────────────────────────────────────────

    /// Check if a pattern is irrefutable (always matches).
    fn is_irrefutable(pat: &Pattern) -> bool {
        match pat {
            Pattern::Wildcard | Pattern::Variable(_) => true,
            Pattern::Tuple(pats) | Pattern::List(pats) => pats.iter().all(Self::is_irrefutable),
            Pattern::Struct { fields, .. } => {
                // A struct pattern with all irrefutable field sub-patterns
                // covers every possible instance of that struct.
                fields.iter().all(|(_, p)| Self::is_irrefutable(p))
            }
            Pattern::ListCons(heads, _tail) => {
                // [h1, h2, ..rest] is irrefutable if all head elements are
                // irrefutable — the tail variable always binds.
                heads.iter().all(Self::is_irrefutable)
            }
            Pattern::OptionSome(_) => {
                // Some(x) only matches Some, not None – not irrefutable
                false
            }
            Pattern::Or(alts) => alts.iter().any(Self::is_irrefutable),
            _ => false,
        }
    }

    /// Check if a pattern (or any of its Or alternatives) satisfies a predicate.
    fn arm_contains(pat: &Pattern, pred: &dyn Fn(&Pattern) -> bool) -> bool {
        match pat {
            Pattern::Or(alts) => alts.iter().any(|a| pred(a)),
            other => pred(other),
        }
    }

    /// Produce a human-readable string for a pattern (used in error messages).
    fn pattern_to_string(pat: &Pattern) -> String {
        match pat {
            Pattern::Wildcard => "_".to_string(),
            Pattern::Variable(name) => name.to_string(),
            Pattern::IntLiteral(n) => n.to_string(),
            Pattern::FloatLiteral(f) => f.to_string(),
            Pattern::StringLiteral(s) => format!("\"{}\"", s),
            Pattern::BoolLiteral(b) => b.to_string(),
            Pattern::CharLiteral(c) => format!("'{}'", c),
            Pattern::Tuple(pats) => {
                let inner: Vec<String> = pats.iter().map(Self::pattern_to_string).collect();
                format!("({})", inner.join(", "))
            }
            Pattern::List(pats) => {
                let inner: Vec<String> = pats.iter().map(Self::pattern_to_string).collect();
                format!("[{}]", inner.join(", "))
            }
            Pattern::ListCons(heads, tail) => {
                let mut parts: Vec<String> = heads.iter().map(Self::pattern_to_string).collect();
                parts.push(format!("..{}", tail));
                format!("[{}]", parts.join(", "))
            }
            Pattern::EmptyList => "[]".to_string(),
            Pattern::Or(alts) => {
                let inner: Vec<String> = alts.iter().map(Self::pattern_to_string).collect();
                inner.join(" | ")
            }
            Pattern::Enum { variant, binding, .. } => {
                if let Some(b) = binding {
                    format!("{}({})", variant, Self::pattern_to_string(b))
                } else {
                    format!("{}()", variant)
                }
            }
            Pattern::OptionSome(binding) => {
                if let Some(b) = binding {
                    format!("Some({})", Self::pattern_to_string(b))
                } else {
                    "Some()".to_string()
                }
            }
            Pattern::OptionNone => "None()".to_string(),
            Pattern::Struct { name, fields } => {
                let inner: Vec<String> = fields.iter().map(|(fname, pat)| {
                    if matches!(pat, Pattern::Variable(v) if v == fname) {
                        fname.to_string()
                    } else {
                        format!("{}: {}", fname, Self::pattern_to_string(pat))
                    }
                }).collect();
                format!("{} {{ {} }}", name, inner.join(", "))
            }
        }
    }

    /// Validate that a pattern is compatible with the match type.
    /// Returns an error if, e.g., an IntLiteral pattern is used on a String match.
    fn validate_pattern_type(&self, pattern: &Pattern, match_type: &TType, pos: &FilePosition) -> NovaResult<()> {
        match pattern {
            Pattern::Wildcard | Pattern::Variable(_) => Ok(()),
            Pattern::IntLiteral(_) => {
                if *match_type != TType::Int && *match_type != TType::Any {
                    return Err(self.generate_error_with_pos(
                        format!("Int literal pattern cannot match `{}`", match_type),
                        "This arm uses an integer pattern, but the match subject is not an Int.",
                        pos.clone(),
                    ));
                }
                Ok(())
            }
            Pattern::FloatLiteral(_) => {
                if *match_type != TType::Float && *match_type != TType::Any {
                    return Err(self.generate_error_with_pos(
                        format!("Float literal pattern cannot match `{}`", match_type),
                        "This arm uses a float pattern, but the match subject is not a Float.",
                        pos.clone(),
                    ));
                }
                Ok(())
            }
            Pattern::StringLiteral(_) => {
                if *match_type != TType::String && *match_type != TType::Any {
                    return Err(self.generate_error_with_pos(
                        format!("String literal pattern cannot match `{}`", match_type),
                        "This arm uses a string pattern, but the match subject is not a String.",
                        pos.clone(),
                    ));
                }
                Ok(())
            }
            Pattern::BoolLiteral(_) => {
                if *match_type != TType::Bool && *match_type != TType::Any {
                    return Err(self.generate_error_with_pos(
                        format!("Bool literal pattern cannot match `{}`", match_type),
                        "This arm uses a bool pattern, but the match subject is not a Bool.",
                        pos.clone(),
                    ));
                }
                Ok(())
            }
            Pattern::CharLiteral(_) => {
                if *match_type != TType::Char && *match_type != TType::Any {
                    return Err(self.generate_error_with_pos(
                        format!("Char literal pattern cannot match `{}`", match_type),
                        "This arm uses a char pattern, but the match subject is not a Char.",
                        pos.clone(),
                    ));
                }
                Ok(())
            }
            Pattern::EmptyList | Pattern::List(_) | Pattern::ListCons(_, _) => {
                match match_type {
                    TType::List { .. } | TType::Any => Ok(()),
                    _ => Err(self.generate_error_with_pos(
                        format!("List pattern cannot match `{}`", match_type),
                        "This arm uses a list pattern, but the match subject is not a List.",
                        pos.clone(),
                    )),
                }
            }
            Pattern::Tuple(pats) => {
                match match_type {
                    TType::Tuple { elements } => {
                        if pats.len() != elements.len() {
                            return Err(self.generate_error_with_pos(
                                format!(
                                    "Tuple pattern has {} elements but the match type has {}",
                                    pats.len(), elements.len()
                                ),
                                "The number of elements in the tuple pattern must match the tuple type.",
                                pos.clone(),
                            ));
                        }
                        for (i, pat) in pats.iter().enumerate() {
                            self.validate_pattern_type(pat, &elements[i], pos)?;
                        }
                        Ok(())
                    }
                    TType::Any => Ok(()),
                    _ => Err(self.generate_error_with_pos(
                        format!("Tuple pattern cannot match `{}`", match_type),
                        "This arm uses a tuple pattern, but the match subject is not a Tuple.",
                        pos.clone(),
                    )),
                }
            }
            Pattern::Or(alternatives) => {
                for alt in alternatives {
                    self.validate_pattern_type(alt, match_type, pos)?;
                }
                Ok(())
            }
            Pattern::Enum { variant, .. } => {
                match match_type {
                    TType::Option { .. } => {
                        // Only Some and None are valid
                        if variant.as_ref() != "Some" && variant.as_ref() != "None" {
                            return Err(self.generate_error_with_pos(
                                format!("Unknown Option variant `{}`", variant),
                                "Option only has `Some(x)` and `None()` variants.",
                                pos.clone(),
                            ));
                        }
                        Ok(())
                    }
                    TType::Custom { name, .. } => {
                        // Check that the variant exists in the enum definition
                        if let Some(fields) = self.typechecker.environment.custom_types.get(name.as_ref()) {
                            let found = fields.iter().any(|(n, _)| n.as_ref() == variant.as_ref());
                            if !found {
                                let available: Vec<String> = fields.iter()
                                    .filter(|(n, _)| n.as_ref() != "type")
                                    .map(|(n, _)| format!("`{}`", n))
                                    .collect();
                                return Err(self.generate_error_with_pos(
                                    format!("Variant `{}` not found in `{}`", variant, name),
                                    format!("Available variants: {}", available.join(", ")),
                                    pos.clone(),
                                ));
                            }
                        }
                        Ok(())
                    }
                    TType::Any => Ok(()),
                    _ => Err(self.generate_error_with_pos(
                        format!("Enum pattern `{}(...)` cannot match `{}`", variant, match_type),
                        "Enum/variant patterns require the match subject to be an enum or Option type.",
                        pos.clone(),
                    )),
                }
            }
            Pattern::Struct { name, fields } => {
                match match_type {
                    TType::Custom { name: type_name, .. } => {
                        if name != type_name {
                            return Err(self.generate_error_with_pos(
                                format!("Struct pattern `{}` does not match type `{}`", name, type_name),
                                "The struct name in the pattern must match the match subject type.",
                                pos.clone(),
                            ));
                        }
                        // Validate field names and count
                        if let Some(struct_fields) = self.typechecker.environment.custom_types.get(name.as_ref()) {
                            // Filter out the auto-added "type" field
                            let real_fields: Vec<&(Rc<str>, TType)> = struct_fields.iter()
                                .filter(|(n, _)| n.as_ref() != "type")
                                .collect();
                            // Check for `_` wildcard sentinel (discards remaining fields)
                            let has_wildcard = fields.iter().any(|(n, p)| n.as_ref() == "_" && *p == Pattern::Wildcard);
                            let explicit_fields: Vec<&(Rc<str>, Pattern)> = fields.iter()
                                .filter(|(n, p)| !(n.as_ref() == "_" && *p == Pattern::Wildcard))
                                .collect();
                            if has_wildcard {
                                // With `_`, explicit fields must be fewer than struct fields
                                if explicit_fields.len() >= real_fields.len() {
                                    return Err(self.generate_error_with_pos(
                                        format!(
                                            "Struct pattern `{}` uses `_` but already names all {} fields",
                                            name, real_fields.len()
                                        ),
                                        "`_` discards the remaining fields, but all fields are already named.",
                                        pos.clone(),
                                    ));
                                }
                            } else if fields.len() != real_fields.len() {
                                return Err(self.generate_error_with_pos(
                                    format!(
                                        "Struct pattern `{}` has {} fields but the struct has {}",
                                        name, fields.len(), real_fields.len()
                                    ),
                                    "The number of fields in the pattern must match the struct definition.\n  \
                                     Use `_` to discard unnamed fields: `Struct { field1, _ }`",
                                    pos.clone(),
                                ));
                            }
                            for (field_name, sub_pat) in &explicit_fields {
                                let found = real_fields.iter().find(|(n, _)| n == field_name);
                                if found.is_none() {
                                    let available: Vec<String> = real_fields.iter()
                                        .map(|(n, _)| format!("`{}`", n))
                                        .collect();
                                    return Err(self.generate_error_with_pos(
                                        format!("Field `{}` not found in struct `{}`", field_name, name),
                                        format!("Available fields: {}", available.join(", ")),
                                        pos.clone(),
                                    ));
                                }
                                // Validate sub-pattern type
                                let field_ty = found.map(|(_, t)| t.clone()).unwrap_or(TType::Any);
                                self.validate_pattern_type(sub_pat, &field_ty, pos)?;
                            }
                        }
                        Ok(())
                    }
                    TType::Any => Ok(()),
                    _ => Err(self.generate_error_with_pos(
                        format!("Struct pattern cannot match `{}`", match_type),
                        "Struct patterns require the match subject to be a struct type.",
                        pos.clone(),
                    )),
                }
            }
            Pattern::OptionSome(_) | Pattern::OptionNone => {
                match match_type {
                    TType::Option { .. } | TType::Any => Ok(()),
                    _ => Err(self.generate_error_with_pos(
                        format!("Option pattern cannot match `{}`", match_type),
                        "Some/None patterns require the match subject to be an Option type.",
                        pos.clone(),
                    )),
                }
            }
        }
    }

    /// Parse a single pattern.
    ///
    /// Patterns:
    ///   _                     → Wildcard
    ///   42 / -3               → IntLiteral
    ///   3.14 / -1.0           → FloatLiteral
    ///   "hello"               → StringLiteral
    ///   'a'                   → CharLiteral
    ///   true / false          → BoolLiteral
    ///   []                    → EmptyList
    ///   [p1, p2, ..]          → List(pats)
    ///   [p1, p2, ..rest]      → ListCons(heads, rest)
    ///   (p1, p2, ..)          → Tuple(pats)
    ///   identifier            → Variable(name)
    ///   Variant(pat)          → Enum { variant, binding }
    ///   Struct { f1: p, f2 }  → Struct { name, fields }
    ///   pat1 | pat2           → Or([pat1, pat2])
    fn parse_pattern(&mut self) -> NovaResult<Pattern> {
        let pat = self.parse_single_pattern()?;
        // Check for | (OR patterns)
        if self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::Pipe)) {
            let mut alternatives = vec![pat];
            while self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::Pipe)) {
                self.advance(); // consume |
                alternatives.push(self.parse_single_pattern()?);
            }
            return Ok(Pattern::Or(alternatives));
        }
        Ok(pat)
    }

    /// When matching on Option, transform `Enum { variant: "Some", binding }` → OptionSome(binding)
    /// and `Enum { variant: "None", binding: None }` → OptionNone.
    fn resolve_option_patterns(&self, pat: Pattern, match_type: &TType) -> Pattern {
        match pat {
            // Top-level Option resolution
            Pattern::Enum { ref variant, ref binding, .. } if variant.as_ref() == "Some" && matches!(match_type, TType::Option { .. }) => {
                // Recursively resolve the binding sub-pattern with the inner Option type
                let inner_type = match match_type {
                    TType::Option { inner } => inner.as_ref().clone(),
                    _ => TType::Any,
                };
                let resolved_binding = binding.as_ref().map(|b| {
                    Box::new(self.resolve_option_patterns(b.as_ref().clone(), &inner_type))
                });
                Pattern::OptionSome(resolved_binding)
            }
            Pattern::Enum { ref variant, ref binding, .. } if variant.as_ref() == "None" && binding.is_none() && matches!(match_type, TType::Option { .. }) => {
                Pattern::OptionNone
            }
            // Resolve tag index for enum variants inside nested patterns
            Pattern::Enum { variant, binding, tag: _ } => {
                // Look up the tag index from the enum definition
                let resolved_tag = match match_type {
                    TType::Custom { name, .. } => {
                        if let Some(fields) = self.typechecker.environment.custom_types.get(name.as_ref()) {
                            fields.iter().enumerate()
                                .find(|(_, (n, _))| *n == variant)
                                .map(|(i, _)| i)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                let resolved_binding = binding.map(|b| {
                    // Resolve the binding's type from the enum field definition
                    let binding_type = match match_type {
                        TType::Custom { name, .. } => {
                            self.typechecker.environment.custom_types.get(name.as_ref())
                                .and_then(|fields| fields.iter().find(|(n, _)| *n == variant))
                                .map(|(_, t)| t.clone())
                                .unwrap_or(TType::Any)
                        }
                        _ => TType::Any,
                    };
                    Box::new(self.resolve_option_patterns(*b, &binding_type))
                });
                Pattern::Enum { variant, binding: resolved_binding, tag: resolved_tag }
            }
            Pattern::Or(alternatives) => {
                Pattern::Or(alternatives.into_iter()
                    .map(|a| self.resolve_option_patterns(a, match_type))
                    .collect())
            }
            Pattern::Tuple(pats) => {
                let elements = match match_type {
                    TType::Tuple { elements } => elements.clone(),
                    _ => vec![TType::Any; pats.len()],
                };
                Pattern::Tuple(pats.into_iter().enumerate()
                    .map(|(i, p)| {
                        let ty = elements.get(i).cloned().unwrap_or(TType::Any);
                        self.resolve_option_patterns(p, &ty)
                    })
                    .collect())
            }
            Pattern::List(pats) => {
                let inner = match match_type {
                    TType::List { inner } => inner.as_ref().clone(),
                    _ => TType::Any,
                };
                Pattern::List(pats.into_iter()
                    .map(|p| self.resolve_option_patterns(p, &inner))
                    .collect())
            }
            Pattern::ListCons(heads, tail) => {
                let inner = match match_type {
                    TType::List { inner } => inner.as_ref().clone(),
                    _ => TType::Any,
                };
                Pattern::ListCons(
                    heads.into_iter()
                        .map(|p| self.resolve_option_patterns(p, &inner))
                        .collect(),
                    tail,
                )
            }
            Pattern::Struct { name, fields } => {
                // Look up struct field types from environment to resolve nested Option patterns
                // AND reorder fields to match struct definition order (compiler uses positional indexing)
                let struct_fields: Vec<(Rc<str>, TType)> = self.typechecker.environment
                    .custom_types
                    .get(name.as_ref())
                    .cloned()
                    .unwrap_or_default();
                // Filter out the auto-added "type" field
                let real_struct_fields: Vec<&(Rc<str>, TType)> = struct_fields.iter()
                    .filter(|(n, _)| n.as_ref() != "type")
                    .collect();
                // Check for `_` wildcard sentinel (discards remaining fields)
                let has_wildcard = fields.iter().any(|(n, p)| n.as_ref() == "_" && *p == Pattern::Wildcard);
                let explicit_fields: Vec<&(Rc<str>, Pattern)> = fields.iter()
                    .filter(|(n, p)| !(n.as_ref() == "_" && *p == Pattern::Wildcard))
                    .collect();
                // Reorder pattern fields to match struct definition order
                // If `_` wildcard is present, fill unmentioned fields with Wildcard
                let mut ordered_fields = Vec::with_capacity(real_struct_fields.len());
                for (sf_name, sf_type) in &real_struct_fields {
                    if let Some((_, pat)) = explicit_fields.iter().find(|(fn_, _)| fn_ == sf_name) {
                        ordered_fields.push(((*sf_name).clone(), self.resolve_option_patterns(pat.clone(), sf_type)));
                    } else if has_wildcard {
                        // Field not mentioned but `_` covers it
                        ordered_fields.push(((*sf_name).clone(), Pattern::Wildcard));
                    }
                }
                // If reordering succeeded (all fields accounted for), use ordered
                if ordered_fields.len() == real_struct_fields.len() {
                    Pattern::Struct { name, fields: ordered_fields }
                } else {
                    // Fallback: original order (for error reporting or edge cases)
                    let fallback: Vec<(Rc<str>, Pattern)> = fields.into_iter()
                        .filter(|(n, p)| !(n.as_ref() == "_" && *p == Pattern::Wildcard))
                        .map(|(fname, p)| {
                            let field_ty = struct_fields.iter()
                                .find(|(n, _)| n == &fname)
                                .map(|(_, t)| t.clone())
                                .unwrap_or(TType::Any);
                            (fname, self.resolve_option_patterns(p, &field_ty))
                        })
                        .collect();
                    Pattern::Struct { name, fields: fallback }
                }
            }
            other => other,
        }
    }

    /// Parse a single pattern (without OR).
    fn parse_single_pattern(&mut self) -> NovaResult<Pattern> {
        let pos = self.get_current_token_position();
        match self.current_token_value().cloned() {
            // wildcard _
            Some(TokenValue::Identifier(id)) if id.as_ref() == "_" => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            // bool literals
            Some(TokenValue::Bool(b)) => {
                self.advance();
                Ok(Pattern::BoolLiteral(b))
            }
            // string literal
            Some(TokenValue::StringLiteral(s)) => {
                self.advance();
                Ok(Pattern::StringLiteral(s))
            }
            // char literal
            Some(TokenValue::Char(c)) => {
                self.advance();
                Ok(Pattern::CharLiteral(c))
            }
            // integer literal
            Some(TokenValue::Integer(n)) => {
                self.advance();
                Ok(Pattern::IntLiteral(n))
            }
            // float literal
            Some(TokenValue::Float(f)) => {
                self.advance();
                Ok(Pattern::FloatLiteral(f))
            }
            // negative number: -42 or -3.14
            Some(TokenValue::Operator(Operator::Subtraction)) => {
                self.advance();
                match self.current_token_value().cloned() {
                    Some(TokenValue::Integer(n)) => {
                        self.advance();
                        Ok(Pattern::IntLiteral(-n))
                    }
                    Some(TokenValue::Float(f)) => {
                        self.advance();
                        Ok(Pattern::FloatLiteral(-f))
                    }
                    _ => Err(self.generate_error_with_pos(
                        "Expected a number after `-` in pattern",
                        "Negative patterns must be followed by an Int or Float literal.",
                        pos,
                    )),
                }
            }
            // list pattern: [p1, p2, ...] or [p1, ..rest] or []
            Some(TokenValue::StructuralSymbol(StructuralSymbol::LeftSquareBracket)) => {
                self.advance(); // consume [
                // empty list
                if self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::RightSquareBracket)) {
                    self.advance(); // consume ]
                    return Ok(Pattern::EmptyList);
                }
                let mut pats = vec![];
                loop {
                    // check for ..rest (spread/cons pattern)
                    if self.current_token().is_some_and(|t| t.is_op(Operator::ExclusiveRange)) {
                        self.advance(); // consume ..
                        let (rest_name, _) = self.get_identifier()?;
                        self.consume_symbol(StructuralSymbol::RightSquareBracket)?;
                        return Ok(Pattern::ListCons(pats, rest_name));
                    }
                    pats.push(self.parse_pattern()?);
                    if self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::Comma)) {
                        self.advance(); // consume ,
                        // allow trailing comma before ]
                        if self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::RightSquareBracket)) {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                self.consume_symbol(StructuralSymbol::RightSquareBracket)?;
                Ok(Pattern::List(pats))
            }
            // tuple pattern: (p1, p2, ...)
            Some(TokenValue::StructuralSymbol(StructuralSymbol::LeftParen)) => {
                self.advance(); // consume (
                let mut pats = vec![];
                if !self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::RightParen)) {
                    pats.push(self.parse_pattern()?);
                    while self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::Comma)) {
                        self.advance(); // consume ,
                        if self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::RightParen)) {
                            break; // trailing comma
                        }
                        pats.push(self.parse_pattern()?);
                    }
                }
                self.consume_symbol(StructuralSymbol::RightParen)?;
                Ok(Pattern::Tuple(pats))
            }
            // identifier → could be: variable, enum variant Foo(), struct Foo { ... }
            Some(TokenValue::Identifier(id)) => {
                self.advance();
                // Check if followed by ( → Enum variant pattern
                if self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::LeftParen)) {
                    self.advance(); // consume (
                    let binding = if self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::RightParen)) {
                        // No binding: None() or Red()
                        None
                    } else {
                        Some(Box::new(self.parse_pattern()?))
                    };
                    self.consume_symbol(StructuralSymbol::RightParen)?;
                    return Ok(Pattern::Enum { variant: id, binding, tag: None });
                }
                // Check if followed by { → Struct pattern
                if self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::LeftBrace)) {
                    self.advance(); // consume {
                    let mut fields = vec![];
                    let mut has_wildcard = false;
                    while !self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::RightBrace)) {
                        // Check for `_` wildcard (discard remaining fields)
                        if self.current_token().is_some_and(|t| t.is_id("_")) {
                            self.advance(); // consume `_`
                            has_wildcard = true;
                            if self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::Comma)) {
                                self.advance();
                            }
                            continue;
                        }
                        let (field_name, _) = self.get_identifier()?;
                        if self.current_token().is_some_and(|t| t.is_op(Operator::Colon)) {
                            self.advance(); // consume :
                            let pat = self.parse_pattern()?;
                            fields.push((field_name, pat));
                        } else {
                            // shorthand: `x` means `x: x` (bind to variable with same name)
                            fields.push((field_name.clone(), Pattern::Variable(field_name)));
                        }
                        if self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::Comma)) {
                            self.advance();
                        }
                    }
                    self.consume_symbol(StructuralSymbol::RightBrace)?;
                    // If `_` was used, fill in the remaining fields with wildcards
                    if has_wildcard {
                        // We store a sentinel: the field name "_" maps to Wildcard
                        // validate_pattern_type and resolve_option_patterns will expand this
                        fields.push((Rc::from("_"), Pattern::Wildcard));
                    }
                    return Ok(Pattern::Struct { name: id, fields });
                }
                // Plain variable binding
                Ok(Pattern::Variable(id))
            }
            _ => Err(self.generate_error_with_pos(
                "Invalid pattern in match arm",
                "Expected one of: literal (Int, Float, String, Char, Bool),\n  \
                 `_` (wildcard), a variable name, `[...]` (list pattern), `(...)` (tuple pattern),\n  \
                 `Variant(...)` (enum pattern), `Struct { ... }` (struct pattern), or `pat1 | pat2` (OR pattern).",
                pos,
            )),
        }
    }

    /// Parse a value-match expression (non-enum).
    fn value_match_expr(&mut self, expr: Expr) -> NovaResult<Expr> {
        let pos = self.get_current_token_position();
        let match_type = expr.get_type();
        let mut arms: Vec<(Pattern, Option<Expr>, Vec<Statement>)> = vec![];
        self.consume_symbol(StructuralSymbol::LeftBrace)?;
        let mut default_branch: Option<Vec<Statement>> = None;

        while !self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::RightBrace)) {
            let pat_pos = self.get_current_token_position();
            let pat = self.parse_pattern()?;

            // Validate pattern type compatibility (BEFORE reordering, to catch field count/name errors)
            self.validate_pattern_type(&pat, &match_type, &pat_pos)?;

            let pat = self.resolve_option_patterns(pat, &match_type);

            // check if this is a default wildcard (only if NOT followed by `if` guard)
            if pat == Pattern::Wildcard && !self.current_token().is_some_and(|t| t.is_id("if")) {
                if default_branch.is_some() {
                    return Err(self.generate_error_with_pos(
                        "Default branch `_` is already defined",
                        "A match expression can only have one default `_` branch.",
                        pat_pos,
                    ));
                }
                self.consume_operator(Operator::FatArrow)?;
                if self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::LeftBrace)) {
                    default_branch = Some(self.block()?);
                } else {
                    let body = self.expr()?;
                    default_branch = Some(vec![Statement::Expression {
                        ttype: body.get_type(),
                        expr: body,
                    }]);
                };
                while self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::Comma)) {
                    self.advance();
                }
                continue;
            }

            // ── Parse optional if-guard: `pattern if condition => body` ──
            let guard = if self.current_token().is_some_and(|t| t.is_id("if")) {
                self.advance(); // consume `if`
                // Register bindings before parsing guard (guard can reference bound variables)
                self.typechecker.environment.push_block();
                self.register_pattern_bindings(&pat, &match_type)?;
                let guard_expr = self.logical_or_expr()?;
                if guard_expr.get_type() != TType::Bool && guard_expr.get_type() != TType::Void {
                    return Err(self.generate_error_with_pos(
                        format!("Match guard must be a Bool, found `{}`", guard_expr.get_type()),
                        "The `if` guard in a match arm must evaluate to a Bool.\n  Example: `pattern if x > 0 => { ... }`",
                        pat_pos.clone(),
                    ));
                }
                self.typechecker.environment.pop_block();
                Some(guard_expr)
            } else {
                None
            };

            self.consume_operator(Operator::FatArrow)?;

            // type-check: register bindings from the pattern
            self.typechecker.environment.push_block();
            self.register_pattern_bindings(&pat, &match_type)?;

            let body = if self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::LeftBrace)) {
                self.block()?
            } else {
                let body_expr = self.expr()?;
                vec![Statement::Expression {
                    ttype: body_expr.get_type(),
                    expr: body_expr,
                }]
            };

            self.typechecker.environment.pop_block();
            arms.push((pat, guard, body));

            while self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::Comma)) {
                self.advance();
            }
        }
        self.consume_symbol(StructuralSymbol::RightBrace)?;

        // ── Duplicate arm detection ──
        // Two arms with the same pattern are only duplicates if neither has a guard
        for i in 0..arms.len() {
            for j in (i + 1)..arms.len() {
                if arms[i].0 == arms[j].0 && arms[i].1.is_none() && arms[j].1.is_none() {
                    return Err(self.generate_error_with_pos(
                        format!("Duplicate match arm: `{}`", Self::pattern_to_string(&arms[j].0)),
                        "This pattern already appears in an earlier arm, so this arm can never be reached.\n  Remove the duplicate or use a different pattern.\n  Tip: use an `if` guard to distinguish arms with the same pattern:\n    pattern if condition => { ... }",
                        pos.clone(),
                    ));
                }
            }
        }

        // ── Exhaustiveness check ──
        let has_catchall = default_branch.is_some()
            || arms.iter().any(|(p, guard, _)| guard.is_none() && Self::is_irrefutable(p));
        if !has_catchall {
            // Bool exhaustiveness: true + false (only unguarded arms count)
            let is_bool_exhaustive = match_type == TType::Bool && {
                let has_true = arms.iter().any(|(p, guard, _)| guard.is_none() && Self::arm_contains(p, &|q| *q == Pattern::BoolLiteral(true)));
                let has_false = arms.iter().any(|(p, guard, _)| guard.is_none() && Self::arm_contains(p, &|q| *q == Pattern::BoolLiteral(false)));
                has_true && has_false
            };
            // Option exhaustiveness: Some + None (only unguarded arms count)
            let is_option_exhaustive = matches!(match_type, TType::Option { .. }) && {
                let has_some = arms.iter().any(|(p, guard, _)| guard.is_none() && Self::arm_contains(p, &|q| matches!(q, Pattern::OptionSome(_))));
                let has_none = arms.iter().any(|(p, guard, _)| guard.is_none() && Self::arm_contains(p, &|q| matches!(q, Pattern::OptionNone)));
                has_some && has_none
            };
            // User enum exhaustiveness: all variants present (only unguarded arms count)
            let is_enum_exhaustive = match &match_type {
                TType::Custom { name, .. } => {
                    if self.typechecker.environment.enums.has(name) {
                        if let Some(fields) = self.typechecker.environment.custom_types.get(name.as_ref()) {
                            let variant_names: Vec<&str> = fields.iter()
                                .filter(|(n, _)| n.as_ref() != "type")
                                .map(|(n, _)| n.as_ref())
                                .collect();
                            variant_names.iter().all(|vname| {
                                arms.iter().any(|(p, guard, _)| guard.is_none() && Self::arm_contains(p, &|q| matches!(q, Pattern::Enum { variant, .. } if variant.as_ref() == *vname)))
                            })
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                _ => false,
            };
            if !is_bool_exhaustive && !is_option_exhaustive && !is_enum_exhaustive {
                return Err(self.generate_error_with_pos(
                    "Non-exhaustive match",
                    format!(
                        "When matching on `{}`, not all values can be covered.\n  Add a default: `_ => ...`",
                        match_type
                    ),
                    pos,
                ));
            }
        }

        // type-check: all arms produce the same type
        let mut result_type: Option<TType> = None;
        for (_, _, arm_body) in arms.iter() {
            if let Some(arm_ty) = Self::tail_type(arm_body) {
                if let Some(ref prev) = result_type {
                    if *prev != arm_ty {
                        return Err(self.generate_error_with_pos(
                            "All arms of a match expression must return the same type",
                            format!(
                                "One arm returns `{}` but another returns `{}`.",
                                prev, arm_ty
                            ),
                            pos.clone(),
                        ));
                    }
                } else {
                    result_type = Some(arm_ty);
                }
            }
        }
        if let Some(ref def) = default_branch {
            if let Some(def_ty) = Self::tail_type(def) {
                if let Some(ref prev) = result_type {
                    if *prev != def_ty {
                        return Err(self.generate_error_with_pos(
                            "All arms of a match expression must return the same type",
                            format!(
                                "The default `_` arm returns `{}` but other arms return `{}`.",
                                def_ty, prev
                            ),
                            pos.clone(),
                        ));
                    }
                } else {
                    result_type = Some(def_ty);
                }
            }
        }

        let ttype = result_type.unwrap_or(TType::Void);

        Ok(Expr::ValueMatchExpr {
            ttype,
            expr: Box::new(expr),
            arms,
            default: default_branch,
            position: pos,
        })
    }

    /// Parse a value-match statement (non-enum).
    fn value_match_statement(&mut self, expr: Expr) -> NovaResult<Option<Statement>> {
        let pos = self.get_current_token_position();
        let match_type = expr.get_type();
        let mut arms: Vec<(Pattern, Option<Expr>, Vec<Statement>)> = vec![];
        self.consume_symbol(StructuralSymbol::LeftBrace)?;
        let mut default_branch: Option<Vec<Statement>> = None;

        while !self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::RightBrace)) {
            let pat_pos = self.get_current_token_position();
            let pat = self.parse_pattern()?;

            // Validate pattern type compatibility (BEFORE reordering, to catch field count/name errors)
            self.validate_pattern_type(&pat, &match_type, &pat_pos)?;

            let pat = self.resolve_option_patterns(pat, &match_type);

            if pat == Pattern::Wildcard && !self.current_token().is_some_and(|t| t.is_id("if")) {
                if default_branch.is_some() {
                    return Err(self.generate_error_with_pos(
                        "Default branch `_` is already defined",
                        "A match statement can only have one default `_` branch.",
                        pat_pos.clone(),
                    ));
                }
                self.consume_operator(Operator::FatArrow)?;
                if self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::LeftBrace)) {
                    default_branch = Some(self.block()?);
                } else {
                    let body = self.expr()?;
                    default_branch = Some(vec![Statement::Expression {
                        ttype: body.get_type(),
                        expr: body,
                    }]);
                };
                while self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::Comma)) {
                    self.advance();
                }
                continue;
            }

            // ── Parse optional if-guard: `pattern if condition => body` ──
            let guard = if self.current_token().is_some_and(|t| t.is_id("if")) {
                self.advance(); // consume `if`
                self.typechecker.environment.push_block();
                self.register_pattern_bindings(&pat, &match_type)?;
                let guard_expr = self.logical_or_expr()?;
                if guard_expr.get_type() != TType::Bool && guard_expr.get_type() != TType::Void {
                    return Err(self.generate_error_with_pos(
                        format!("Match guard must be a Bool, found `{}`", guard_expr.get_type()),
                        "The `if` guard in a match arm must evaluate to a Bool.\n  Example: `pattern if x > 0 => { ... }`",
                        pat_pos.clone(),
                    ));
                }
                self.typechecker.environment.pop_block();
                Some(guard_expr)
            } else {
                None
            };

            self.consume_operator(Operator::FatArrow)?;

            self.typechecker.environment.push_block();
            self.register_pattern_bindings(&pat, &match_type)?;

            let body = if self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::LeftBrace)) {
                self.block()?
            } else {
                let body_expr = self.expr()?;
                vec![Statement::Expression {
                    ttype: body_expr.get_type(),
                    expr: body_expr,
                }]
            };

            self.typechecker.environment.pop_block();
            arms.push((pat, guard, body));

            while self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::Comma)) {
                self.advance();
            }
        }
        self.consume_symbol(StructuralSymbol::RightBrace)?;

        // ── Duplicate arm detection ──
        // Two arms with the same pattern are only duplicates if neither has a guard
        for i in 0..arms.len() {
            for j in (i + 1)..arms.len() {
                if arms[i].0 == arms[j].0 && arms[i].1.is_none() && arms[j].1.is_none() {
                    return Err(self.generate_error_with_pos(
                        format!("Duplicate match arm: `{}`", Self::pattern_to_string(&arms[j].0)),
                        "This pattern already appears in an earlier arm, so this arm can never be reached.\n  Remove the duplicate or use a different pattern.\n  Tip: use an `if` guard to distinguish arms with the same pattern:\n    pattern if condition => { ... }",
                        pos.clone(),
                    ));
                }
            }
        }

        // ── Exhaustiveness check ──
        let has_catchall = default_branch.is_some()
            || arms.iter().any(|(p, guard, _)| guard.is_none() && Self::is_irrefutable(p));
        if !has_catchall {
            let is_bool_exhaustive = match_type == TType::Bool && {
                let has_true = arms.iter().any(|(p, guard, _)| guard.is_none() && Self::arm_contains(p, &|q| *q == Pattern::BoolLiteral(true)));
                let has_false = arms.iter().any(|(p, guard, _)| guard.is_none() && Self::arm_contains(p, &|q| *q == Pattern::BoolLiteral(false)));
                has_true && has_false
            };
            let is_option_exhaustive = matches!(match_type, TType::Option { .. }) && {
                let has_some = arms.iter().any(|(p, guard, _)| guard.is_none() && Self::arm_contains(p, &|q| matches!(q, Pattern::OptionSome(_))));
                let has_none = arms.iter().any(|(p, guard, _)| guard.is_none() && Self::arm_contains(p, &|q| matches!(q, Pattern::OptionNone)));
                has_some && has_none
            };
            let is_enum_exhaustive = match &match_type {
                TType::Custom { name, .. } => {
                    if self.typechecker.environment.enums.has(name) {
                        if let Some(fields) = self.typechecker.environment.custom_types.get(name.as_ref()) {
                            let variant_names: Vec<&str> = fields.iter()
                                .filter(|(n, _)| n.as_ref() != "type")
                                .map(|(n, _)| n.as_ref())
                                .collect();
                            variant_names.iter().all(|vname| {
                                arms.iter().any(|(p, guard, _)| guard.is_none() && Self::arm_contains(p, &|q| matches!(q, Pattern::Enum { variant, .. } if variant.as_ref() == *vname)))
                            })
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                _ => false,
            };
            if !is_bool_exhaustive && !is_option_exhaustive && !is_enum_exhaustive {
                return Err(self.generate_error_with_pos(
                    "Non-exhaustive match",
                    format!(
                        "When matching on `{}`, not all values can be covered.\n  Add a default: `_ => ...`",
                        match_type
                    ),
                    pos,
                ));
            }
        }

        Ok(Some(Statement::ValueMatch {
            ttype: TType::Void,
            expr,
            arms,
            default: default_branch,
            position: pos,
        }))
    }

    /// Register variable bindings from a pattern into the typechecker environment.
    fn register_pattern_bindings(&mut self, pattern: &Pattern, match_type: &TType) -> NovaResult<()> {
        match pattern {
            Pattern::Wildcard => {}
            Pattern::Variable(name) => {
                self.typechecker.environment.insert_symbol(
                    name,
                    match_type.clone(),
                    None,
                    SymbolKind::Variable,
                );
            }
            Pattern::IntLiteral(_) | Pattern::FloatLiteral(_) | Pattern::StringLiteral(_)
            | Pattern::BoolLiteral(_) | Pattern::CharLiteral(_) | Pattern::EmptyList => {}
            Pattern::List(pats) => {
                let inner = match match_type {
                    TType::List { inner } => inner.as_ref().clone(),
                    TType::Tuple { elements } => {
                        // for tuples matched as list
                        for (i, pat) in pats.iter().enumerate() {
                            let elem_ty = elements.get(i).cloned().unwrap_or(TType::Any);
                            self.register_pattern_bindings(pat, &elem_ty)?;
                        }
                        return Ok(());
                    }
                    _ => TType::Any,
                };
                for pat in pats {
                    self.register_pattern_bindings(pat, &inner)?;
                }
            }
            Pattern::ListCons(head_pats, tail_name) => {
                let inner = match match_type {
                    TType::List { inner } => inner.as_ref().clone(),
                    _ => TType::Any,
                };
                for pat in head_pats {
                    self.register_pattern_bindings(pat, &inner)?;
                }
                // tail is the same list type
                self.typechecker.environment.insert_symbol(
                    tail_name,
                    match_type.clone(),
                    None,
                    SymbolKind::Variable,
                );
            }
            Pattern::Tuple(pats) => {
                let elements = match match_type {
                    TType::Tuple { elements } => elements.clone(),
                    _ => vec![TType::Any; pats.len()],
                };
                for (i, pat) in pats.iter().enumerate() {
                    let elem_ty = elements.get(i).cloned().unwrap_or(TType::Any);
                    self.register_pattern_bindings(pat, &elem_ty)?;
                }
            }
            Pattern::Or(alternatives) => {
                // Or patterns must not bind variables (enforced elsewhere),
                // but we register from the first alternative for type checking.
                if let Some(first) = alternatives.first() {
                    self.register_pattern_bindings(first, match_type)?;
                }
            }
            Pattern::Enum { variant, binding, .. } => {
                // The binding gets the inner type of the enum variant
                if let Some(sub_pat) = binding {
                    // Determine the inner type of the matched variant
                    let inner_type = match match_type {
                        TType::Option { inner } => {
                            // Some(x) → x gets the inner type; None() has no binding
                            inner.as_ref().clone()
                        }
                        TType::Custom { name, type_params } => {
                            // Look up the variant in custom_types
                            let fields = self.typechecker.environment.custom_types
                                .get(name.as_ref())
                                .cloned()
                                .unwrap_or_default();
                            // Resolve generic params if needed
                            let generic_params = self.typechecker.environment.generic_type_struct
                                .get(name.as_ref())
                                .cloned();
                            let mut vtype = TType::Any;
                            for (fname, ftype) in &fields {
                                if fname.as_ref() == variant.as_ref() {
                                    vtype = if let Some(ref gp) = generic_params {
                                        TypeChecker::replace_generic_types(&ftype, gp, type_params)
                                    } else {
                                        ftype.clone()
                                    };
                                    break;
                                }
                            }
                            vtype
                        }
                        _ => TType::Any,
                    };
                    self.register_pattern_bindings(sub_pat, &inner_type)?;
                }
            }
            Pattern::Struct { name, fields } => {
                // Look up the struct fields and register bindings
                let struct_fields: Vec<(Rc<str>, TType)> = self.typechecker.environment
                    .custom_types
                    .get(name.as_ref())
                    .cloned()
                    .unwrap_or_default();
                for (field_name, pat) in fields {
                    let field_ty = struct_fields.iter()
                        .find(|(n, _)| n == field_name)
                        .map(|(_, t)| t.clone())
                        .unwrap_or(TType::Any);
                    self.register_pattern_bindings(pat, &field_ty)?;
                }
            }
            Pattern::OptionSome(binding) => {
                if let Some(sub_pat) = binding {
                    let inner_type = match match_type {
                        TType::Option { inner } => inner.as_ref().clone(),
                        _ => TType::Any,
                    };
                    self.register_pattern_bindings(sub_pat, &inner_type)?;
                }
            }
            Pattern::OptionNone => {
                // No bindings for None
            }
        }
        Ok(())
    }

    fn match_expr(&mut self) -> NovaResult<Expr> {
        self.consume_identifier(Some("match"))?;
        let expr = self.expr()?;

        let type_name = expr.get_type().custom_to_string().map(|s| s.to_string());
        let is_enum = type_name
            .as_deref()
            .is_some_and(|n| self.typechecker.environment.enums.has(&Rc::from(n)));

        if !is_enum {
            // Dispatch to generalized (value) pattern matching
            return self.value_match_expr(expr);
        }

        let pos = self.get_current_token_position();
        let mut branches: Vec<(usize, Option<Rc<str>>, Vec<Statement>)> = vec![];
        self.consume_symbol(LeftBrace)?;
        let mut default_branch: Option<Vec<Statement>> = None;

        while !self
            .current_token()
            .is_some_and(|t| t.is_symbol(RightBrace))
        {
            let (variant, vpos) = self.get_identifier()?;

            // ── Detect qualified variant names like `Color::Red` ──
            if self
                .current_token()
                .is_some_and(|t| t.is_op(Operator::DoubleColon))
            {
                let next_variant = self
                    .input
                    .get(self.index + 1)
                    .and_then(|t| {
                        if let TokenValue::Identifier(id) = &t.value {
                            Some(id.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "...".into());
                return Err(self.generate_error_with_pos(
                    format!(
                        "Use just `{}` instead of qualifying with the enum type",
                        next_variant
                    ),
                    format!(
                        "Match arms use the variant name alone, not the fully-qualified path.\n  \
                         Write `{nv}` instead of `{v}::{nv}`",
                        nv = next_variant,
                        v = variant
                    ),
                    vpos,
                ));
            }

            if &*variant == "_" {
                if default_branch.is_some() {
                    return Err(self.generate_error_with_pos(
                        "Default branch `_` is already defined",
                        "A match expression can only have one default `_` branch. Remove the duplicate.",
                        vpos,
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
                    }]);
                };
                while self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
                    self.advance();
                }
                continue;
            }

            // ── Collect first variant (and optional binding) ──
            let mut or_variants: Vec<(Rc<str>, Option<Rc<str>>)> = vec![];
            {
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
                or_variants.push((variant, enum_id));
            }

            // ── Collect additional OR variants: `| Variant2() | Variant3()` ──
            while self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::Pipe)) {
                self.advance(); // consume |
                let (or_variant, or_vpos) = self.get_identifier()?;
                // Detect qualified names
                if self.current_token().is_some_and(|t| t.is_op(Operator::DoubleColon)) {
                    let next_variant = self
                        .input
                        .get(self.index + 1)
                        .and_then(|t| {
                            if let TokenValue::Identifier(id) = &t.value {
                                Some(id.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| "...".into());
                    return Err(self.generate_error_with_pos(
                        format!("Use just `{}` instead of qualifying with the enum type", next_variant),
                        format!(
                            "Match arms use the variant name alone, not the fully-qualified path.\n  \
                             Write `{nv}` instead of `{v}::{nv}`",
                            nv = next_variant,
                            v = or_variant
                        ),
                        or_vpos,
                    ));
                }
                let mut or_enum_id = None;
                if self.current_token().is_some_and(|t| t.is_symbol(LeftParen)) {
                    self.consume_symbol(LeftParen)?;
                    if !self.current_token().is_some_and(|t| t.is_symbol(RightParen)) {
                        or_enum_id = Some(self.get_identifier()?.0);
                    }
                    self.consume_symbol(RightParen)?;
                }
                or_variants.push((or_variant, or_enum_id));
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
                            format!(
                                "Expected a generic custom type, found `{}`",
                                expr.get_type()
                            ),
                            "This type has generic type parameters but the value does not carry type parameter information.\n  This is an internal type error — please report it.",
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

                // ── Validate all OR variants and collect their tags ──
                let mut or_tags: Vec<usize> = vec![];
                let mut shared_enum_id: Option<Rc<str>> = None;
                let mut shared_vtype = TType::None;

                for (or_var, or_eid) in or_variants.iter() {
                    let mut tag = 0;
                    let mut found = false;
                    let mut vtype = TType::None;

                    for (i, field) in new_fields.iter().enumerate() {
                        if *or_var == field.0 {
                            tag = i;
                            vtype = field.1.clone();
                            found = true;
                        }
                    }

                    if vtype != TType::None && or_eid.is_none() {
                        return Err(self.generate_error_with_pos(
                            format!(
                                "Variant `{}` carries data but is missing a binding variable",
                                or_var
                            ),
                            format!(
                                "This variant holds a value of type `{}`. You must bind it to a variable.\n  Example: `{}(my_var) => {{ ... }}`",
                                vtype, or_var
                            ),
                            vpos.clone(),
                        ));
                    }

                    if !found {
                        let available: Vec<String> = new_fields
                            .iter()
                            .filter(|(name, _)| name.as_ref() != "type")
                            .map(|(name, ttype)| {
                                if *ttype == TType::None {
                                    format!("`{}`", name)
                                } else {
                                    format!("`{}` (holds `{}`)", name, ttype)
                                }
                            })
                            .collect();
                        return Err(self.generate_error_with_pos(
                            format!("Variant `{}` not found in this enum type", or_var),
                            format!(
                                "Available variants: {}. Check for typos — variant names are case-sensitive.",
                                available.join(", ")
                            ),
                            vpos.clone(),
                        ));
                    }

                    or_tags.push(tag);
                    if or_eid.is_some() {
                        shared_enum_id = or_eid.clone();
                        shared_vtype = vtype;
                    }
                }

                let enum_id = shared_enum_id.or_else(|| or_variants[0].1.clone());

                self.typechecker.environment.push_block();
                self.typechecker.environment.insert_symbol(
                    enum_id.as_deref().unwrap_or_default(),
                    shared_vtype,
                    None,
                    SymbolKind::Variable,
                );

                let body = if self.current_token().is_some_and(|t| t.is_symbol(LeftBrace)) {
                    self.block()?
                } else {
                    let body = self.expr()?;
                    vec![Statement::Expression {
                        ttype: body.clone().get_type(),
                        expr: body,
                    }]
                };

                // Push one arm per tag, all sharing the same body
                for tag in or_tags {
                    branches.push((tag, enum_id.clone(), body.clone()));
                }

                self.typechecker.environment.pop_block();
            }

            while self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
                self.advance();
            }
        }
        self.consume_symbol(RightBrace)?;

        // ── Duplicate arm detection (enum match expression) ──
        {
            let mut seen_tags: Vec<usize> = vec![];
            if let Some(fields) = self.typechecker.environment.custom_types.get(
                expr.get_type().custom_to_string().unwrap()
            ) {
                for (tag, _, _) in &branches {
                    if seen_tags.contains(tag) {
                        let variant_name = fields.get(*tag)
                            .map(|(n, _)| n.to_string())
                            .unwrap_or_else(|| format!("tag {}", tag));
                        return Err(self.generate_error_with_pos(
                            format!("Duplicate match arm for variant `{}`", variant_name),
                            "This variant already has an arm. Each enum variant can only appear once.\n  Remove the duplicate arm.",
                            pos.clone(),
                        ));
                    }
                    seen_tags.push(*tag);
                }
            }
        }

        // ── Exhaustiveness check ──
        if default_branch.is_none() {
            let mut covered = vec![];
            for (tag, _, _) in branches.iter() {
                covered.push(*tag);
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
                            format!(
                                "Expected a generic custom type, found `{}`",
                                expr.get_type()
                            ),
                            "This type has generic type parameters but the value does not carry type parameter information.\n  This is an internal type error — please report it.",
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
                        let arm_hint = if field.1 == TType::None {
                            format!("{}() => {{ ... }}", field.0)
                        } else {
                            format!("{}(val) => {{ ... }}", field.0)
                        };
                        return Err(self.generate_error_with_pos(
                            format!("Variant `{}` is not covered in match", field.0),
                            format!(
                                "All enum variants must be handled.\n  Add: `{}`\n  Or add a default branch: `_ => {{ ... }}`",
                                arm_hint
                            ),
                            pos,
                        ));
                    }
                }
            }
        }

        // ── Type-check: all arms must produce the same type ──
        let mut result_type: Option<TType> = None;
        for (_, _, arm_body) in branches.iter() {
            if let Some(arm_ty) = Self::tail_type(arm_body) {
                if let Some(ref prev) = result_type {
                    if *prev != arm_ty {
                        return Err(self.generate_error_with_pos(
                            "All arms of a match expression must return the same type".to_string(),
                            format!(
                                "One arm returns `{}` but another returns `{}`.\n  All must be the same type since this is used as an expression.",
                                prev, arm_ty
                            ),
                            pos,
                        ));
                    }
                } else {
                    result_type = Some(arm_ty);
                }
            }
        }
        if let Some(ref def) = default_branch {
            if let Some(def_ty) = Self::tail_type(def) {
                if let Some(ref prev) = result_type {
                    if *prev != def_ty {
                        return Err(self.generate_error_with_pos(
                            "All arms of a match expression must return the same type".to_string(),
                            format!(
                                "The default `_` arm returns `{}` but other arms return `{}`.\n  All must be the same type since this is used as an expression.",
                                def_ty, prev
                            ),
                            pos,
                        ));
                    }
                } else {
                    result_type = Some(def_ty);
                }
            }
        }

        let ttype = result_type.unwrap_or(TType::Void);

        Ok(Expr::MatchExpr {
            ttype,
            expr: Box::new(expr),
            arms: branches,
            default: default_branch,
            position: pos,
        })
    }

    fn match_statement(&mut self) -> NovaResult<Option<Statement>> {
        self.consume_identifier(Some("match"))?;
        let expr = self.expr()?;

        let type_name = expr.get_type().custom_to_string().map(|s| s.to_string());
        let is_enum = type_name
            .as_deref()
            .is_some_and(|n| self.typechecker.environment.enums.has(&Rc::from(n)));

        if !is_enum {
            // Dispatch to generalized (value) pattern matching
            return self.value_match_statement(expr);
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

            // ── Fix: detect qualified variant names like `Color::Red` ──
            if self
                .current_token()
                .is_some_and(|t| t.is_op(Operator::DoubleColon))
            {
                // Peek ahead to grab the variant name after `::`
                let next_variant = self
                    .input
                    .get(self.index + 1)
                    .and_then(|t| {
                        if let TokenValue::Identifier(id) = &t.value {
                            Some(id.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "...".into());
                return Err(self.generate_error_with_pos(
                    format!(
                        "Use just `{}` instead of qualifying with the enum type",
                        next_variant
                    ),
                    format!(
                        "Match arms use the variant name alone, not the fully-qualified path.\n  \
                         Write `{nv}` instead of `{v}::{nv}`\n  \
                         Example:\n    match value {{\n      {nv}()  => {{ ... }}\n      Other(x) => {{ ... }}\n    }}",
                        nv = next_variant,
                        v = variant
                    ),
                    pos,
                ));
            }

            if &*variant == "_" {
                // check to see if default branch is already defined
                if default_branch.is_some() {
                    return Err(self.generate_error_with_pos(
                        "Default branch `_` is already defined",
                        "A match statement can only have one default `_` branch. Remove the duplicate.",
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
                // ── Fix: silently skip optional trailing commas after default arm ──
                while self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
                    self.advance();
                }
                continue;
            }
            // ── Collect first variant (and optional binding) ──
            let mut or_variants: Vec<(Rc<str>, Option<Rc<str>>)> = vec![];
            {
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
                or_variants.push((variant, enum_id));
            }

            // ── Collect additional OR variants: `| Variant2() | Variant3()` ──
            while self.current_token().is_some_and(|t| t.is_symbol(StructuralSymbol::Pipe)) {
                self.advance(); // consume |
                let (or_variant, or_vpos) = self.get_identifier()?;
                // Detect qualified names
                if self.current_token().is_some_and(|t| t.is_op(Operator::DoubleColon)) {
                    let next_variant = self
                        .input
                        .get(self.index + 1)
                        .and_then(|t| {
                            if let TokenValue::Identifier(id) = &t.value {
                                Some(id.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| "...".into());
                    return Err(self.generate_error_with_pos(
                        format!("Use just `{}` instead of qualifying with the enum type", next_variant),
                        format!(
                            "Match arms use the variant name alone, not the fully-qualified path.\n  \
                             Write `{nv}` instead of `{v}::{nv}`",
                            nv = next_variant,
                            v = or_variant
                        ),
                        or_vpos,
                    ));
                }
                let mut or_enum_id = None;
                if self.current_token().is_some_and(|t| t.is_symbol(LeftParen)) {
                    self.consume_symbol(LeftParen)?;
                    if !self.current_token().is_some_and(|t| t.is_symbol(RightParen)) {
                        or_enum_id = Some(self.get_identifier()?.0);
                    }
                    self.consume_symbol(RightParen)?;
                }
                or_variants.push((or_variant, or_enum_id));
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
                            format!("Expected a generic custom type, found `{}`", expr.get_type()),
                            "This type has generic type parameters but the value does not carry type parameter information.\n  This is an internal type error — please report it.",
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

                // ── Validate all OR variants and collect their tags ──
                let mut or_tags: Vec<usize> = vec![];
                let mut shared_enum_id: Option<Rc<str>> = None;
                let mut shared_vtype = TType::None;

                for (or_var, or_eid) in or_variants.iter() {
                    let mut tag = 0;
                    let mut found = false;
                    let mut vtype = TType::None;

                    for (i, field) in new_fields.iter().enumerate() {
                        if *or_var == field.0 {
                            tag = i;
                            vtype = field.1.clone();
                            found = true;
                        }
                    }

                    if vtype != TType::None && or_eid.is_none() {
                        return Err(self.generate_error_with_pos(
                            format!(
                                "Variant `{}` carries data but is missing a binding variable",
                                or_var
                            ),
                            format!(
                                "This variant holds a value of type `{}`. You must bind it to a variable.\n  Example: `{}(my_var) => {{ ... }}`",
                                vtype, or_var
                            ),
                            pos.clone(),
                        ));
                    }

                    if !found {
                        let available: Vec<String> = new_fields
                            .iter()
                            .filter(|(name, _)| name.as_ref() != "type")
                            .map(|(name, ttype)| {
                                if *ttype == TType::None {
                                    format!("`{}`", name)
                                } else {
                                    format!("`{}` (holds `{}`)", name, ttype)
                                }
                            })
                            .collect();
                        return Err(self.generate_error_with_pos(
                            format!("Variant `{}` not found in this enum type", or_var),
                            format!(
                                "Available variants: {}. Check for typos — variant names are case-sensitive.",
                                available.join(", ")
                            ),
                            pos.clone(),
                        ));
                    }

                    or_tags.push(tag);
                    if or_eid.is_some() {
                        shared_enum_id = or_eid.clone();
                        shared_vtype = vtype;
                    }
                }

                let enum_id = shared_enum_id.or_else(|| or_variants[0].1.clone());

                self.typechecker.environment.push_block();
                self.typechecker.environment.insert_symbol(
                    enum_id.as_deref().unwrap_or_default(),
                    shared_vtype,
                    None,
                    SymbolKind::Variable,
                );

                let body = if self.current_token().is_some_and(|t| t.is_symbol(LeftBrace)) {
                    self.block()?
                } else {
                    let body = self.expr()?;
                    vec![Statement::Expression {
                        ttype: body.clone().get_type(),
                        expr: body,
                    }]
                };

                // Push one arm per tag, all sharing the same body
                for tag in or_tags {
                    branches.push((tag, enum_id.clone(), body.clone()));
                }

                self.typechecker.environment.pop_block();
            }

            // ── Fix: silently skip optional trailing commas between match arms ──
            while self.current_token().is_some_and(|t| t.is_symbol(Comma)) {
                self.advance();
            }
        }
        self.consume_symbol(RightBrace)?;

        // ── Duplicate arm detection (enum match statement) ──
        {
            let mut seen_tags: Vec<usize> = vec![];
            if let Some(fields) = self.typechecker.environment.custom_types.get(
                expr.get_type().custom_to_string().unwrap()
            ) {
                for (tag, _, _) in &branches {
                    if seen_tags.contains(tag) {
                        let variant_name = fields.get(*tag)
                            .map(|(n, _)| n.to_string())
                            .unwrap_or_else(|| format!("tag {}", tag));
                        return Err(self.generate_error_with_pos(
                            format!("Duplicate match arm for variant `{}`", variant_name),
                            "This variant already has an arm. Each enum variant can only appear once.\n  Remove the duplicate arm.",
                            pos.clone(),
                        ));
                    }
                    seen_tags.push(*tag);
                }
            }
        }

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
                            format!("Expected a generic custom type, found `{}`", expr.get_type()),
                            "This type has generic type parameters but the value does not carry type parameter information.\n  This is an internal type error — please report it.",
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
                        let arm_hint = if field.1 == TType::None {
                            format!("{}() => {{ ... }}", field.0)
                        } else {
                            format!("{}(val) => {{ ... }}", field.0)
                        };
                        return Err(self.generate_error_with_pos(
                            format!("Variant `{}` is not covered in match", field.0),
                            format!(
                                "All enum variants must be handled.\n  Add: `{}`\n  Or add a default branch: `_ => {{ ... }}`",
                                arm_hint
                            ),
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
                "Each parameter must have a unique name. Choose a different name for this parameter.",
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
                // ── Detect common wrong-language keywords and give helpful hints ──
                "var" | "const" => {
                    let kw = id.clone();
                    Err(self.generate_error(
                        format!("Unknown keyword `{}`", kw),
                        format!(
                            "Nova uses `let` for variable declarations (there is no `{}`).\n  \
                             Example: `let x = 5`\n  \
                             With a type annotation: `let x: Int = 5`",
                            kw
                        ),
                    ))
                }
                "def" | "func" | "function" => {
                    let kw = id.clone();
                    Err(self.generate_error(
                        format!("Unknown keyword `{}`", kw),
                        format!(
                            "Nova uses `fn` to define functions (there is no `{}`).\n  \
                             Example: `fn add(a: Int, b: Int) -> Int {{ return a + b }}`\n  \
                             For closures: `|x: Int| x * 2`",
                            kw
                        ),
                    ))
                }
                "class" => {
                    Err(self.generate_error(
                        "Unknown keyword `class`",
                        "Nova uses `struct` for data types and `enum` for tagged unions (there are no classes).\n  \
                         Struct example: `struct Point { x: Float, y: Float }`\n  \
                         Enum example:  `enum Color { Red, Green, Blue }`\n  \
                         To add methods: `fn extends method_name(self: MyType) -> R { ... }`",
                    ))
                }
                "switch" => {
                    Err(self.generate_error(
                        "Unknown keyword `switch`",
                        "Nova uses `match` for pattern matching on enums (there is no `switch`).\n  \
                         Note: `match` only works on enum types, not on integers or strings.\n  \
                         Example:\n    enum Color { Red, Green, Blue }\n    match my_color {\n      Red()   => { println(\"red\") }\n      Green() => { println(\"green\") }\n      _       => { println(\"other\") }\n    }\n  \
                         For integer/string branching, use `if`/`elif`/`else` instead.",
                    ))
                }
                "elif" => {
                    // `elif` at statement level means it's not following an if/elif body.
                    // In a proper if-chain, elif is consumed by if_statement() directly.
                    Err(self.generate_error(
                        "Unexpected `elif` without a preceding `if`",
                        "An `elif` must follow an `if` or another `elif` block.\n  \
                         Example:\n    if x > 0 {\n      println(\"positive\")\n    } elif x < 0 {\n      println(\"negative\")\n    } else {\n      println(\"zero\")\n    }",
                    ))
                }
                "null" | "nil" | "none" | "undefined" => {
                    let kw = id.clone();
                    Err(self.generate_error(
                        format!("Unknown keyword `{}`", kw),
                        format!(
                            "Nova uses `None(T)` to represent the absence of a value (there is no `{}`).\n  \
                             Example: `let x: Option(Int) = None(Int)`\n  \
                             Note: `None` requires a type parameter in parentheses.",
                            kw
                        ),
                    ))
                }
                "True" | "False" => {
                    let kw = id.clone();
                    Err(self.generate_error(
                        format!("Unknown identifier `{}`", kw),
                        format!(
                            "Boolean literals in Nova are lowercase: `true` and `false` (not `{}`).\n  \
                             Example: `let flag = true`",
                            kw
                        ),
                    ))
                }
                "this" => {
                    Err(self.generate_error(
                        "Unknown identifier `this`",
                        "Nova does not use `this`. Methods receive the instance as an explicit first parameter.\n  \
                         Example: `fn extends greet(p: Person) -> String { return p.name }`\n  \
                         The first parameter (commonly named `self`) is used instead of `this`.",
                    ))
                }
                "void" | "Void" => {
                    Err(self.generate_error(
                        "Cannot use `Void` as a value",
                        "Void is a return type, not a value. Functions that return nothing have return type Void.\n  \
                         If you want to represent \"no value\", use `None(T)` with the Option type.\n  \
                         Example: `let x: Option(Int) = None(Int)`",
                    ))
                }
                "mut" => {
                    Err(self.generate_error(
                        "Unknown keyword `mut`",
                        "Nova does not have a `mut` keyword — all variables are mutable by default.\n  \
                         Just use `let` to declare a variable.\n  \
                         Example: `let x = 5`  (x can be reassigned later with `x = 10`)",
                    ))
                }
                "lambda" => {
                    Err(self.generate_error(
                        "Unknown keyword `lambda`",
                        "Nova uses `fn` for named functions and `|args| expr` for closures/lambdas.\n  \
                         Closure example: `let double = |x: Int| x * 2`\n  \
                         Multi-line:      `let f = |x: Int| { let y = x + 1; y * 2 }`",
                    ))
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

        // Check for duplicate enum/struct name before initializing
        if self.typechecker.environment.no_override.has(&enum_name) {
            return Err(self.generate_error_with_pos(
                format!("Enum `{}` is already defined", enum_name),
                "Each enum name can only be defined once. Choose a different name or remove the duplicate definition.",
                position.clone(),
            ));
        }

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

        // Register `Self` as a type alias for this enum while parsing variants
        let self_type = TType::Custom {
            name: enum_name.clone(),
            type_params: generic_field_names
                .iter()
                .map(|g| TType::Generic { name: g.clone() })
                .collect(),
        };
        self.typechecker
            .environment
            .type_alias
            .insert("Self".into(), self_type);

        self.consume_symbol(LeftBrace)?;
        let parameter_list = self.enum_list()?;
        self.consume_symbol(RightBrace)?;

        // Remove the `Self` alias — it's only valid inside the enum body
        self.typechecker.environment.type_alias.remove("Self");
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
                        "Enum `{}` uses generic type `{}` but it is not declared",
                        enum_name, generic_type
                    ),
                    format!(
                        "Declare generic types in the enum header: `enum {}({}) {{ ... }}`\n  Example: `enum Option(T) {{ Some: $T, None }}`",
                        enum_name,
                        if generic_field_names.is_empty() {
                            generic_type.to_string()
                        } else {
                            format!("{}, {}", generic_field_names.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", "), generic_type)
                        }
                    ),
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

        self.typechecker
            .environment
            .no_override
            .insert(enum_name.clone());

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

        // Register `Self` as a type alias for this struct while parsing fields
        let self_type = TType::Custom {
            name: struct_name.clone(),
            type_params: generic_field_names
                .iter()
                .map(|g| TType::Generic { name: g.clone() })
                .collect(),
        };
        self.typechecker
            .environment
            .type_alias
            .insert("Self".into(), self_type);

        self.consume_symbol(LeftBrace)?;
        let parameter_list = self.parameter_list()?;
        self.consume_symbol(RightBrace)?;

        // Remove the `Self` alias — it's only valid inside the struct body
        self.typechecker.environment.type_alias.remove("Self");

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
                        "Struct `{}` uses generic type `{}` but it is not declared",
                        struct_name, generic_type
                    ),
                    format!(
                        "Declare generic types in the struct header: `struct {}({}) {{ ... }}`\n  Example: `struct Pair(T) {{ first: $T, second: $T }}`",
                        struct_name,
                        if generic_field_names.is_empty() {
                            generic_type.to_string()
                        } else {
                            format!("{}, {}", generic_field_names.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", "), generic_type)
                        }
                    ),
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
                format!("Struct `{}` is already defined", struct_name),
                "Each struct name can only be defined once. Choose a different name or remove the duplicate definition.".to_string(),
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

        // ── Destructuring foreach: `for (a, b) in list { … }` ──
        if self.current_token().is_some_and(|t| {
            t.is_symbol(LeftParen) || t.is_symbol(LeftSquareBracket)
        }) {
            return self.for_destructure();
        }
        // Check for struct destructuring: `for Ident { ... } in list { … }`
        if let Some(TokenValue::Identifier(id)) = self.current_token_value().cloned() {
            if self.peek_offset_value(1) == Some(&TokenValue::StructuralSymbol(LeftBrace)) {
                if self.typechecker.environment.custom_types.contains_key(id.as_ref())
                    && !self.typechecker.environment.enums.has(&id)
                {
                    return self.for_destructure();
                }
            }
        }

        if let Some(Keyword(KeyWord::In)) = self.peek_offset_value(1) {
            // Handle foreach statement

            let (identifier, pos) = self.get_identifier()?;
            if self.typechecker.environment.has(&identifier) {
                return Err(self.generate_error_with_pos(
                    format!("Variable `{}` is already defined in this scope", identifier),
                    format!("`{}` already exists. Choose a different loop variable name.", identifier),
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
                            format!("`for..in` can only iterate over lists, found `{}`", array.get_type()),
                            format!("The expression has type `{}`, but `for..in` requires a `[T]` (list).\n  Example: `for item in my_list {{ ... }}`\n  For ranges, use: `for i in 0 ..< 10 {{ ... }}`", array.get_type()),
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
                    format!("For-loop condition must be a Bool, found `{}`", test.get_type()),
                    "The middle expression in `for init; condition; step { ... }` must be a Bool.\n  Use a comparison like `i < 10`, `i != 0`, etc.",
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

    /// Parse `for (a, b) in list { … }`, `for [h, ..t] in list { … }`,
    /// or `for StructName { fields } in list { … }`.
    fn for_destructure(&mut self) -> NovaResult<Option<Statement>> {
        let pattern = self.parse_single_pattern()?;

        // Only allow irrefutable patterns
        match &pattern {
            Pattern::Tuple(_) | Pattern::List(_) | Pattern::ListCons(_, _)
            | Pattern::Variable(_) | Pattern::Wildcard | Pattern::EmptyList
            | Pattern::Struct { .. } => {}
            _ => {
                return Err(self.generate_error_with_pos(
                    "Invalid pattern in for destructuring".to_string(),
                    "Only tuple `(a, b)`, list `[a, b]`, cons `[h, ..t]`, and struct `Name { f1, f2 }` patterns are allowed in for loops.",
                    self.get_current_token_position(),
                ));
            }
        }

        self.consume_keyword(KeyWord::In)?;
        let arraypos = self.get_current_token_position();
        let array = self.expr()?;

        // Must be a list
        let inner_type = if let TType::List { inner } = array.get_type() {
            *inner
        } else {
            return Err(self.generate_error_with_pos(
                format!("`for..in` can only iterate over lists, found `{}`", array.get_type()),
                "The expression must be a list type `[T]`.",
                arraypos.clone(),
            ));
        };

        // Validate tuple arity if tuple pattern
        if let Pattern::Tuple(pats) = &pattern {
            if let TType::Tuple { elements } = &inner_type {
                if pats.len() != elements.len() {
                    return Err(self.generate_error_with_pos(
                        format!(
                            "Tuple pattern has {} elements but list elements have {}",
                            pats.len(),
                            elements.len()
                        ),
                        "The number of pattern elements must match the tuple size.",
                        arraypos.clone(),
                    ));
                }
            }
        }

        // Validate struct pattern against inner type
        if let Pattern::Struct { name, fields } = &pattern {
            match &inner_type {
                TType::Custom { name: type_name, .. } => {
                    if name != type_name {
                        return Err(self.generate_error_with_pos(
                            format!("Struct pattern `{}` does not match list element type `{}`", name, type_name),
                            format!(
                                "The struct name in the pattern must match the list element type.\n  \
                                 The list contains `{}`, but the pattern uses `{}`.\n  \
                                 Example: `for {} {{ ... }} in list {{ ... }}`",
                                type_name, name, type_name
                            ),
                            arraypos.clone(),
                        ));
                    }
                    if let Some(struct_fields) = self.typechecker.environment.custom_types.get(name.as_ref()) {
                        let real_fields: Vec<&(Rc<str>, TType)> = struct_fields.iter()
                            .filter(|(n, _)| n.as_ref() != "type")
                            .collect();
                        // Check for `_` wildcard sentinel (discards remaining fields)
                        let has_wildcard = fields.iter().any(|(n, p)| n.as_ref() == "_" && *p == Pattern::Wildcard);
                        let explicit_fields: Vec<&(Rc<str>, Pattern)> = fields.iter()
                            .filter(|(n, p)| !(n.as_ref() == "_" && *p == Pattern::Wildcard))
                            .collect();
                        if has_wildcard {
                            if explicit_fields.len() >= real_fields.len() {
                                return Err(self.generate_error_with_pos(
                                    format!(
                                        "Struct pattern `{}` uses `_` but already names all {} fields",
                                        name, real_fields.len()
                                    ),
                                    "`_` discards the remaining fields, but all fields are already named.",
                                    arraypos.clone(),
                                ));
                            }
                        } else if fields.len() != real_fields.len() {
                            let field_names: Vec<String> = real_fields.iter()
                                .map(|(n, _)| n.to_string())
                                .collect();
                            return Err(self.generate_error_with_pos(
                                format!(
                                    "Struct pattern `{}` has {} fields but the struct has {}",
                                    name, fields.len(), real_fields.len()
                                ),
                                format!(
                                    "All fields must be listed. Use `_` to discard a field.\n  \
                                     Expected: `for {} {{ {} }} in list {{ ... }}`",
                                    name, field_names.join(", ")
                                ),
                                arraypos.clone(),
                            ));
                        }
                        for (field_name, _) in &explicit_fields {
                            if !real_fields.iter().any(|(n, _)| n == field_name) {
                                let available: Vec<String> = real_fields.iter()
                                    .map(|(n, _)| format!("`{}`", n))
                                    .collect();
                                return Err(self.generate_error_with_pos(
                                    format!("Field `{}` not found in struct `{}`", field_name, name),
                                    format!(
                                        "Available fields: {}.\n  \
                                         Struct field names must match the definition exactly.\n  \
                                         Example: `for {} {{ {} }} in list {{ ... }}`",
                                        available.join(", "),
                                        name,
                                        real_fields.iter().map(|(n, _)| n.to_string()).collect::<Vec<_>>().join(", ")
                                    ),
                                    arraypos.clone(),
                                ));
                            }
                        }
                    }
                }
                TType::Any => {}
                _ => {
                    return Err(self.generate_error_with_pos(
                        format!("Cannot destructure `{}` with a struct pattern", inner_type),
                        "Struct patterns can only destructure struct types.\n  \
                         Struct destructuring works with lists of structs:\n  \
                         `for Point {{ x, y }} in list_of_points {{ ... }}`",
                        arraypos.clone(),
                    ));
                }
            }
        }

        self.typechecker.environment.push_block();
        self.register_pattern_bindings(&pattern, &inner_type)?;
        let body = self.block()?;
        self.typechecker.environment.pop_block();

        Ok(Some(Statement::ForeachDestructure {
            pattern,
            expr: array,
            body,
            position: arraypos,
        }))
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
                    format!("`while let` expects an Option type, found `{}`", expr.get_type()),
                    "The `while let` pattern loops while an Option has a value.\n  Syntax: `while let variable = option_expression { ... }`\n  The expression must return an Option type.".to_string(),
                    pos.clone(),
                ));
            };

            // make sure symbol doesn't already exist
            if self.typechecker.environment.has(&identifier) {
                Err(self.generate_error_with_pos(
                    format!("Variable `{}` is already defined in this scope", identifier),
                    format!("`{}` already exists. Choose a different variable name for `while let`.", identifier),
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
            let test = self.logical_or_expr()?;
            if test.get_type() != TType::Bool && test.get_type() != TType::Void {
                return Err(self.generate_error_with_pos(
                    format!("While-loop condition must be a Bool, found `{}`", test.get_type()),
                    "The condition in `while <expr> { ... }` must evaluate to a Bool.\n  Use a comparison like `x > 0`, `!done`, etc.",
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
                    format!("`if let` expects an Option type, found `{}`", expr.get_type()),
                    "The `if let` pattern unwraps an Option value.\n  Syntax: `if let variable_name = option_expression { ... }`\n  Example:\n    let opt: Option(Int) = Some(42)\n    if let value = opt {\n      println(Cast::string(value))\n    }\n  Note: Do NOT use `if let Some(x) = ...` — just use `if let x = ...`".to_string(),
                    pos.clone(),
                ));
            };

            // make sure symbol doesn't already exist
            if self.typechecker.environment.has(&identifier) {
                Err(self.generate_error_with_pos(
                    format!("Variable `{}` is already defined in this scope", identifier),
                    format!("`{}` already exists. Choose a different variable name for `if let`.", identifier),
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
                    let else_pos = self.get_current_token_position();
                    self.advance();
                    if self.current_token().is_some_and(|t| t.is_id("if")) {
                        return Err(self.generate_error_with_pos(
                            "Unexpected `else if` — Nova uses `elif`",
                            "Nova does not support `else if`. Use `elif` instead.\n  Example: `elif condition { ... }`",
                            else_pos,
                        ));
                    }
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
                    format!("Condition must be a Bool, found `{}`", test.get_type()),
                    format!(
                        "The condition expression returned `{}` but `if` requires a Bool.\n  Use a comparison like `x > 0`, `x == 0`, `x != \"\"`, etc.",
                        test.get_type()
                    ),
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
                let else_pos = self.get_current_token_position();
                self.advance();
                if self.current_token().is_some_and(|t| t.is_id("if")) {
                    return Err(self.generate_error_with_pos(
                        "Unexpected `else if` — Nova uses `elif`",
                        "Nova does not support `else if`. Use `elif` instead.\n  Example: `elif condition { ... }`",
                        else_pos,
                    ));
                }
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
        let pos = self.get_current_token_position();

        // ── Destructuring let: `let (a, b) = expr`, `let [h, ..t] = expr`,
        //    `let StructName { fields } = expr` ──
        if self.current_token().is_some_and(|t| {
            t.is_symbol(LeftParen) || t.is_symbol(LeftSquareBracket)
        }) {
            return self.let_destructure(pos);
        }
        // Check for struct destructuring: `let Ident { ... } = expr`
        if let Some(TokenValue::Identifier(id)) = self.current_token_value().cloned() {
            if self.peek_offset_value(1) == Some(&TokenValue::StructuralSymbol(LeftBrace)) {
                // Only dispatch if the identifier is a known struct (not an enum)
                if self.typechecker.environment.custom_types.contains_key(id.as_ref())
                    && !self.typechecker.environment.enums.has(&id)
                {
                    return self.let_destructure(pos);
                }
            }
        }

        let mut global = false;
        // refactor out into two parsing ways for ident. one with module and one without
        let (mut identifier, mut pos) = self.get_identifier()?;
        if self.modules.has(&identifier) {
            // throw error
            return Err(self.generate_error_with_pos(
                format!("Cannot use module name `{}` as a variable", identifier),
                format!("`{}` is already used as a module name. Choose a different variable name.", identifier),
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
                    // ── Fix: detect uncalled enum/struct constructors ──
                    let hint = if let TType::Function { return_type, parameters } = expr.get_type() {
                        if *return_type == ttype {
                            if parameters.len() == 1 && parameters[0] == TType::None {
                                // Nullary constructor like `Color::Red` — needs `()`
                                format!(
                                    "This is a constructor function, not a value — it needs to be called.\n  \
                                     Add `()` after the variant name to construct the value.\n  \
                                     Example: `let {id}: {ty} = {ty}::VariantName()`",
                                    id = identifier, ty = ttype
                                )
                            } else {
                                let param_types: Vec<String> = parameters.iter().map(|p| format!("{}", p)).collect();
                                let placeholders: Vec<&str> = parameters.iter().map(|p| match p {
                                    TType::Int => "0",
                                    TType::Float => "0.0",
                                    TType::Bool => "true",
                                    TType::String => "\"value\"",
                                    TType::Char => "'a'",
                                    _ => "value",
                                }).collect();
                                format!(
                                    "This is a constructor that takes ({params}) and returns `{ty}`. \
                                     Call it with arguments to create the value.\n  \
                                     Example: `let {id}: {ty} = {ty}::VariantName({args})`",
                                    params = param_types.join(", "), ty = ttype,
                                    id = identifier,
                                    args = placeholders.join(", ")
                                )
                            }
                        } else {
                            format!("The declared type is `{}` but the expression returns `{}`.\n  Make sure the right-hand side matches the declared type.",
                                ttype, expr.get_type())
                        }
                    } else {
                        format!("The declared type is `{}` but the expression returns `{}`.\n  Make sure the right-hand side matches the declared type.",
                            ttype, expr.get_type())
                    };
                    return Err(self.generate_error_with_pos(
                        format!("Cannot assign `{}` to `{}`", expr.get_type(), ttype),
                        hint,
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
                format!("Variable `{}` cannot be assigned to Void", identifier),
                "The expression does not return a value (returns Void).\n  Make sure the right-hand side is an expression that produces a value, not a statement.",
                pos.clone(),
            ));
        }

        // `_` is a discard — it never enters the scope and compiles to a POP.
        // It can be used any number of times in the same scope.
        if identifier.deref() == "_" {
            return Ok(Expr::Let {
                ttype: TType::Void,
                identifier,
                expr: Box::new(expr),
                global: false,
            });
        }

        // make sure symbol doesnt already exist
        if self.typechecker.environment.has(&identifier) {
            Err(self.generate_error_with_pos(
                format!("Variable `{}` is already defined in this scope", identifier),
                format!("A variable named `{}` already exists. Use a different name, or use `{} = <expr>` to reassign.", identifier, identifier),
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

    /// Parse `let (a, b) = expr` or `let [h, ..t] = expr` destructuring.
    /// Called from `let_expr` when the token after `let` is `(` or `[`.
    fn let_destructure(&mut self, pos: FilePosition) -> NovaResult<Expr> {
        let pattern = self.parse_single_pattern()?;

        // Only allow irrefutable patterns (tuple, list, variable, wildcard, listcons, struct)
        match &pattern {
            Pattern::Tuple(_) | Pattern::List(_) | Pattern::ListCons(_, _)
            | Pattern::Variable(_) | Pattern::Wildcard | Pattern::EmptyList
            | Pattern::Struct { .. } => {}
            _ => {
                return Err(self.generate_error_with_pos(
                    "Invalid pattern in let destructuring".to_string(),
                    "Only tuple `(a, b)`, list `[a, b]`, cons `[h, ..t]`, and struct `Name { f1, f2 }` patterns are allowed in let destructuring.",
                    pos.clone(),
                ));
            }
        }

        // Let destructuring requires irrefutable patterns — no literals allowed
        if !Self::is_irrefutable(&pattern) {
            return Err(self.generate_error_with_pos(
                "Refutable pattern in let destructuring".to_string(),
                "Let destructuring requires patterns that always match.\n  Literal values (like `0`, `\"hello\"`, `true`) are not allowed in let patterns.\n  Use `match` instead if you need to match specific values.",
                pos.clone(),
            ));
        }

        self.consume_operator(Operator::Assignment)?;
        let expr = self.expr()?;
        let expr_type = expr.get_type();

        // Void check
        if expr_type == TType::Void {
            return Err(self.generate_error_with_pos(
                "Cannot destructure a Void expression".to_string(),
                "The right-hand side does not return a value.",
                pos.clone(),
            ));
        }

        // Validate tuple arity
        if let Pattern::Tuple(pats) = &pattern {
            if let TType::Tuple { elements } = &expr_type {
                if pats.len() != elements.len() {
                    return Err(self.generate_error_with_pos(
                        format!(
                            "Tuple pattern has {} elements but expression has {}",
                            pats.len(),
                            elements.len()
                        ),
                        "The number of pattern elements must match the tuple size.",
                        pos.clone(),
                    ));
                }
            } else if !matches!(expr_type, TType::Any) {
                return Err(self.generate_error_with_pos(
                    format!("Cannot destructure `{}` with a tuple pattern", expr_type),
                    "Tuple patterns can only destructure tuple types.",
                    pos.clone(),
                ));
            }
        }

        // Validate list pattern against list type
        if matches!(&pattern, Pattern::List(_) | Pattern::ListCons(_, _) | Pattern::EmptyList) {
            if !matches!(expr_type, TType::List { .. } | TType::Any) {
                return Err(self.generate_error_with_pos(
                    format!("Cannot destructure `{}` with a list pattern", expr_type),
                    "List patterns can only destructure list types.",
                    pos.clone(),
                ));
            }
        }

        // Validate struct pattern: type, field count, field names, sub-pattern types
        if let Pattern::Struct { name, fields } = &pattern {
            match &expr_type {
                TType::Custom { name: type_name, .. } => {
                    if name != type_name {
                        return Err(self.generate_error_with_pos(
                            format!("Struct pattern `{}` does not match type `{}`", name, type_name),
                            format!(
                                "The struct name in the pattern must match the expression type.\n  \
                                 The value has type `{}`, but the pattern uses `{}`.\n  \
                                 Example: `let {} {{ ... }} = expr`",
                                type_name, name, type_name
                            ),
                            pos.clone(),
                        ));
                    }
                    if let Some(struct_fields) = self.typechecker.environment.custom_types.get(name.as_ref()) {
                        let real_fields: Vec<&(Rc<str>, TType)> = struct_fields.iter()
                            .filter(|(n, _)| n.as_ref() != "type")
                            .collect();
                        // Check for `_` wildcard sentinel (discards remaining fields)
                        let has_wildcard = fields.iter().any(|(n, p)| n.as_ref() == "_" && *p == Pattern::Wildcard);
                        let explicit_fields: Vec<&(Rc<str>, Pattern)> = fields.iter()
                            .filter(|(n, p)| !(n.as_ref() == "_" && *p == Pattern::Wildcard))
                            .collect();
                        if has_wildcard {
                            if explicit_fields.len() >= real_fields.len() {
                                return Err(self.generate_error_with_pos(
                                    format!(
                                        "Struct pattern `{}` uses `_` but already names all {} fields",
                                        name, real_fields.len()
                                    ),
                                    "`_` discards the remaining fields, but all fields are already named.",
                                    pos.clone(),
                                ));
                            }
                        } else if fields.len() != real_fields.len() {
                            let field_names: Vec<String> = real_fields.iter()
                                .map(|(n, _)| n.to_string())
                                .collect();
                            return Err(self.generate_error_with_pos(
                                format!(
                                    "Struct pattern `{}` has {} fields but the struct has {}",
                                    name, fields.len(), real_fields.len()
                                ),
                                format!(
                                    "All fields must be listed. Use `_` to discard a field.\n  \
                                     Expected: `let {} {{ {} }} = expr`",
                                    name, field_names.join(", ")
                                ),
                                pos.clone(),
                            ));
                        }
                        for (field_name, sub_pat) in &explicit_fields {
                            let found = real_fields.iter().find(|(n, _)| n == field_name);
                            if found.is_none() {
                                let available: Vec<String> = real_fields.iter()
                                    .map(|(n, _)| format!("`{}`", n))
                                    .collect();
                                return Err(self.generate_error_with_pos(
                                    format!("Field `{}` not found in struct `{}`", field_name, name),
                                    format!(
                                        "Available fields: {}.\n  \
                                         Struct field names must match the definition exactly.\n  \
                                         Example: `let {} {{ {} }} = expr`",
                                        available.join(", "),
                                        name,
                                        real_fields.iter().map(|(n, _)| n.to_string()).collect::<Vec<_>>().join(", ")
                                    ),
                                    pos.clone(),
                                ));
                            }
                            let field_ty = found.map(|(_, t)| t.clone()).unwrap_or(TType::Any);
                            self.validate_pattern_type(sub_pat, &field_ty, &pos)?;
                        }
                    }
                }
                TType::Any => {}
                _ => {
                    return Err(self.generate_error_with_pos(
                        format!("Cannot destructure `{}` with a struct pattern", expr_type),
                        "Struct patterns can only destructure struct types.",
                        pos.clone(),
                    ));
                }
            }
        }

        // Reorder struct pattern fields to match definition order (compiler uses positional indexing)
        // Also resolve nested option patterns
        let pattern = self.resolve_option_patterns(pattern, &expr_type);

        // Register pattern bindings in scope
        self.register_pattern_bindings(&pattern, &expr_type)?;

        Ok(Expr::LetDestructure {
            ttype: TType::Void,
            pattern,
            expr: Box::new(expr),
            position: pos,
        })
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
                        format!("Type `{}` does not exist", custom_type),
                        format!("`{}` is not a defined struct or enum.\n  `fn extends` can only extend existing custom types or built-in types.\n  Syntax: `fn extends({}) method_name(self: {}, ...) -> ReturnType {{ ... }}`",
                            custom_type, custom_type, custom_type),
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
                    format!("Module `{}` does not exist", custom_type),
                    format!("`{}` is not a defined module.\n  `fn mod` can only add functions to existing modules.\n  Make sure the module is imported or declared before this function.", custom_type),
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
                        format!("Dunder method `{}` requires exactly 2 parameters", id),
                        format!("Got {} parameter(s).\n  Dunder methods define operator overloads and must take exactly 2 parameters (left and right operands).\n  Example: `fn extends {} (a: MyType, b: MyType) -> MyType {{ ... }}`", parameters.len(), id),
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
                        format!("Cannot define dunder method `{}` on a module", id),
                        "Dunder methods define operator overloads and must extend a custom type, not a module.",
                        pos.clone(),
                    ));
                }
                if !get_first {
                    return Err(self.generate_error_with_pos(
                        format!("Dunder method `{}` must use `extends` from the first parameter", id),
                        format!("Dunder methods must extend from the first parameter's type.\n  Example: `fn extends {}(a: MyType, b: MyType) -> MyType {{ ... }}`", id),
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
                            format!("Cannot extend from type `{}`", ttype),
                            "Only custom types (structs/enums) and built-in types (Int, Float, Bool, String, Char, List, Option, Tuple, Function) can be extended.\n  Use `fn extends method_name(self: Type, ...) -> R {{ ... }}` syntax.",
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
                        format!("Parameter name `{}` conflicts with an existing function", &identifier),
                        format!("A function named `{}` already exists in scope.\n  Use a different parameter name to avoid shadowing.", &identifier),
                        pos.clone(),
                    ));
                }
                // check if normal function exist
                if self.typechecker.environment.has(&identifier) {
                    return Err(self.generate_error_with_pos(
                        format!("Parameter name `{}` conflicts with an existing function", &identifier),
                        format!("A function named `{}` already exists in scope.\n  Use a different parameter name to avoid shadowing.", &identifier),
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
                        "Function `{}` with parameter types ({}) is already defined",
                        identifier,
                        typeinput
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                    ),
                    "A function with the same name and parameter types already exists.\n  Nova supports function overloading — use different parameter types to create a new overload.",
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
                        "Cannot create generic function `{}` — a non-generic overload already exists",
                        &identifier
                    ),
                    "A non-generic function with this name already exists and cannot be overridden by a generic version.\n  Rename the function or remove the existing overload.",
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
            .map(|cap| cap.iter().map(|v| v.0.clone()).collect())
            .unwrap_or_default();

        self.typechecker.environment.pop_scope();
        self.typechecker.environment.live_generics.pop();
        for c in captured.iter() {
            if let Some(mc) = self.typechecker.environment.get_type_capture(&c.clone()) {
                let pos = self.get_current_token_position();

                if let Some(cap) = self.typechecker.environment.captured.last_mut() {
                    cap.insert(
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
        }

        captured = self
            .typechecker
            .environment
            .captured
            .last()
            .map(|cap| cap.iter().map(|v| v.0.clone()).collect())
            .unwrap_or_default();
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
            if let Some(scope) = self.typechecker.environment.values.last() {
                if let Some(v) = scope.get(dc) {
                    if !matches!(v.kind, SymbolKind::Captured) {
                        if let Some(cap) = self.typechecker.environment.captured.last_mut() {
                            cap.remove(dc);
                        }
                    }
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
                    "Function is missing a `return` statement in one or more branches",
                    "All code paths must return a value when the function has a non-Void return type.\n  Make sure every `if`/`elif`/`else` branch has a `return` statement.",
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

    /// Determine the result type produced by the last statement of a block.
    ///
    /// Returns `Some(ttype)` when the tail statement is an expression, an
    /// if/else whose branches both produce the same type, or a match
    /// whose arms all produce the same type.  Returns `None` when the
    /// block cannot be used as an expression (Void).
    fn tail_type(stmts: &[Statement]) -> Option<TType> {
        match stmts.last() {
            Some(Statement::Expression { ttype, .. }) => {
                if *ttype == TType::None {
                    None
                } else {
                    Some(ttype.clone())
                }
            }
            Some(Statement::If {
                body,
                alternative: Some(alt),
                ..
            }) => {
                let body_ty = Self::tail_type(body)?;
                let alt_ty = Self::tail_type(alt)?;
                if body_ty == alt_ty {
                    Some(body_ty)
                } else {
                    None
                }
            }
            Some(Statement::Match {
                arms,
                default,
                ..
            }) => {
                let mut iter_type: Option<TType> = None;
                for (_, _, arm_body) in arms {
                    let arm_ty = Self::tail_type(arm_body)?;
                    if let Some(ref prev) = iter_type {
                        if *prev != arm_ty {
                            return None;
                        }
                    } else {
                        iter_type = Some(arm_ty);
                    }
                }
                if let Some(def) = default {
                    let def_ty = Self::tail_type(def)?;
                    if let Some(ref prev) = iter_type {
                        if *prev != def_ty {
                            return None;
                        }
                    } else {
                        iter_type = Some(def_ty);
                    }
                }
                iter_type
            }
            _ => None,
        }
    }

    fn block_expr(&mut self) -> NovaResult<Expr> {
        self.consume_symbol(LeftBrace)?;
        self.typechecker.environment.push_block();
        let statements = self.compound_statement()?;
        self.typechecker.environment.pop_block();
        self.consume_symbol(RightBrace)?;
        // Determine the block's result type from the last statement
        let ttype = Self::tail_type(&statements).unwrap_or(TType::Void);
        Ok(Expr::Block {
            body: statements,
            ttype,
        })
    }

    fn compound_statement(&mut self) -> NovaResult<Vec<Statement>> {
        let mut initial_statements = vec![];
        let mut last_stmt_pos = self.get_current_token_position();
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

                // Must-use check: before parsing the next statement, verify that
                // the PREVIOUS statement (if it was an expression) didn't silently
                // drop an Option value.  The last statement is exempt because it
                // may serve as a tail expression (block result, implicit return, etc.).
                if let Some(Statement::Expression { ttype: TType::Option { inner }, .. }) = statements.last() {
                    return Err(self.generate_error_with_pos(
                        format!(
                            "Option({}) value must be used — it cannot be silently discarded",
                            inner
                        ),
                        "An Option value was used as a statement but its result is being dropped.\n  \
                         Handle it with one of:\n  \
                         • `?` to unwrap:          `readFile(\"data.txt\")?`\n  \
                         • `if let` to match:      `if let data = readFile(\"data.txt\") { ... }`\n  \
                         • assign to a variable:   `let result = readFile(\"data.txt\")`",
                        last_stmt_pos.clone(),
                    ));
                }

                last_stmt_pos = self.get_current_token_position();
                if let Some(statement) = self.statement()? {
                    statements.push(statement);
                }
                if self.index == index_change {
                    return Err(self.generate_error("Expected statement", "A statement was expected but the parser could not continue.\n  Check for missing semicolons, extra tokens, or unclosed braces."));
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
                "Expected `module` declaration",
                "Every Nova file must begin with a module declaration.\n  Example: `module my_module`\n  This must be the first statement in the file.",
            ));
        }

        self.ast.program = self.compound_statement()?;
        self.eof()
    }
}
