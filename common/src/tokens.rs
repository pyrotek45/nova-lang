pub type TokenList = Vec<Token>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub row: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unary {
    Positive,
    Negitive,
    Not,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operator {
    Concat,
    AdditionAssignment,
    SubtractionAssignment,
    And,
    Or,
    Colon,
    GtrOrEqu,
    LssOrEqu,
    DoubleColon,
    RightArrow,
    GreaterThan,
    LessThan,
    Assignment,
    Addition,
    Subtraction,
    Division,
    Multiplication,
    Equality,
    NotEqual,
    Access,
    ListAccess,
    Call,
    Modulo,
    Not,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TType {
    EmptyList,
    None,
    Any,
    Int,
    Float,
    Bool,
    Str,
    Void,
    Auto,
    Custom(String),
    List(Box<TType>),
    Function(Vec<TType>, Box<TType>),
    Generic(String),
    Option(Box<TType>),
}

pub fn generate_unique_string(input: &str, types: &[TType]) -> String {
    if types.is_empty() {
        return input.to_owned();
    }
    let type_strings: Vec<String> = types.iter().map(|t| type_to_string(t)).collect();
    let types_concatenated = type_strings.join("_");
    format!("{}_{}", input, types_concatenated)
}

// pub fn generate_module_string(input: &str, modules: &[String]) -> String {
//     if modules.is_empty() {
//         return input.to_owned();
//     }
//     let modules_concatenated = modules.join("::");
//     format!("{}::{}", modules_concatenated, input)
// }

pub fn type_to_string(ttype: &TType) -> String {
    match ttype {
        TType::Any => "any".to_string(),
        TType::Int => "int".to_string(),
        TType::Float => "float".to_string(),
        TType::Bool => "bool".to_string(),
        TType::Str => "str".to_string(),
        TType::Void => "void".to_string(),
        TType::Auto => "auto".to_string(),
        TType::Custom(name) => name.clone(),
        TType::List(inner) => format!("list_{}", type_to_string(inner)),
        TType::Function(args, ret) => {
            let args_str = args
                .iter()
                .map(|t| type_to_string(t))
                .collect::<Vec<String>>()
                .join("_");
            format!("function_{}_{}", args_str, type_to_string(ret))
        }
        TType::Generic(name) => format!("generic_{}", name),
        TType::None => "none".to_string(),
        TType::Option(name) => format!("option_{}", type_to_string(name)),
        TType::EmptyList => format!("empty_list"),
    }
}

impl TType {
    pub fn get_inner(&self) -> Option<TType> {
        match self {
            TType::List(inner) => Some(*inner.clone()),
            _ => None,
        }
    }

    pub fn is_function(&self) -> bool {
        match self {
            TType::Function(_, _) => true,
            _ => false,
        }
    }

    pub fn custom_to_string(&self) -> Option<String> {
        match self {
            TType::Custom(v) => Some(v.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Type(TType, Position),
    Identifier(String, Position),
    Integer(i64, Position),
    Float(f64, Position),
    String(String, Position),
    Char(char, Position),
    Symbol(char, Position),
    Bool(bool, Position),
    Operator(Operator, Position),
    NewLine(Position),
    EOF(Position),
}

impl Token {
    pub fn get_bool(self) -> Option<bool> {
        if let Token::Bool(v, _) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn get_str(self) -> Option<String> {
        if let Token::String(v, _) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn get_id(self) -> Option<String> {
        if let Token::Identifier(v, _) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn get_int(self) -> Option<i64> {
        if let Token::Integer(v, _) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn get_float(self) -> Option<f64> {
        if let Token::Float(v, _) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn expect_id(self) -> Option<String> {
        if let Token::Identifier(id, _) = self {
            Some(id)
        } else {
            None
        }
    }

    pub fn is_identifier(&self) -> bool {
        match self {
            Token::Identifier(_, _) => true,
            _ => false,
        }
    }

    pub fn is_id(&self, ident: &str) -> bool {
        if let Token::Identifier(id, _) = self {
            &ident == &id
        } else {
            false
        }
    }

    pub fn line(&self) -> usize {
        match self {
            Token::Type(_, pos)
            | Token::Identifier(_, pos)
            | Token::Integer(_, pos)
            | Token::Float(_, pos)
            | Token::String(_, pos)
            | Token::Char(_, pos)
            | Token::Symbol(_, pos)
            | Token::Bool(_, pos)
            | Token::Operator(_, pos)
            | Token::EOF(pos) => pos.line,
            Token::NewLine(pos) => pos.line,
        }
    }

    pub fn row(&self) -> usize {
        match self {
            Token::Type(_, pos)
            | Token::Identifier(_, pos)
            | Token::Integer(_, pos)
            | Token::Float(_, pos)
            | Token::String(_, pos)
            | Token::Char(_, pos)
            | Token::Symbol(_, pos)
            | Token::Bool(_, pos)
            | Token::Operator(_, pos)
            | Token::EOF(pos) => pos.row,
            Token::NewLine(pos) => pos.row,
        }
    }

    pub fn is_symbol(&self, c: char) -> bool {
        if let Token::Symbol(s, _) = self {
            *s == c
        } else {
            false
        }
    }
    pub fn is_newline(&self) -> bool {
        if let Token::NewLine(_) = self {
            true
        } else {
            false
        }
    }
    pub fn is_relop(&self) -> bool {
        if let Token::Operator(op, _) = self {
            match op {
                Operator::Equality
                | Operator::And
                | Operator::Or
                | Operator::GtrOrEqu
                | Operator::LssOrEqu
                | Operator::GreaterThan
                | Operator::LessThan
                | Operator::NotEqual => true,
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn is_op(&self, op: Operator) -> bool {
        if let Token::Operator(o, _) = self {
            *o == op
        } else {
            false
        }
    }

    pub fn is_adding_op(&self) -> bool {
        if let Token::Operator(op, _) = self {
            *op == Operator::Addition || *op == Operator::Subtraction || *op == Operator::Concat
        } else {
            false
        }
    }

    pub fn is_multi_op(&self) -> bool {
        if let Token::Operator(op, _) = self {
            *op == Operator::Multiplication || *op == Operator::Division || *op == Operator::Modulo
        } else {
            false
        }
    }

    pub fn is_assign(&self) -> bool {
        if let Token::Operator(op, _) = self {
            *op == Operator::Assignment
                || *op == Operator::AdditionAssignment
                || *op == Operator::SubtractionAssignment
        } else {
            false
        }
    }

    pub fn is_eof(&self) -> bool {
        if let Token::EOF(_) = self {
            true
        } else {
            false
        }
    }

    pub fn get_operator(&self) -> Operator {
        if let Token::Operator(op, _) = self {
            op.clone()
        } else {
            todo!()
        }
    }
}
