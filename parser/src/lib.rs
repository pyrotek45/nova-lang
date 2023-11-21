use std::collections::HashMap;

use common::{
    error::{self, NovaError},
    nodes::{new_env, Arg, Ast, Atom, Env, Expr, Field, Statement, SymbolKind},
    table::{self, Table},
    tokens::{generate_unique_string, Operator, Position, TType, Token, TokenList, Unary},
};
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
    pub environment: Env,
}

pub fn new(filepath: &str) -> Parser {
    let mut env = new_env();
    env.insert_symbol(
        "isSome",
        TType::Function(
            vec![TType::Option(Box::new(TType::Generic("a".to_string())))],
            Box::new(TType::Bool),
        ),
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "unwrap",
        TType::Function(
            vec![TType::Option(Box::new(TType::Generic("a".to_string())))],
            Box::new(TType::Generic("a".to_string())),
        ),
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "none",
        TType::Function(
            vec![TType::None],
            Box::new(TType::Option(Box::new(TType::None))),
        ),
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "some",
        TType::Function(
            vec![TType::Generic("a".to_string())],
            Box::new(TType::Option(Box::new(TType::Generic("a".to_string())))),
        ),
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "free",
        TType::Function(vec![TType::Any], Box::new(TType::Void)),
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "print",
        TType::Function(vec![TType::Generic("a".to_string())], Box::new(TType::Void)),
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "println",
        TType::Function(vec![TType::Generic("a".to_string())], Box::new(TType::Void)),
        None,
        SymbolKind::GenericFunction,
    );
    env.insert_symbol(
        "clone",
        TType::Function(
            vec![TType::Generic("a".to_string())],
            Box::new(TType::Generic("a".to_string())),
        ),
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
    fn eof(&mut self) -> Result<(), NovaError> {
        if let Token::EOF(_) = self.current_token() {
            Ok(())
        } else {
            return Err(common::error::parser_error(
                "Parsing not completed, Left over tokens unparsed".to_string(),
                "Make sure your statement ends with ';' ".to_string(),
                self.current_token().line(),
                self.current_token().row(),
                self.filepath.clone(),
                None,
            ));
        }
    }

    fn eat_if_newline(&mut self) {
        if self.current_token().is_newline() {
            self.advance()
        }
    }

    fn generate_error(&self, msg: String, note: String) -> NovaError {
        error::parser_error(
            msg,
            note,
            self.current_token().line(),
            self.current_token().row(),
            self.filepath.clone(),
            None,
        )
    }

    fn get_line_and_row(&self) -> (usize, usize) {
        let line = self.current_token().line();
        let row = self.current_token().row();
        (line, row)
    }

    fn get_pos(&self) -> Position {
        Position {
            line: self.current_token().line(),
            row: self.current_token().row(),
        }
    }

    fn consume_operator(&mut self, op: Operator) -> Result<(), NovaError> {
        if let Token::Operator(oper, _) = self.current_token() {
            if op == oper {
                self.advance();
                return Ok(());
            }
        }
        Err(self.generate_error(
            format!("unexpected operator, got {:?}", self.current_token()),
            format!("expecting {:?}", op),
        ))
    }

    fn consume_symbol(&mut self, symbol: char) -> Result<(), NovaError> {
        if let Token::Symbol(sym, _) = self.current_token() {
            if sym == symbol {
                self.advance();
                return Ok(());
            }
        }

        Err(self.generate_error(
            format!("unexpected symbol, got {:?}", self.current_token()),
            format!("expecting {:?}", symbol),
        ))
    }

    fn consume_identifier(&mut self, symbol: Option<&str>) -> Result<(), NovaError> {
        match self.current_token() {
            Token::Identifier(sym, _) if symbol.map_or(true, |s| sym == s) => {
                self.advance();
                Ok(())
            }
            _ => {
                let current_token = self.current_token();
                return Err(self.generate_error(
                    format!("unexpected identifier, got {:?}", current_token),
                    match symbol {
                        Some(s) => format!("expecting {:?}", s),
                        None => "expecting an identifier".to_string(),
                    },
                ));
            }
        }
    }

    // refactor out to tokens file
    fn check_and_map_types(
        &self,
        type_list1: &[TType],
        type_list2: &[TType],
        type_map: &mut HashMap<String, TType>,
    ) -> Result<HashMap<String, TType>, NovaError> {
        if type_list1.len() != type_list2.len() {
            return Err(self.generate_error(
                "E2 Incorrect amount of arguments".to_owned(),
                format!("Got {:?} , but expexting {:?}", type_list2, type_list1),
            ));
        }
        for (t1, t2) in type_list1.iter().zip(type_list2.iter()) {
            match (t1, t2) {
                (TType::Any, t2) => {
                    if t2 != &TType::Void {
                        continue;
                    } else {
                        return Err(self.generate_error(
                            format!("Type error, expecting {:?}, but found {:?}", TType::Any, t2),
                            format!("expecting input, got void",),
                        ));
                    }
                }
                (TType::Generic(name1), _) => {
                    if t2 == &TType::None {
                        return Err(common::error::parser_error(
                            format!(
                                "Type error, expecting some value, but found {:?}",
                                t2
                            ),
                            format!("Cannot bind to a None value"),
                            self.current_token().line(),
                            self.current_token().row(),
                            self.filepath.clone(),
                            None,
                        ));
                    }
                    if let Some(mapped_type) = type_map.clone().get(name1) {
                        if mapped_type != t2 {
                            return Err(common::error::parser_error(
                                format!(
                                    "Type error, expecting {:?}, but found {:?}",
                                    mapped_type.clone(),
                                    t2
                                ),
                                format!(
                                    "{:?} != {:?} \n ~ {:?} -> {:?}\n ~ {:?}",
                                    mapped_type.clone(),
                                    t2,
                                    type_list1,
                                    mapped_type.clone(),
                                    type_list2
                                ),
                                self.current_token().line(),
                                self.current_token().row(),
                                self.filepath.clone(),
                                None,
                            ));
                        }
                    } else {
                        type_map.insert(name1.clone(), t2.clone());
                    }
                }
                (TType::List(inner1), TType::List(inner2)) => {
                    self.check_and_map_types(&[*inner1.clone()], &[*inner2.clone()], type_map)?;
                }
                (TType::Option(inner1), TType::Option(inner2)) => {
                    if**inner1 == TType::None {
                        continue;
                    } else {
                        self.check_and_map_types(&[*inner1.clone()], &[*inner2.clone()], type_map)?;
                    }
                }
                (TType::Function(params1, ret1), TType::Function(params2, ret2)) => {
                    if params1.len() != params2.len() {
                        return Err(self.generate_error(
                            format!("Function got incorrect arguments"),
                            format!(""),
                        ));
                    }

                    self.check_and_map_types(params1, params2, type_map)?;
                    self.check_and_map_types(&[*ret1.clone()], &[*ret2.clone()], type_map)?;
                }
                _ if t1 == t2 => continue,
                _ => {
                    return Err(self.generate_error(
                        format!("{:?} and {:?} do not match", t1, t2),
                        "Type error".to_owned(),
                    ));
                }
            }
        }
        Ok(type_map.clone())
    }

    fn get_output(
        &self,
        output: TType,
        type_map: &mut std::collections::HashMap<String, TType>,
    ) -> Result<TType, NovaError> {
        match output {
            TType::Generic(name) => {
                if let Some(mapped_type) = type_map.get(&name) {
                    Ok(mapped_type.clone())
                } else {
                    Ok(TType::Generic(name.clone()))
                }
            }
            TType::List(inner) => {
                let mapped_inner = self.get_output(*inner.clone(), type_map)?;
                Ok(TType::List(Box::new(mapped_inner)))
            }
            TType::Option(inner) => {
                let mapped_inner = self.get_output(*inner.clone(), type_map)?;
                Ok(TType::Option(Box::new(mapped_inner)))
            }
            TType::Function(params, ret) => {
                let mut mapped_params = Vec::new();
                for param in params {
                    let mapped_param = self.get_output(param, type_map)?;
                    mapped_params.push(mapped_param);
                }

                let mapped_ret = self.get_output(*ret.clone(), type_map)?;

                Ok(TType::Function(mapped_params, Box::new(mapped_ret)))
            }
            _ => Ok(output.clone()),
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
            Token::Operator(op, _) => match op {
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

    fn list_constructor(&mut self) -> Result<Vec<Expr>, NovaError> {
        let mut exprs = vec![];
        self.consume_symbol('[')?;
        self.eat_if_newline();
        if !self.current_token().is_symbol(']') {
            exprs.push(self.expr()?);
        }
        while self.current_token().is_symbol(',') || self.current_token().is_newline() {
            self.eat_if_newline();
            if self.current_token().is_symbol(']') {
                break;
            }
            self.advance();
            self.eat_if_newline();
            if self.current_token().is_symbol(']') {
                break;
            }
            let e = self.expr()?;
            if e.get_type() != TType::Void {
                exprs.push(e);
            } else {
                return Err(self.generate_error(
                    format!("cannot insert a void expression"),
                    format!("List expressions must not be void"),
                ));
            }
        }
        self.consume_symbol(']')?;
        Ok(exprs)
    }

    fn expr_list(&mut self) -> Result<Vec<Expr>, NovaError> {
        let mut exprs = vec![];
        self.consume_symbol('[')?;
        self.eat_if_newline();
        if !self.current_token().is_symbol(']') {
            exprs.push(self.expr()?);
        }
        while self.current_token().is_symbol(',') || self.current_token().is_newline() {
            self.eat_if_newline();
            if self.current_token().is_symbol(']') {
                break;
            }
            self.advance();
            self.eat_if_newline();
            if self.current_token().is_symbol(']') {
                break;
            }
            let e = self.expr()?;
            if e.get_type() != TType::Void {
                exprs.push(e);
            } else {
                return Err(self.generate_error(
                    format!("cannot insert a void expression"),
                    format!("List expressions must not be void"),
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
        conpos: Position,
    ) -> Result<Vec<Expr>, NovaError> {
        let mut exprs: HashMap<String, Expr> = HashMap::default();

        self.consume_symbol('{')?;
        self.eat_if_newline();
        let (id, _) = self.identifier()?;
        self.consume_operator(Operator::Assignment)?;
        exprs.insert(id.clone(), self.expr()?);

        while self.current_token().is_symbol(',') {
            self.eat_if_newline();
            self.advance();
            if self.current_token().is_symbol('}') {
                break;
            }
            self.eat_if_newline();
            if self.current_token().is_symbol('}') {
                break;
            }
            let (id, _) = self.identifier()?;
            self.consume_operator(Operator::Assignment)?;
            exprs.insert(id.clone(), self.expr()?);
            if self.current_token().is_symbol('}') {
                break;
            }
        }
        self.eat_if_newline();
        self.consume_symbol('}')?;

        let mut new_exprs = vec![];

        for (fieldname, fieldtype) in fields.iter() {
            if fieldname == "type" {
                continue;
            }
            if let Some(innerexpr) = exprs.get(fieldname) {
                self.check_and_map_types(
                    &vec![innerexpr.get_type()],
                    &vec![fieldtype.clone()],
                    &mut HashMap::default(),
                )?;
                new_exprs.push(innerexpr.clone())
            } else {
                return Err(common::error::parser_error(
                    format!("{} is missing field {} ", constructor, fieldname.clone()),
                    format!(""),
                    conpos.line,
                    conpos.row,
                    self.filepath.clone(),
                    None,
                ));
            }
        }

        if exprs.len() != fields.len() - 1 {
            return Err(common::error::parser_error(
                format!(
                    "{} has {} fields, you have {} ",
                    constructor,
                    fields.len() - 1,
                    exprs.len()
                ),
                format!(""),
                conpos.line,
                conpos.row,
                self.filepath.clone(),
                None,
            ));
        }

        if new_exprs.len() != fields.len() - 1 {
            return Err(common::error::parser_error(
                format!(
                    "{} has {} fields, not all of them are covered",
                    constructor,
                    fields.len() - 1
                ),
                format!(""),
                conpos.line,
                conpos.row,
                self.filepath.clone(),
                None,
            ));
        }

        Ok(new_exprs)
    }

    fn method(
        &mut self,
        identifier: String,
        first_argument: Expr,
        pos: Position,
    ) -> Result<Expr, NovaError> {
        let mut arguments = vec![first_argument];
        arguments.extend(self.argument_list()?);

        let mut inputtypes: Vec<TType> = arguments.iter().map(|t| t.get_type()).collect();
        if inputtypes.is_empty() {
            inputtypes.push(TType::None)
        }

        // get function type and check arguments
        if let Some((
            TType::Function(function_parameters, mut function_output),
            function_id,
            function_kind,
        )) = self.environment.get_function_type(&identifier, &inputtypes)
        {
            match function_kind {
                SymbolKind::GenericFunction => {
                    let mut map = self.check_and_map_types(
                        &function_parameters,
                        &inputtypes,
                        &mut HashMap::default(),
                    )?;
                    function_output = Box::new(self.get_output(*function_output, &mut map)?);
                    return Ok(Expr::Literal(
                        *function_output.clone(),
                        Atom::Call(function_id, arguments),
                    ));
                }
                SymbolKind::Function => {
                    let mut map = self.check_and_map_types(
                        &function_parameters,
                        &inputtypes,
                        &mut HashMap::default(),
                    )?;
                    function_output = Box::new(self.get_output(*function_output, &mut map)?);
                    return Ok(Expr::Literal(
                        *function_output.clone(),
                        Atom::Call(function_id, arguments),
                    ));
                }
                SymbolKind::Constructor => {
                    let mut map = self.check_and_map_types(
                        &function_parameters,
                        &inputtypes,
                        &mut HashMap::default(),
                    )?;
                    function_output = Box::new(self.get_output(*function_output, &mut map)?);
                    return Ok(Expr::Literal(
                        *function_output.clone(),
                        Atom::Call(function_id, arguments),
                    ));
                }
                SymbolKind::Variable => {
                    let mut map = self.check_and_map_types(
                        &function_parameters,
                        &inputtypes,
                        &mut HashMap::default(),
                    )?;
                    function_output = Box::new(self.get_output(*function_output, &mut map)?);
                    return Ok(Expr::Literal(
                        *function_output.clone(),
                        Atom::Call(function_id, arguments),
                    ));
                }
                SymbolKind::Parameter => {
                    let mut map = self.check_and_map_types(
                        &function_parameters,
                        &inputtypes,
                        &mut HashMap::default(),
                    )?;
                    function_output = Box::new(self.get_output(*function_output, &mut map)?);
                    return Ok(Expr::Literal(
                        *function_output.clone(),
                        Atom::Call(function_id, arguments),
                    ));
                }
            }
        } else {
            if let Some((
                TType::Function(function_parameters, mut function_output),
                function_id,
                function_kind,
            )) = self.environment.get_type_capture(&identifier)
            {
                self.environment.captured.last_mut().unwrap().insert(
                    identifier.clone(),
                    TType::Function(function_parameters.clone(), function_output.clone()),
                );
                match function_kind {
                    SymbolKind::GenericFunction => {
                        let mut map = self.check_and_map_types(
                            &function_parameters,
                            &inputtypes,
                            &mut HashMap::default(),
                        )?;
                        function_output = Box::new(self.get_output(*function_output, &mut map)?);
                        return Ok(Expr::Literal(
                            *function_output.clone(),
                            Atom::Call(function_id, arguments),
                        ));
                    }
                    SymbolKind::Constructor => {
                        let mut map = self.check_and_map_types(
                            &function_parameters,
                            &inputtypes,
                            &mut HashMap::default(),
                        )?;
                        function_output = Box::new(self.get_output(*function_output, &mut map)?);
                        return Ok(Expr::Literal(
                            *function_output.clone(),
                            Atom::Call(function_id, arguments),
                        ));
                    }
                    SymbolKind::Function => {
                        let mut map = self.check_and_map_types(
                            &function_parameters,
                            &inputtypes,
                            &mut HashMap::default(),
                        )?;
                        function_output = Box::new(self.get_output(*function_output, &mut map)?);
                        return Ok(Expr::Literal(
                            *function_output.clone(),
                            Atom::Call(function_id, arguments),
                        ));
                    }
                    SymbolKind::Variable => {
                        let mut map = self.check_and_map_types(
                            &function_parameters,
                            &inputtypes,
                            &mut HashMap::default(),
                        )?;
                        function_output = Box::new(self.get_output(*function_output, &mut map)?);
                        return Ok(Expr::Literal(
                            *function_output.clone(),
                            Atom::Call(function_id, arguments),
                        ));
                    }
                    SymbolKind::Parameter => {
                        let mut map = self.check_and_map_types(
                            &function_parameters,
                            &inputtypes,
                            &mut HashMap::default(),
                        )?;
                        function_output = Box::new(self.get_output(*function_output, &mut map)?);
                        return Ok(Expr::Literal(
                            *function_output.clone(),
                            Atom::Call(function_id, arguments),
                        ));
                    }
                }
            } else {
                return Err(common::error::parser_error(
                    format!("Not a valid call: {}", identifier),
                    format!(
                        "No function signature '{}' with {:?} as arguments",
                        identifier, inputtypes
                    ),
                    pos.line,
                    pos.row,
                    self.filepath.clone(),
                    None,
                ));
            }
        }
    }

    fn call(&mut self, identifier: String, pos: Position) -> Result<Expr, NovaError> {
        let arguments: Vec<Expr>;
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

        // get function type and check arguments
        if let Some((
            TType::Function(function_parameters, mut function_output),
            function_id,
            function_kind,
        )) = self
            .environment
            .get_function_type(&identifier, &argument_types)
        {
            match function_kind {
                SymbolKind::GenericFunction => {
                    let mut map = self.check_and_map_types(
                        &function_parameters,
                        &argument_types,
                        &mut HashMap::default(),
                    )?;
                    function_output = Box::new(self.get_output(*function_output, &mut map)?);
                    return Ok(Expr::Literal(
                        *function_output.clone(),
                        Atom::Call(function_id, arguments),
                    ));
                }
                SymbolKind::Constructor => {
                    let mut map = self.check_and_map_types(
                        &function_parameters,
                        &argument_types,
                        &mut HashMap::default(),
                    )?;
                    function_output = Box::new(self.get_output(*function_output, &mut map)?);
                    return Ok(Expr::Literal(
                        *function_output.clone(),
                        Atom::Call(function_id, arguments),
                    ));
                }
                SymbolKind::Function => {
                    let mut map = self.check_and_map_types(
                        &function_parameters,
                        &argument_types,
                        &mut HashMap::default(),
                    )?;
                    function_output = Box::new(self.get_output(*function_output, &mut map)?);
                    return Ok(Expr::Literal(
                        *function_output.clone(),
                        Atom::Call(function_id, arguments),
                    ));
                }
                SymbolKind::Variable => {
                    let mut map = self.check_and_map_types(
                        &function_parameters,
                        &argument_types,
                        &mut HashMap::default(),
                    )?;
                    function_output = Box::new(self.get_output(*function_output, &mut map)?);
                    return Ok(Expr::Literal(
                        *function_output.clone(),
                        Atom::Call(function_id, arguments),
                    ));
                }
                SymbolKind::Parameter => {
                    let mut map = self.check_and_map_types(
                        &function_parameters,
                        &argument_types,
                        &mut HashMap::default(),
                    )?;
                    function_output = Box::new(self.get_output(*function_output, &mut map)?);
                    return Ok(Expr::Literal(
                        *function_output.clone(),
                        Atom::Call(function_id, arguments),
                    ));
                }
            }
        } else {
            if let Some((
                TType::Function(function_parameters, mut function_output),
                function_id,
                function_kind,
            )) = self
                .environment
                .get_function_type_capture(&identifier, &argument_types)
            {
                self.environment.captured.last_mut().unwrap().insert(
                    identifier.clone(),
                    TType::Function(function_parameters.clone(), function_output.clone()),
                );

                match function_kind {
                    SymbolKind::GenericFunction => {
                        let mut map = self.check_and_map_types(
                            &function_parameters,
                            &argument_types,
                            &mut HashMap::default(),
                        )?;
                        function_output = Box::new(self.get_output(*function_output, &mut map)?);
                        return Ok(Expr::Literal(
                            *function_output.clone(),
                            Atom::Call(function_id, arguments),
                        ));
                    }
                    SymbolKind::Constructor => {
                        let mut map = self.check_and_map_types(
                            &function_parameters,
                            &argument_types,
                            &mut HashMap::default(),
                        )?;
                        function_output = Box::new(self.get_output(*function_output, &mut map)?);
                        return Ok(Expr::Literal(
                            *function_output.clone(),
                            Atom::Call(function_id, arguments),
                        ));
                    }
                    SymbolKind::Function => {
                        let mut map = self.check_and_map_types(
                            &function_parameters,
                            &argument_types,
                            &mut HashMap::default(),
                        )?;
                        function_output = Box::new(self.get_output(*function_output, &mut map)?);
                        return Ok(Expr::Literal(
                            *function_output.clone(),
                            Atom::Call(function_id, arguments),
                        ));
                    }
                    SymbolKind::Variable => {
                        let mut map = self.check_and_map_types(
                            &function_parameters,
                            &argument_types,
                            &mut HashMap::default(),
                        )?;
                        function_output = Box::new(self.get_output(*function_output, &mut map)?);
                        return Ok(Expr::Literal(
                            *function_output.clone(),
                            Atom::Call(function_id, arguments),
                        ));
                    }
                    SymbolKind::Parameter => {
                        let mut map = self.check_and_map_types(
                            &function_parameters,
                            &argument_types,
                            &mut HashMap::default(),
                        )?;
                        function_output = Box::new(self.get_output(*function_output, &mut map)?);
                        return Ok(Expr::Literal(
                            *function_output.clone(),
                            Atom::Call(function_id, arguments),
                        ));
                    }
                }
            } else {
                return Err(common::error::parser_error(
                    format!("Not a valid call: {}", identifier),
                    format!(
                        "E1 No function signature '{}' with {:?} as arguments",
                        identifier, argument_types
                    ),
                    pos.line,
                    pos.row,
                    self.filepath.clone(),
                    None,
                ));
            }
        }
    }

    fn field(
        &mut self,
        identifier: String,
        mut lhs: Expr,
        pos: Position,
    ) -> Result<Expr, NovaError> {
        if let Some(name) = lhs.get_type().custom_to_string() {
            if let Some(fields) = self.environment.custom_types.get(&name) {
                let mut found = false;
                for (index, (field_name, ttype)) in fields.iter().enumerate() {
                    if &identifier == field_name {
                        lhs = Expr::Field(ttype.clone(), name.clone(), index, Box::new(lhs));
                        found = true;
                        break;
                    }
                }
                if !found {
                    return Err(common::error::parser_error(
                        format!("No field '{}' found for {}", identifier, name),
                        format!("cannot retrieve field"),
                        pos.line,
                        pos.row,
                        self.filepath.clone(),
                        None,
                    ));
                }
            } else {
                return Err(common::error::parser_error(
                    format!("No field '{}' found for {}", identifier, name),
                    format!("cannot retrieve field"),
                    pos.line,
                    pos.row,
                    self.filepath.clone(),
                    None,
                ));
            }
        } else {
            return Err(common::error::parser_error(
                format!("{:?} has no '{}' field", lhs.get_type(), identifier),
                format!("cannot retrieve field"),
                pos.line,
                pos.row,
                self.filepath.clone(),
                None,
            ));
        }
        Ok(lhs)
    }

    fn chain(&mut self, mut lhs: Expr) -> Result<Expr, NovaError> {
        let (identifier, pos) = self.identifier()?;
        match self.current_token() {
            Token::Operator(Operator::DoubleColon, _) => {
                let mut rhs = lhs.clone();
                while self.current_token().is_op(Operator::DoubleColon) {
                    self.consume_operator(Operator::DoubleColon)?;
                    let (field, pos) = self.identifier()?;
                    if let Some(ctype) = self.environment.get_type(&identifier) {
                        rhs = self.field(
                            field.clone(),
                            Expr::Literal(ctype, Atom::Id(identifier.clone())),
                            pos,
                        )?;
                    } else {
                        dbg!(&lhs, &rhs);
                        panic!()
                    }
                }
                // function pointer return call <func()(args)>
                let mut arguments = vec![lhs.clone()];
                arguments.extend(self.argument_list()?);
                if let TType::Function(argtypes, mut output) = rhs.get_type() {
                    if arguments.len() != argtypes.len() {
                        return Err(self.generate_error(
                            format!("E1 Inccorrect number of arguments"),
                            format!("Got {:?}, expected {:?}", arguments.len(), argtypes.len()),
                        ));
                    }
                    let mut inputtypes = vec![];
                    for t in arguments.iter() {
                        inputtypes.push(t.get_type())
                    }
                    let mut map: HashMap<String, TType> = HashMap::default();
                    map = self.check_and_map_types(&argtypes, &inputtypes, &mut map)?;
                    output = Box::new(self.get_output(*output.clone(), &mut map)?);
                    lhs = Expr::Call(*output, "anon".to_string(), Box::new(rhs), arguments);
                } else {
                    return Err(self.generate_error(
                        format!("Cant call {:?}", lhs.get_type()),
                        format!("not a function"),
                    ));
                }
            }
            Token::Symbol('(', _) => {
                lhs = self.method(identifier.clone(), lhs, pos)?;
            }
            Token::Symbol('[', _) => {
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
        self.consume_symbol('[')?;
        let index = self.mid_expr()?;
        self.consume_symbol(']')?;
        if index.get_type() != TType::Int {
            return Err(self.generate_error(
                format!("Must index list with an int"),
                format!("Cannot index into list with {:?}", index.get_type()),
            ));
        }
        if let Some(inner) = ttype.get_inner() {
            lhs = Expr::Indexed(
                inner.clone(),
                identifier.clone(),
                Box::new(index),
                Box::new(lhs),
            );
            if self.current_token().is_symbol('[') {
                lhs = self.index(identifier.clone(), lhs, inner)?;
            }
        } else {
            return Err(self.generate_error(
                format!("Cannot index into non list"),
                format!("Must be of type list"),
            ));
        }
        Ok(lhs)
    }

    fn anchor(&mut self, identifier: String, pos: Position) -> Result<Expr, NovaError> {
        let anchor = match self.current_token() {
            Token::Symbol('[', _) => {
                if let Some(ttype) = self.environment.get_type(&identifier) {
                    self.index(
                        identifier.clone(),
                        Expr::Literal(ttype.clone(), Atom::Id(identifier.clone())),
                        ttype.clone(),
                    )?
                } else {
                    if let Some((ttype, _, kind)) = self.environment.get_type_capture(&identifier) {
                        self.environment
                            .captured
                            .last_mut()
                            .unwrap()
                            .insert(identifier.clone(), ttype.clone());
                        self.environment.insert_symbol(
                            &identifier,
                            ttype.clone(),
                            Some(pos.clone()),
                            kind,
                        );
                        self.index(
                            identifier.clone(),
                            Expr::Literal(ttype.clone(), Atom::Id(identifier.clone())),
                            ttype.clone(),
                        )?
                    } else {
                        return Err(common::error::parser_error(
                            format!("E1 Not a valid symbol: {}", identifier),
                            format!("Unknown identifier"),
                            pos.line,
                            pos.row,
                            self.filepath.clone(),
                            None,
                        ));
                    }
                }
            }
            Token::Symbol('(', _) => self.call(identifier.clone(), pos)?,
            _ => {
                if self.current_token().is_symbol('{')
                    && self.environment.custom_types.contains_key(&identifier)
                {
                    self.call(identifier.clone(), pos.clone())?
                } else {
                    if let Some(ttype) = self.environment.get_type(&identifier) {
                        Expr::Literal(ttype.clone(), Atom::Id(identifier.clone()))
                    } else {
                        if let Some((ttype, _, kind)) =
                            self.environment.get_type_capture(&identifier)
                        {
                            self.environment
                                .captured
                                .last_mut()
                                .unwrap()
                                .insert(identifier.clone(), ttype.clone());
                            self.environment.insert_symbol(
                                &identifier,
                                ttype.clone(),
                                Some(pos.clone()),
                                kind,
                            );
                            Expr::Literal(ttype.clone(), Atom::Id(identifier.clone()))
                        } else {
                            return Err(common::error::parser_error(
                                format!("E2 Not a valid symbol: {}", identifier),
                                format!("Unknown identifier"),
                                pos.line,
                                pos.row,
                                self.filepath.clone(),
                                None,
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
            Token::Char(char, _) => {
                self.advance();
                left = Expr::Literal(TType::Char, Atom::Char(char))
            }
            Token::Identifier(id, _) if id.as_str() == "fn" => {
                let pos = self.get_pos();
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
                    if let TType::Function(_, _) = ttype.clone() {
                        // check if generic function exist
                        if self.environment.has(&identifier) {
                            return Err(self.generate_error(
                                format!("Generic Function {} already defined", &identifier),
                                "Cannot redefine a generic function".to_string(),
                            ));
                        }
                        // check if normal function exist
                        if self.environment.has(&identifier) {
                            return Err(self.generate_error(
                                format!("Function {} already defined", &identifier,),
                                "Cannot redefine a generic function".to_string(),
                            ));
                        }
                        // build argument list
                        input.push(Arg {
                            identifier: identifier,
                            ttype: ttype.clone(),
                        });
                    } else {
                        input.push(Arg {
                            identifier: identifier,
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
                        TType::Function(paraminput, output) => {
                            self.environment.insert_symbol(
                                &id,
                                TType::Function(paraminput.clone(), Box::new(*output.clone())),
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
                let captured = self
                    .environment
                    .captured
                    .last()
                    .unwrap()
                    .iter()
                    .map(|v| v.0.clone())
                    .collect();
                self.environment.pop_scope();
                // check return types
                let (_, has_return) = self.check_returns(
                    &statements,
                    output.clone(),
                    pos.line,
                    pos.row,
                    &self.filepath,
                )?;
                if !has_return && output != TType::Void {
                    return Err(common::error::parser_error(
                        "Function is missing a return statement in a branch".to_string(),
                        "Function missing return".to_string(),
                        pos.line,
                        pos.row,
                        self.filepath.to_owned(),
                        None,
                    ));
                }

                if output == TType::Void {
                    if let Some(Statement::Return(_, _, _, _)) = statements.last() {
                    } else {
                        statements.push(Statement::Return(
                            output.clone(),
                            Expr::None,
                            self.current_token().line(),
                            self.current_token().row(),
                        ));
                    }
                }

                left = Expr::Closure(
                    TType::Function(typeinput, Box::new(output)),
                    input,
                    statements,
                    captured,
                )
            }
            Token::Symbol('[', _) => {
                let expr_list = self.expr_list()?;
                let mut ttype = TType::None;
                if !expr_list.is_empty() {
                    ttype = expr_list[0].get_type()
                }
                for elem in expr_list.clone() {
                    if elem.get_type() != ttype {
                        return Err(self.generate_error(
                            format!("List must contain same type"),
                            format!("Got type {:?}, expected type {:?}", elem.get_type(), ttype),
                        ));
                    }
                }
                match self.current_token() {
                    Token::Operator(Operator::Colon, _) => {
                        self.consume_operator(Operator::Colon)?;
                        ttype = self.ttype()?;}
                    _ => {}
                }
                if ttype == TType::None {
                    return Err(self.generate_error(
                        format!("List must have a type"),
                        format!("use `[]: type` to annotate an empty list"),
                    ));
                }
                left = Expr::ListConstructor(TType::List(Box::new(ttype)), expr_list)
            }
            Token::Symbol('(', _) => {
                self.consume_symbol('(')?;
                let expr = self.expr()?;
                self.consume_symbol(')')?;
                left = expr;
                if let Some(sign) = sign {
                    if Unary::Not == sign {
                        if left.get_type() != TType::Bool {
                            return Err(self.generate_error(
                                "cannot apply not operation to a non bool".to_string(),
                                "Make sure expression returns a bool type".to_string(),
                            ));
                        }
                    }
                    left = Expr::Unary(left.clone().get_type(), sign, Box::new(left));
                }
            }
            Token::Identifier(_, _) => {
                let (mut identifier, pos) = self.identifier()?;

                match self.current_token() {
                    Token::Symbol('@', _) => {
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
                    _ => {}
                }

                if self.current_token().is_symbol('@') {}

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
                    left = Expr::Unary(left.clone().get_type(), sign, Box::new(left));
                }
            }
            Token::Integer(v, _) => {
                self.advance();
                left = Expr::Literal(TType::Int, Atom::Integer(v));
                if let Some(sign) = sign {
                    if Unary::Not == sign {
                        if left.get_type() != TType::Bool {
                            return Err(self.generate_error(
                                "cannot apply not operation to a non bool".to_string(),
                                "Make sure expression returns a bool type".to_string(),
                            ));
                        }
                    }
                    left = Expr::Unary(left.clone().get_type(), sign, Box::new(left));
                }
            }
            Token::Float(v, _) => {
                self.advance();
                left = Expr::Literal(TType::Float, Atom::Float(v));
                if let Some(sign) = sign {
                    if Unary::Not == sign {
                        if left.get_type() != TType::Bool {
                            return Err(self.generate_error(
                                "cannot apply not operation to a non bool".to_string(),
                                "Make sure expression returns a bool type".to_string(),
                            ));
                        }
                    }
                    left = Expr::Unary(left.clone().get_type(), sign, Box::new(left));
                }
            }
            Token::String(v, _) => {
                self.advance();
                left = Expr::Literal(TType::String, Atom::String(v))
            }

            Token::Bool(v, _) => {
                self.advance();
                left = Expr::Literal(TType::Bool, Atom::Bool(v))
            }
            Token::EOF(_) => {
                return Err(common::error::parser_error(
                    format!("End of file error"),
                    format!("expected expression"),
                    self.current_token().line(),
                    self.current_token().row(),
                    self.filepath.clone(),
                    None,
                ));
            }
            _ => left = Expr::None,
        }
        loop {
            match self.current_token() {
                Token::Operator(Operator::DoubleColon, _) => {
                    self.consume_operator(Operator::DoubleColon)?;
                    let (field, pos) = self.identifier()?;
                    left = self.field(field.clone(), left, pos)?;
                }
                Token::Symbol('.', _) => {
                    self.consume_symbol('.')?;
                    left = self.chain(left)?;
                }
                Token::Symbol('(', _) => {
                    // function pointer return call <func()(args)>
                    let mut arguments = self.argument_list()?;
                    if arguments.is_empty() {
                        arguments.push(Expr::None)
                    }
                    if let TType::Function(argtypes, mut output) = left.get_type() {
                        if arguments.len() != argtypes.len() {
                            return Err(self.generate_error(
                                format!("E3 Inccorrect number of arguments"),
                                format!("Got {:?}, expected {:?}", arguments.len(), argtypes.len()),
                            ));
                        }
                        let mut inputtypes = vec![];
                        for t in arguments.iter() {
                            inputtypes.push(t.get_type())
                        }
                        let mut map: HashMap<String, TType> = HashMap::default();
                        map = self.check_and_map_types(&argtypes, &inputtypes, &mut map)?;
                        output = Box::new(self.get_output(*output.clone(), &mut map)?);
                        left = Expr::Call(*output, "anon".to_string(), Box::new(left), arguments);
                    } else {
                        return Err(self.generate_error(
                            format!("Cant call {:?}", left.get_type()),
                            format!("not a function"),
                        ));
                    }
                }
                Token::Symbol('[', _) => {
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
        let line = self.current_token().line();
        let row = self.current_token().row();
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
                    left = Expr::Binop(
                        left.clone().get_type(),
                        operation,
                        Box::new(left),
                        Box::new(right),
                    );
                } else {
                    return Err(common::error::parser_error(
                        format!(
                            "Type error, cannot apply operation {:?} to {:?} and {:?}",
                            operation.clone(),
                            left.clone(),
                            right.clone()
                        ),
                        format!("type mismatch"),
                        line,
                        row,
                        self.filepath.clone(),
                        None,
                    ));
                }
            }
        }
        Ok(left)
    }

    fn expr(&mut self) -> Result<Expr, NovaError> {
        let mut left = self.top_expr()?;
        while self.current_token().is_assign() {
            let oline = self.current_token().line();
            let orow = self.current_token().row();
            if let Some(operation) = self.current_token().get_operator() {
                self.advance();
                let right = self.top_expr()?;
                match left.clone() {
                    Expr::Literal(t, v) => match v {
                        Atom::Id(_) => {
                            if t != right.get_type() {
                                return Err(common::error::parser_error(
                                    format!(
                                        "Type error, cannot assing {:?} to {:?}",
                                        right.clone().get_type(),
                                        left.clone().get_type(),
                                    ),
                                    format!("Assingment error"),
                                    oline,
                                    orow,
                                    self.filepath.clone(),
                                    None,
                                ));
                            }
                        }
                        _ => {
                            return Err(common::error::parser_error(
                                format!("cannot assign {:?} to {:?}", right.clone(), left.clone(),),
                                format!("Cannot assign a value to a literal value"),
                                oline,
                                orow,
                                self.filepath.clone(),
                                None,
                            ));
                        }
                    },
                    _ => {
                        if &right.get_type() == &left.get_type() {
                        } else {
                            return Err(common::error::parser_error(
                                format!(
                                    "cannot assing {:?} to {:?}",
                                    right.clone().get_type(),
                                    left.clone().get_type(),
                                ),
                                format!("type mismatch"),
                                oline,
                                orow,
                                self.filepath.clone(),
                                None,
                            ));
                        }
                    }
                }
                left = Expr::Binop(TType::Void, operation, Box::new(left), Box::new(right));
            }
            
        }
        Ok(left)
    }

    fn top_expr(&mut self) -> Result<Expr, NovaError> {
        let mut left = self.mid_expr()?;
        while self.current_token().is_relop() {
            if let Some(operation) = self.current_token().get_operator() {
                self.advance();
                let right = self.mid_expr()?;
    
                match operation {
                    Operator::And | Operator::Or => {
                        if (left.get_type() != TType::Bool) || (right.get_type() != TType::Bool) {
                            return Err(self.generate_error(
                                format!("Logical operation expects bool"),
                                format!(
                                    "got {:?} {:?}",
                                    left.get_type().clone(),
                                    right.get_type().clone()
                                ),
                            ));
                        }
                        left = Expr::Binop(TType::Bool, operation, Box::new(left), Box::new(right));
                    }
                    Operator::GreaterThan
                    | Operator::GtrOrEqu
                    | Operator::LssOrEqu
                    | Operator::LessThan => {
                        match (left.get_type(), right.get_type()) {
                            (TType::Int, TType::Int) => {}
                            (TType::Float, TType::Float) => {}
                            _ => {
                                return Err(self.generate_error(
                                    format!("Comparison operation expects int or float"),
                                    format!(
                                        "got {:?} {:?}",
                                        left.get_type().clone(),
                                        right.get_type().clone()
                                    ),
                                ));
                            }
                        }
                        left = Expr::Binop(TType::Bool, operation, Box::new(left), Box::new(right));
                    }
                    _ => {
                        left = Expr::Binop(TType::Bool, operation, Box::new(left), Box::new(right));
                    }
                }
            }
           
        }
        Ok(left)
    }

    fn mid_expr(&mut self) -> Result<Expr, NovaError> {
        let mut left = self.term()?;
        while self.current_token().is_adding_op() {
            if let Some(operation) = self.current_token().get_operator() {
                let line = self.current_token().line();
                let row = self.current_token().row();
    
                self.advance();
                let right = self.term()?;
    
                match (left.get_type(), right.get_type()) {
                    (TType::Int, TType::Int)
                    | (TType::Float, TType::Float)
                    | (TType::String, TType::String) => {
                        left = Expr::Binop(
                            left.clone().get_type(),
                            operation,
                            Box::new(left),
                            Box::new(right),
                        );
                    }
                    (_, _) => {
                        return Err(common::error::parser_error(
                            format!(
                                "Type error, cannot apply operation {:?} to {:?} and {:?}",
                                operation.clone(),
                                left.get_type(),
                                right.get_type()
                            ),
                            format!("type mismatch"),
                            line,
                            row,
                            self.filepath.clone(),
                            None,
                        ));
                    }
                }
            }
            
        }
        Ok(left)
    }

    fn ttype(&mut self) -> Result<TType, NovaError> {
        match self.current_token() {
            Token::Symbol('(', _) => {
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
                    Ok(TType::Function(*Box::new(input), Box::new(output)))
                } else {
                    self.consume_symbol(')')?;
                    let mut output = TType::Void;
                    if self.current_token().is_op(Operator::RightArrow) {
                        self.consume_operator(Operator::RightArrow)?;
                        output = self.ttype()?;
                    }
                    Ok(TType::Function(
                        *Box::new(vec![TType::None]),
                        Box::new(output),
                    ))
                }
            }
            Token::Symbol('$', _) => {
                self.consume_symbol('$')?;
                let (generictype, _) = self.identifier()?;
                Ok(TType::Generic(generictype))
            }
            Token::Symbol('?', _) => {
                self.consume_symbol('?')?;
                let ttype = self.ttype()?;
                Ok(TType::Option(Box::new(ttype)))
            }
            Token::Symbol('[', _) => {
                self.consume_symbol('[')?;
                let mut inner = TType::None;
                if !self.current_token().is_symbol(']') {
                    inner = self.ttype()?;
                } 
                self.consume_symbol(']')?;
                Ok(TType::List(Box::new(inner)))
            }
            Token::Type(ttype, _) => {
                self.advance();
                Ok(ttype)
            }
            Token::Identifier(_, _) => {
                let (identifier, _) = self.identifier()?;
                if let Some(_) = self.environment.custom_types.get(&identifier) {
                    Ok(TType::Custom(identifier))
                } else {
                    return Err(self.generate_error(
                        "Expected type annotation".to_string(),
                        format!("Unknown type '{identifier}' "),
                    ));
                }
            }
            _ => {
                return Err(self.generate_error(
                    "Expected type annotation".to_string(),
                    format!("Unknown type value {:?}", self.current_token()),
                ));
            }
        }
    }

    fn identifier(&mut self) -> Result<(String, Position), NovaError> {
        let id = match self.current_token().expect_id() {
            Some(id) => id,
            None => {
                return Err(self.generate_error(
                    "Expected identifier".to_string(),
                    format!("Cannot assign a value to {:?}", self.current_token()),
                ));
            }
        };
        let (line, row) = self.get_line_and_row();
        self.advance();
        Ok((id, Position { line, row }))
    }

    fn parameter_list(&mut self) -> Result<Vec<(TType, String)>, NovaError> {
        let mut parameters: Table<String> = table::new();
        let mut args = vec![];
        if self.current_token().is_identifier() {
            let (id, _) = self.identifier()?;
            parameters.insert(id.clone());
            self.consume_operator(Operator::Colon)?;
            let ttype = self.ttype()?;
            args.push((ttype, id));
        }
        while self.current_token().is_symbol(',') {
            self.advance();
            self.eat_if_newline();
            match self.identifier() {
                Ok((id, _)) => {
                    if parameters.has(&id) {
                        return Err(self.generate_error(
                            format!("paremeter identifier already defined"),
                            format!("try using another name"),
                        ));
                    }
                    parameters.insert(id.clone());
                    self.consume_operator(Operator::Colon)?;
                    let ttype = self.ttype()?;
                    args.push((ttype, id));
                }
                Err(_) => {
                    break;
                }
            }
        }
        Ok(args)
    }

    fn alternative(&mut self) -> Result<Vec<Statement>, NovaError> {
        let test = self.top_expr()?;
        if test.get_type() != TType::Bool {
            return Err(self.generate_error(
                format!("If statement expression must return a bool"),
                format!("got {:?}", test.get_type().clone()),
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
        return Ok(vec![Statement::If(
            TType::Void,
            test,
            statements,
            alternative,
        )]);
    }

    fn import_file(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("using"))?;
        let ifilepath = match self.current_token() {
            Token::String(filepath, _) => filepath,
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
        Ok(Some(Statement::Block(
            iparser.ast.program.clone(),
            newfilepath,
        )))
    }

    fn statement(&mut self) -> Result<Option<Statement>, NovaError> {
        let (line, row) = self.get_line_and_row();

        match self.current_token() {
            Token::Identifier(id, _) => match id.as_str() {
                "using" => self.import_file(),
                "pass" => self.pass_statement(),
                "struct" => self.struct_declaration(),
                "if" => self.if_statement(),
                "while" => self.while_statement(),
                "let" => self.let_statement(),
                "return" => self.return_statement(line, row),
                "fn" => self.function_declaration(),
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
            Token::EOF(_) => Ok(None),
            _ => self.expression_statement(),
        }
    }

    fn pass_statement(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("pass"))?;
        Ok(Some(Statement::Pass))
    }

    fn struct_declaration(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("struct"))?;
        let (identifier, pos) = self.identifier()?;
        // will overwrite, just needed for recursive types.
        self.environment
            .custom_types
            .insert(identifier.clone(), vec![]);
        self.consume_symbol('{')?;
        self.eat_if_newline();
        let parameters = self.parameter_list()?;
        self.eat_if_newline();
        self.consume_symbol('}')?;

        let mut fields: Vec<(String, TType)> = vec![];
        let mut typeinput = vec![];
        for (ttype, identifier) in parameters.clone() {
            typeinput.push(ttype.clone());
            fields.push((identifier, ttype));
        }
        fields.push(("type".to_string(), TType::String));

        let mut input = vec![];
        for (identifier, ttype) in fields.clone() {
            input.push(Field { identifier, ttype })
        }

        if !self.environment.has(&identifier) {
            self.environment.no_override.insert(identifier.to_string());
            self.environment.insert_symbol(
                &identifier,
                TType::Function(typeinput, Box::new(TType::Custom(identifier.clone()))),
                Some(pos.clone()),
                SymbolKind::Constructor,
            );
            self.environment
                .custom_types
                .insert(identifier.clone(), fields);
        } else {
            return Err(common::error::parser_error(
                format!("Struct '{}' is already instantiated", identifier),
                "Cannot reinstantiate the same type".to_string(),
                pos.line,
                pos.row,
                self.filepath.clone(),
                None,
            ));
        }

        Ok(Some(Statement::Struct(
            TType::Custom(identifier.clone()),
            identifier,
            input,
        )))
    }

    fn for_statement(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("for"))?;
        let init = self.expr()?;
        self.consume_symbol(';')?;
        let test = self.expr()?;
        self.consume_symbol(';')?;
        let inc = self.expr()?;
        if test.get_type() != TType::Bool && test.get_type() != TType::Void {
            return Err(self.generate_error(
                format!("test expression must return a bool"),
                format!("got {:?}", test.get_type().clone()),
            ));
        }
        self.environment.push_block();
        let statements = self.block()?;
        self.environment.pop_scope();
        Ok(Some(Statement::For(init, test, inc, statements)))
    }

    fn while_statement(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("while"))?;
        let test = self.top_expr()?;
        if test.get_type() != TType::Bool && test.get_type() != TType::Void {
            return Err(self.generate_error(
                format!("test expression must return a bool"),
                format!("got {:?}", test.get_type().clone()),
            ));
        }
        self.environment.push_block();
        let statements = self.block()?;
        self.environment.pop_scope();

        Ok(Some(Statement::While(test, statements)))
    }

    fn if_statement(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("if"))?;
        let test = self.top_expr()?;
        if test.get_type() != TType::Bool {
            return Err(self.generate_error(
                format!("If statement's expression must return a bool"),
                format!("got {:?}", test.get_type().clone()),
            ));
        }
        //self.environment.push_block();
        let statements = self.block()?;
        //self.environment.pop_block();
        let mut alternative: Option<Vec<Statement>> = None;
        if self.current_token().is_id("elif") {
            self.advance();
            //self.environment.push_block();
            alternative = Some(self.alternative()?);
            //self.environment.pop_block();
        } else if self.current_token().is_id("else") {
            self.advance();
            //self.environment.push_block();
            alternative = Some(self.block()?);
            //self.environment.pop_block();
        }
        Ok(Some(Statement::If(
            TType::Void,
            test,
            statements,
            alternative,
        )))
    }

    fn let_statement(&mut self) -> Result<Option<Statement>, NovaError> {
        // let
        self.consume_identifier(Some("let"))?;
        let mut global = false;
        // refactor out into two parsing ways for ident. one with module and one without
        let (mut identifier, mut pos) = self.identifier()?;
        if identifier == "global" {
            (identifier, pos) = self.identifier()?;
            global = true
        }
        let mut ttype = TType::None;
        let mut expr = Expr::None;
        if self.current_token().is_op(Operator::Colon) {
            self.consume_operator(Operator::Colon)?;
            ttype = self.ttype()?;
            self.consume_operator(Operator::Assignment)?;
            expr = self.expr()?;
            self.check_and_map_types(

                &vec![expr.get_type()],
                &vec![ttype.clone()],


                &mut HashMap::default(),
            )?;
        } else {
            self.consume_operator(Operator::Assignment)?;
            expr = self.expr()?;
            ttype = expr.get_type();
        }

        // cant assing a void
        if expr.get_type() == TType::Void {
            return Err(common::error::parser_error(
                format!("Variable '{}' cannot be assinged to void", identifier),
                "Make sure the expression returns a value".to_string(),
                pos.line,
                pos.row,
                self.filepath.clone(),
                None,
            ));
        }
        // make sure symbol doesnt already exist
        if self.environment.has(&identifier) {
            return Err(common::error::parser_error(
                format!("Symbol '{}' is already instantiated", identifier),
                "Cannot reinstantiate the same symbol in the same scope".to_string(),
                pos.line,
                pos.row,
                self.filepath.clone(),
                None,
            ));
        } else {
            self.environment.insert_symbol(
                &identifier,
                ttype.clone(),
                Some(pos),
                SymbolKind::Variable,
            );
            Ok(Some(Statement::Let(ttype, identifier, expr, global)))
        }
    }

    fn return_statement(
        &mut self,
        line: usize,
        row: usize,
    ) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("return"))?;
        let expr = self.expr()?;
        Ok(Some(Statement::Return(expr.get_type(), expr, line, row)))
    }

    fn function_declaration(&mut self) -> Result<Option<Statement>, NovaError> {
        self.consume_identifier(Some("fn"))?;
        let (mut identifier, pos) = self.identifier()?;

        // check to see if its already defined
        if self.environment.has(&identifier) {
            return Err(self.generate_error(
                format!("Generic Function {identifier} already defined"),
                "Cannot overload a generic function".to_string(),
            ));
        }
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
        // is function using generics?
        fn is_generic(params: &[TType]) -> bool {
            for t in params {
                match t {
                    TType::Any => {
                        return true;
                    }
                    TType::Generic(_) => {
                        return true;
                    }
                    TType::Function(input, output) => {
                        if let TType::Generic(_) = **output {
                            return true;
                        }
                        if is_generic(&input.clone()) || is_generic(&vec![*output.clone()]) {
                            return true;
                        }
                    }
                    TType::List(list) => {
                        if let TType::Generic(_) = **list {
                            return true;
                        }
                        return is_generic(&vec![*list.clone()]);
                    }
                    TType::Option(option) => {
                        if let TType::Generic(_) = **option {
                            return true;
                        }
                        return is_generic(&vec![*option.clone()]);
                    }
                    _ => {}
                }
            }
            return false;
        }
        let generic = is_generic(&typeinput);
        // build helper vecs
        let mut input = vec![];
        for (ttype, identifier) in parameters.clone() {
            if let TType::Function(_, _) = ttype.clone() {
                // check if generic function exist
                if self.environment.has(&identifier) {
                    return Err(self.generate_error(
                        format!("Generic Function {} already defined", &identifier),
                        "Cannot redefine a generic function".to_string(),
                    ));
                }
                // check if normal function exist
                if self.environment.has(&identifier) {
                    return Err(self.generate_error(
                        format!("Function {} already defined", &identifier,),
                        "Cannot redefine a generic function".to_string(),
                    ));
                }
                // build argument list
                input.push(Arg {
                    identifier: identifier,
                    ttype: ttype.clone(),
                });
            } else {
                input.push(Arg {
                    identifier: identifier,
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
            return Err(self.generate_error(
                format!(
                    "Function {identifier} with inputs {:?} is already defined",
                    typeinput
                ),
                "Cannot redefine a function with the same signature".to_string(),
            ));
        }

        // insert function into environment
        if !generic {
            self.environment.insert_symbol(
                &identifier,
                TType::Function(typeinput.clone(), Box::new(output.clone())),
                Some(pos.clone()),
                SymbolKind::Function,
            );
            identifier = generate_unique_string(&identifier, &typeinput);
        } else {
            if self.environment.no_override.has(&identifier) {
                return Err(self.generate_error(
                    format!(
                        "Cannot create generic functon since, {} is already defined",
                        &identifier
                    ),
                    "Cannot create generic function since this function is overload-able"
                        .to_string(),
                ));
            }
            self.environment.insert_symbol(
                &identifier,
                TType::Function(typeinput.clone(), Box::new(output.clone())),
                Some(pos.clone()),
                SymbolKind::GenericFunction,
            );
        }
        self.environment.no_override.insert(identifier.clone());
        // parse body with scope
        self.environment.push_scope();
        // insert params into scope
        for (ttype, id) in parameters.iter() {
            match ttype.clone() {
                TType::Function(paraminput, output) => {
                    self.environment.insert_symbol(
                        &id,
                        TType::Function(paraminput.clone(), Box::new(*output.clone())),
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
        self.environment.pop_scope();
        // check return types
        let (_, has_return) = self.check_returns(
            &statements,
            output.clone(),
            pos.line,
            pos.row,
            &self.filepath,
        )?;
        if !has_return && output != TType::Void {
            return Err(common::error::parser_error(
                "Function is missing a return statement in a branch".to_string(),
                "Function missing return".to_string(),
                pos.line,
                pos.row,
                self.filepath.to_owned(),
                None,
            ));
        }

        // if output void, insert return as last statement if one wasnt added
        if output == TType::Void {
            if let Some(Statement::Return(_, _, _, _)) = statements.last() {
            } else {
                statements.push(Statement::Return(
                    output.clone(),
                    Expr::None,
                    self.current_token().line(),
                    self.current_token().row(),
                ));
            }
        }

        // if last statement isnt a return error
        if let Some(Statement::Return(_, _, _, _)) = statements.last() {
        } else {
            return Err(common::error::parser_error(
                "Function is missing a return statement in a branch".to_string(),
                "Function missing return".to_string(),
                pos.line,
                pos.row,
                self.filepath.to_owned(),
                None,
            ));
        }

        Ok(Some(Statement::Function(
            output, identifier, input, statements,
        )))
    }

    fn check_returns(
        &self,
        statements: &[Statement],
        return_type: TType,
        line: usize,
        row: usize,
        filepath: &str,
    ) -> Result<(TType, bool), NovaError> {
        let mut has_return = false;
        for statement in statements {
            match statement {
                Statement::Pass => has_return = true,
                Statement::Return(ttype, _, _, _) => {
                    self.check_and_map_types(
                        &vec![ttype.clone()],
                        &vec![return_type.clone()],
                        &mut HashMap::default(),
                    )?;
                    has_return = true
                }
                Statement::If(_, _, if_body, else_body) => {
                    let (bodytype, bhr) =
                        self.check_returns(if_body, return_type.clone(), line, row, filepath)?;
                    if let Some(alternative) = else_body {
                        let (elsetype, ehr) = self.check_returns(
                            &alternative,
                            return_type.clone(),
                            line,
                            row,
                            filepath,
                        )?;
                        if bodytype != elsetype {
                            return Err(common::error::parser_error(
                                "Function is missing a return statement in a branch".to_string(),
                                "All branches of if-else must have a return statement".to_string(),
                                line,
                                row,
                                filepath.to_owned(),
                                None,
                            ));
                        }
                        if bhr && ehr {
                            has_return = true
                        }
                    } else {
                        if bhr {
                            has_return = true
                        }
                    }
                }
                _ => {}
            }
        }

        Ok((return_type.clone(), has_return))
    }

    fn expression_statement(&mut self) -> Result<Option<Statement>, NovaError> {
        if !self.current_token().is_newline() {
            let expr = self.expr()?;
            if expr.get_type() != TType::Void {
                return Err(self.generate_error(
                    "Expression returns value, but does nothing with it".to_string(),
                    "Remove the expression or assign it to a variable".to_string(),
                ));
            }
            match expr {
                Expr::None => Ok(None),
                _ => Ok(Some(Statement::Expression(expr.get_type(), expr))),
            }
        } else {
            self.advance();
            Ok(None)
        }
    }

    fn block(&mut self) -> Result<Vec<Statement>, NovaError> {
        self.consume_symbol('{')?;
        self.eat_if_newline();
        let statements = self.compound_statement()?;
        self.consume_symbol('}')?;
        Ok(statements)
    }

    fn compound_statement(&mut self) -> Result<Vec<Statement>, NovaError> {
        let mut statements = vec![];
        self.eat_if_newline();
        if let Some(statement) = self.statement()? {
            statements.push(statement);
        }
        while self.current_token().is_newline() || self.current_token().is_symbol(';') {
            self.advance();
            if self.current_token().is_symbol('}') {
                break;
            }
            if self.current_token().is_newline() {
                continue;
            }
            if let Some(statement) = self.statement()? {
                statements.push(statement);
            }
        }
        Ok(statements)
    }

    pub fn parse(&mut self) -> Result<(), NovaError> {
        self.ast.program = self.compound_statement()?;
        self.eof()
    }
}
