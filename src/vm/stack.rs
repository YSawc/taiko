use crate::util::util::*;

use rustc_hash::FxHashMap;

#[derive(Debug, Clone)]
pub struct Stack {
    pub iseqs: FxHashMap<usize, Vec<u8>>,
    pub ident_table: IdentifierTable,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            iseqs: FxHashMap::default(),
            ident_table: IdentifierTable::default(),
        }
    }
}
