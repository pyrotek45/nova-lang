use crate::error::NovaError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePosition {
    pub filepath: String,
    pub line: usize,
    pub row: usize,
}

pub fn load_file_content(filepath: &str) -> Result<String, NovaError> {
    let source = match std::fs::read_to_string(filepath) {
        Ok(content) => content,
        Err(_) => {
            return Err(NovaError::File {
                msg: format!(" '{filepath}' is not a valid filepath"),
            })
        }
    };
    Ok(source)
}
