#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePosition {
    pub filepath: String,
    pub line: usize,
    pub row: usize,
}
