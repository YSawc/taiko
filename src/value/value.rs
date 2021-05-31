use crate::class::class::*;
use crate::instance::instance::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    FixNum(i64),
    String(String),
    Class(ClassRef),
    Instance(InstanceRef),
}
