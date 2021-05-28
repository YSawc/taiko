use crate::class::class::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    FixNum(i64),
    String(String),
    Class(ClassRef),
}
