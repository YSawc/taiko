#[derive(Debug, Clone, PartialEq)]
pub struct SourceInfo {
    pub code: Vec<char>,
    pub coordinates: Vec<(usize, usize, usize)>,
}

impl SourceInfo {
    pub fn new(code_text: impl Into<String>) -> SourceInfo {
        SourceInfo {
            code: code_text.into().chars().collect::<Vec<char>>(),
            coordinates: vec![],
        }
    }
}
