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
        if let Some(line) = self.coordinates.iter().find(|x| x.2 == loc.2) {
            println!(
                "{}",
                self.code[(line.0)..(line.1)].iter().collect::<String>()
            );
            println!("{}{}", " ".repeat(loc.0), "^".repeat(loc.1 - loc.0 + 1));
        } else {
            panic!("no location found!");
        };
    }
}
