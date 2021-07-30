use crate::class::class::*;
use crate::instance::instance::*;
use crate::util::util::*;

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

impl Value {
    pub fn value(&mut self) -> i64 {
        match self {
            Value::FixNum(num) => *num,
            _ => unimplemented!("Non value detected."),
        }
    }

    pub fn ident(&mut self) -> IdentId {
        match self {
            Value::FixNum(num) => IdentId((*num) as usize),
            _ => unimplemented!("Non value detected."),
        }
    }
}
