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
    Custom(String),
    List(Box<TType>),
    Function(Vec<TType>, Box<TType>),
    Generic(String),
    Option(Box<TType>),
    Tuple(Vec<TType>),
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
        TType::Any => "Any".to_string(),
        TType::Int => "Int".to_string(),
        TType::Float => "Float".to_string(),
        TType::Bool => "Bool".to_string(),
        TType::String => "String".to_string(),
        TType::Void => "Void".to_string(),
        TType::Auto => "Auto".to_string(),
        TType::Custom(name) => name.clone(),
        TType::List(inner) => format!("List_{}", type_to_string(inner)),
        TType::Function(args, ret) => {
            let args_str = args
                .iter()
                .map(|t| type_to_string(t))
                .collect::<Vec<String>>()
                .join("_");
            format!("Function_{}_{}", args_str, type_to_string(ret))
        }
        TType::Generic(name) => format!("Generic_{}", name),
        TType::None => "None".to_string(),
        TType::Option(name) => format!("Option_{}", type_to_string(name)),
        TType::Char => "Char".to_string(),
        TType::Tuple(tuple) => {
            let types = tuple
                .iter()
                .map(|t| type_to_string(t))
                .collect::<Vec<String>>()
                .join("_");
            format!("Function_{}", types)
        }
    }
}

impl TType {
    pub fn get_inner(&self) -> Option<TType> {
        match self {
            TType::List(inner) => Some(*inner.clone()),
            TType::String => Some(TType::String),
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
            TType::Function(_, out) => out.custom_to_string(),
            TType::Custom(v) => Some(v.clone()),
            _ => None,
        }
    }
}
