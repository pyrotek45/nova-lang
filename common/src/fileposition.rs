use std::{fmt, path::Path, rc::Rc};

use crate::error::{NovaError, NovaResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePosition {
    pub filepath: Option<Rc<Path>>,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for FilePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file = self
            .filepath
            .as_deref()
            .unwrap_or(Path::new("repl"));
        write!(f, "{}:{}:{}", file.display(), self.line, self.col)
    }
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

pub fn load_file_content(path: &Path) -> NovaResult<String> {
    let source = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => {
            return Err(Box::new(NovaError::File {
                msg: format!(" '{}' is not a valid filepath", path.display()).into(),
            }))
        }
    };
    Ok(source)
}
