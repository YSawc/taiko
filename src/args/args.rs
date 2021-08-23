// use crate::node::node::*;
// use crate::util::util::*;
use crate::value::value::*;
use crate::vm::vm::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Args {
    pub body: Vec<ISeq>,
    pub args: Vec<Value>,
    pub table: u8,
}

impl Args {
    pub fn new() -> Self {
        Self {
            body: vec![],
            args: vec![],
            table: 0,
        }
    }
}
