use std::fmt::Display;

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
        name: String,
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
        name: String,
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
            TType::Option { inner } => return write!(f, "?{inner}"),
            TType::Tuple { elements } => {
                return write!(f, "#({})", TypeList(elements));
            }
            TType::Function {
                parameters: args,
                return_type,
            } => return write!(f, "({params}) -> {return_type}", params = TypeList(args)),
        };
        f.write_str(literal)
    }
}

// Generate a unique string representation
pub fn generate_unique_string(input: &str, types: &[TType]) -> String {
    if types.is_empty() {
        return input.to_owned();
    }
    let type_strings: Vec<String> = types.iter().map(TType::to_string).collect();
    let types_concatenated = type_strings.join("_");
    format!("{}_{}", input, types_concatenated)
}
