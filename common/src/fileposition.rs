use std::{path::Path, rc::Rc};

use crate::error::NovaError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePosition {
    pub filepath: Option<Rc<Path>>,
    pub line: usize,
    pub col: usize,
}

impl Default for FilePosition {
    fn default() -> Self {
        FilePosition {
            line: 1,
            col: 1,
            filepath: Default::default(),
        }
    }
}

pub fn load_file_content(path: &Path) -> Result<String, NovaError> {
    let source = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => {
            return Err(NovaError::File {
                msg: format!(" '{}' is not a valid filepath", path.display()).into(),
            })
        }
    };
    Ok(source)
}
