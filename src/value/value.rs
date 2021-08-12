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
            Value::Nil => 0,
            _ => unimplemented!("Non value detected."),
        }
    }

    pub fn ident(&mut self) -> IdentId {
        match self {
            Value::FixNum(num) => IdentId((*num) as usize),
            _ => unimplemented!("Non value detected."),
        }
    }

    pub fn usize(&mut self) -> usize {
        match self {
            Value::FixNum(num) => *num as usize,
            Value::Nil => 0,
            _ => unimplemented!("Non value detected."),
        }
    }

    pub fn bool(&mut self) -> bool {
        match self {
            Value::Bool(b) => *b,
            _ => unimplemented!("Non bool detected."),
        }
    }
}
