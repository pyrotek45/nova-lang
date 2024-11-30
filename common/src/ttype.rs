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

    pub fn custom_to_string(&self) -> Option<String> {
        match self {
            TType::Custom { name, .. } => Some(name.to_string()),
            _ => None,
        }
    }

    // Adjusted `to_string` to handle the new field names
    pub fn to_string(&self) -> String {
        match self {
            TType::Any => "Any".to_string(),
            TType::Int => "Int".to_string(),
            TType::Float => "Float".to_string(),
            TType::Bool => "Bool".to_string(),
            TType::String => "String".to_string(),
            TType::Void => "Void".to_string(),
            TType::Auto => "Auto".to_string(),
            TType::Char => "Char".to_string(),
            TType::None => "None".to_string(),
            TType::Custom {
                name, type_params, ..
            } => {
                let type_strings: Vec<String> = type_params.iter().map(TType::to_string).collect();
                let types_concatenated = type_strings.join(",");
                if types_concatenated.is_empty() {
                    name.to_string()
                } else {
                    format!("{}({})", name, types_concatenated)
                }
            }
            TType::Generic { name } => format!("${}", name),
            TType::List { inner } => format!("[{}]", inner.to_string()),
            TType::Option { inner } => format!("?{}", inner.to_string()),
            TType::Tuple { elements } => {
                let types = elements
                    .iter()
                    .map(TType::to_string)
                    .collect::<Vec<String>>()
                    .join(",");
                format!("#({})", types)
            }
            TType::Function {
                parameters: args,
                return_type,
            } => {
                let args_str = args
                    .iter()
                    .map(TType::to_string)
                    .collect::<Vec<String>>()
                    .join(",");
                format!("({}) -> {}", args_str, return_type.to_string())
            }
        }
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
