use crate::class::class::*;
use crate::instance::instance::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    FixNum(i64),
    FixDecimalNum(f64),
    String(String),
    Class(ClassRef),
    Instance(InstanceRef),
    SelfClass(Class),
    Array(Vec<Value>),
}
