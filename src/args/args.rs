// use crate::node::node::*;
// use crate::util::util::*;
use crate::value::value::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Args {
    pub body: u8,
    pub args: Vec<Value>,
    pub table: u8,
}

impl Args {
    pub fn new() -> Self {
        Self {
            body: 0,
            args: vec![],
            table: 0,
        }
    }
}
