use crate::util::annot::*;

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

    pub fn show_loc(&self, loc: &Loc) {
        for line in &self.coordinates {
            if line.1 < loc.0 || line.0 > loc.1 {
                continue;
            }
            println!(
                "{}",
                self.code[(line.0)..(line.1)].iter().collect::<String>()
            );
            use std::cmp::*;
            let read = if loc.0 < line.0 { 0 } else { loc.0 - line.0 };
            let length = min(loc.1, line.1) + 1 - max(loc.0, line.0);
            println!("{}{}", " ".repeat(read), "^".repeat(length));
        }
    }
}
