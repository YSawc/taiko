use crate::util::annot::*;
use rustc_hash::FxHashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct SourceInfo {
    pub code: Vec<char>,
    pub coordinates: Vec<(usize, usize, usize)>,
}

impl SourceInfo {
    pub fn new() -> SourceInfo {
        SourceInfo {
            code: vec![],
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdentId(usize);

impl std::ops::Deref for IdentId {
    type Target = usize;
    fn deref(&self) -> &usize {
        &self.0
    }
}

impl std::hash::Hash for IdentId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentifierTable {
    table: FxHashMap<String, usize>,
    table_rev: FxHashMap<usize, String>,
    ident_id: usize,
}

impl Default for IdentifierTable {
    fn default() -> Self {
        Self::new()
    }
}

impl IdentifierTable {
    pub fn new() -> Self {
        IdentifierTable {
            table: FxHashMap::default(),
            table_rev: FxHashMap::default(),
            ident_id: 0,
        }
    }

    pub fn get_ident_id(&mut self, name: &str) -> IdentId {
        match self.table.get(name) {
            Some(id) => IdentId(*id),
            None => {
                let id = self.ident_id;
                self.table.insert(name.to_string(), id);
                self.table_rev.insert(id, name.to_string());
                self.ident_id += 1;
                IdentId(id)
            }
        }
    }

    pub fn get_name(&mut self, id: IdentId) -> &String {
        self.table_rev.get(&id).unwrap()
    }
}
