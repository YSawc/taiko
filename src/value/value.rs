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
    Range(Box<Value>, Box<Value>),
}

impl Value {
    pub fn value(&mut self) -> i64 {
        match self {
            Value::FixNum(num) => *num,
            Value::Nil => 0,
            _ => unimplemented!("Non value detected."),
        }
    }

    pub fn option_ident(&mut self) -> Option<IdentId> {
        match self {
            Value::FixNum(num) => Some(IdentId((*num) as usize)),
            Value::Nil => None,
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

    pub fn range(&mut self) -> (i64, i64) {
        match self {
            Value::Range(e, s) => match (*e.to_owned(), *s.to_owned()) {
                (Value::FixNum(e), Value::FixNum(s)) => (s, e),
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }

    pub fn to_b(self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(b) => b,
            Value::FixNum(n) => {
                if n > 0 {
                    true
                } else {
                    false
                }
            }
            Value::String(_) => true,
            _ => unimplemented!(),
        }
    }

    pub fn to_i(self) -> i64 {
        match self {
            Value::String(s) => match s.parse() {
                Ok(i) => i,
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }

    pub fn to_class(self) -> Class {
        match self {
            Value::Nil => Class::Nil,
            Value::Bool(_) => Class::Bool,
            Value::FixNum(_) => Class::FixNum,
            Value::FixDecimalNum(_) => Class::FixDecimalNum,
            Value::String(_) => Class::String,
            Value::Class(_) => Class::Class,
            Value::Instance(_) => Class::Instance,
            _ => unimplemented!(),
        }
    }
}
