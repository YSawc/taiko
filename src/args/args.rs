use crate::node::node::*;
use crate::value::value::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Args {
    pub node: Node,
    pub value: Vec<Value>,
    pub table: Node,
}

impl Args {
    pub fn new() -> Self {
        Self {
            node: Node::new_none(),
            value: vec![],
            table: Node::new_none(),
        }
    }
}
