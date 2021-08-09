use crate::util::util::*;
use crate::value::value::*;

use rustc_hash::FxHashMap;

#[derive(Debug, Clone)]
pub struct Stack {
    pub iseqs: FxHashMap<usize, Vec<u8>>,
    pub ident_table: IdentifierTable,
    pub stack_poses: Vec<usize>,
    pub iseq_poses: Vec<usize>,
    pub eval_stacks: Vec<Vec<Value>>,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            iseqs: FxHashMap::default(),
            ident_table: IdentifierTable::default(),
            stack_poses: vec![],
            iseq_poses: vec![],
            eval_stacks: vec![],
        }
    }
}
