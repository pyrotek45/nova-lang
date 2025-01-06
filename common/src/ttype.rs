use std::{
    fmt::{Display, Write},
    rc::Rc,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TType {
    None,
    Any,
    Int,
    Float,
    Bool,
    String,
    Char,
    Void,
    Auto,
    Custom {
        name: Rc<str>,
        type_params: Vec<TType>,
    },
    List {
        inner: Box<TType>,
    },
    Function {
        parameters: Vec<TType>,
        return_type: Box<TType>,
    },
    Generic {
        name: Rc<str>,
    },
    Option {
        inner: Box<TType>,
    },
    Tuple {
        elements: Vec<TType>,
    },
}

impl TType {
    pub fn get_inner(&self) -> Option<&TType> {
        match self {
            TType::List { inner } | TType::Option { inner } => Some(inner),
            TType::String => Some(&TType::String),
            _ => None,
        }
    }

    pub fn is_function(&self) -> bool {
        matches!(self, TType::Function { .. })
    }

    pub fn custom_to_string(&self) -> Option<&str> {
        match self {
            TType::Custom { name, .. } => Some(name),
            _ => None,
        }
    }
}

struct TypeList<'s>(&'s [TType]);

impl Display for TypeList<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut args = self.0.iter();
        if let Some(first) = args.next() {
            write!(f, "{first}")?
        }
        for arg in args {
            write!(f, ",{arg}")?
        }
        Ok(())
    }
}

impl Display for TType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let literal = match self {
            TType::Any => "Any",
            TType::Int => "Int",
            TType::Float => "Float",
            TType::Bool => "Bool",
            TType::String => "String",
            TType::Void => "Void",
            TType::Auto => "Auto",
            TType::Char => "Char",
            TType::None => "None",
            TType::Custom {
                name, type_params, ..
            } => {
                if !type_params.is_empty() {
                    return write!(f, "{name}({})", TypeList(type_params));
                }
                name
            }
            TType::Generic { name } => return write!(f, "${name}"),
            TType::List { inner } => return write!(f, "[{inner}]"),
            TType::Option { inner } => return write!(f, "Option({inner})"),
            TType::Tuple { elements } => {
                return write!(f, "({})", TypeList(elements));
            }
            TType::Function {
                parameters: args,
                return_type,
            } => return write!(f, "fn({params}) -> {return_type}", params = TypeList(args)),
        };
        f.write_str(literal)
    }
}

// Generate a unique string representation
pub fn generate_unique_string(input: &str, types: &[TType]) -> String {
    if types.is_empty() {
        return input.to_owned();
    }
    let mut out = format!("{input}_");
    let mut types = types.iter();
    if let Some(t) = types.next() {
        write!(out, "{t}").unwrap();
    }
    types.for_each(|t| write!(out, "_{t}").unwrap());
    out
}
