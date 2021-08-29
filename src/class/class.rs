// use crate::node::node::*;
use crate::util::util::*;
use crate::value::value::*;
use crate::vm::vm::*;
use rustc_hash::FxHashMap;

#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub id: IdentId,
    pub name: String,
    pub ptr: Box<usize>,
    pub ident_table: IdentifierTable,
    pub instance_var: FxHashMap<IdentId, Value>,
    pub instance_method_table: MethodTable,
    pub method_table: MethodTable,
    pub class_var: FxHashMap<IdentId, Value>,
    pub class_method_table: MethodTable,
    pub subclass: FxHashMap<IdentId, ClassRef>,
}

impl ClassInfo {
    pub fn new(id: IdentId, name: String, ptr: usize) -> Self {
        Self {
            id,
            name,
            ptr: Box::new(ptr),
            ident_table: IdentifierTable::default(),
            instance_var: FxHashMap::default(),
            instance_method_table: FxHashMap::default(),
            method_table: FxHashMap::default(),
            class_var: FxHashMap::default(),
            class_method_table: FxHashMap::default(),
            subclass: FxHashMap::default(),
        }
    }

    pub fn add_class_method(&mut self, id: IdentId, info: MethodInfo) -> Option<MethodInfo> {
        self.class_method_table.insert(id, info)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassRef(pub usize);

impl ClassRef {
    pub fn to_val(&mut self) -> Value {
        Value::Class(*self)
    }
}

impl std::hash::Hash for ClassRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

#[derive(Debug, Clone)]
pub struct GlobalClassTable {
    pub table: FxHashMap<ClassRef, ClassInfo>,
    class_id: usize,
}

impl Default for GlobalClassTable {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalClassTable {
    pub fn new() -> Self {
        Self {
            table: FxHashMap::default(),
            class_id: 0,
        }
    }

    pub fn new_class(&mut self, id: IdentId, name: String, ptr: usize) -> ClassRef {
        let info = ClassInfo::new(id, name, ptr);
        let new_class = ClassRef(self.class_id);
        self.class_id += 1;
        self.table.insert(new_class, info);
        new_class
    }

    pub fn get(&mut self, class_ref: ClassRef) -> &ClassInfo {
        self.table
            .get(&class_ref)
            .unwrap_or_else(|| panic!("GlobalClassTable#get(): ClassRef is not valid."))
    }

    pub fn get_mut(&mut self, class_ref: ClassRef) -> &mut ClassInfo {
        self.table
            .get_mut(&class_ref)
            .unwrap_or_else(|| panic!("ClobalClassTable#get_mut(): {:?} is not valid.", class_ref))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Class {
    Nil,
    Bool,
    FixNum,
    FixDecimalNum,
    String,
    Class,
    Instance,
}
