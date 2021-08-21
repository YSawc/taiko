use crate::args::args::*;
use crate::class::class::*;

use crate::instance::instance::*;
use crate::node::node::*;
use crate::util::annot::*;
use crate::util::util::*;
use crate::value::value::*;
use crate::vm::inst::*;
use crate::vm::stack::*;
use rustc_hash::FxHashMap;

use std::ops::Deref;

pub type ISeq = u8;

#[derive(Debug, Clone)]
pub struct VM {
    pub stack: Stack,
    pub source_info: SourceInfo,
    pub ident_table: IdentifierTable,
    pub class_table: GlobalClassTable,
    pub instance_table: GlobalInstanceTable,
    pub method_table: MethodTable,
    pub const_table: ValueTable,
    pub class_stack: Vec<ClassRef>,
    pub scope_stack: Vec<LocalScope>,
    pub global_stack: Vec<GlobalScope>,
    pub env: Vec<Env>,
}

pub type ValueTable = FxHashMap<IdentId, Value>;
pub type BuiltinFunc = fn(eval: &mut VM, receiver: Value, args: Args) -> Value;

#[derive(Debug, Clone)]
pub enum Env {
    ClassRef(ClassRef),
    InstanceRef(InstanceRef),
}

#[derive(Clone)]
pub enum MethodInfo {
    RubyFunc {
        params: Vec<IdentId>,
        ptr: usize,
        local_scope: LocalScope,
    },
    BuiltinFunc {
        name: String,
        func: BuiltinFunc,
    },
}

impl std::fmt::Debug for MethodInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MethodInfo::RubyFunc {
                params,
                ptr,
                local_scope,
            } => {
                write!(f, "RubyFunc {:?} {:?} {:?}", params, ptr, local_scope)
            }
            MethodInfo::BuiltinFunc { name, .. } => write!(f, "BuiltinFunc {:?}", name),
        }
    }
}

pub type MethodTable = FxHashMap<IdentId, MethodInfo>;

#[derive(Debug, Clone, PartialEq)]
pub struct LocalScope {
    lvar_table: ValueTable,
    propagated_table: ValueTable,
}

impl LocalScope {
    pub fn new() -> Self {
        Self {
            lvar_table: FxHashMap::default(),
            propagated_table: FxHashMap::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalScope {
    gvar_table: ValueTable,
}

impl GlobalScope {
    pub fn new() -> Self {
        Self {
            gvar_table: FxHashMap::default(),
        }
    }
}

pub type EvalResult = Result<Value, RuntimeError>;

pub type RuntimeError = Annot<RuntimeErrorKind>;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeErrorKind {
    Unimplemented(String),
    Unreachable(String),
    Name(String),
    NoMethod(String),
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Stack::new(),
            source_info: SourceInfo::new(),
            ident_table: IdentifierTable::new(),
            class_table: GlobalClassTable::new(),
            instance_table: GlobalInstanceTable::new(),
            method_table: FxHashMap::default(),
            const_table: FxHashMap::default(),
            class_stack: vec![],
            scope_stack: vec![LocalScope::new()],
            global_stack: vec![GlobalScope::new()],
            env: vec![],
        }
    }

    pub fn show_source(&mut self, node: &Node) {
        self.source_info.show_loc(&node.loc);
    }

    pub fn init(&mut self, source_info: SourceInfo, ident_table: IdentifierTable, node: Node) {
        self.repl_init_method(source_info, ident_table);
        self.init_iseq(node);
    }

    pub fn repl_init_method(&mut self, source_info: SourceInfo, ident_table: IdentifierTable) {
        self.source_info = source_info;
        self.ident_table = ident_table;

        let id = self.ident_table.get_ident_id(&"top".to_string());
        let classref = self.new_class(id, 0);
        self.env.push(Env::ClassRef(classref));
        self.class_stack.push(classref);

        macro_rules! reg_method_table {
                     ( $($id:expr => $func:path),+ ) => {
                         $(
                             let id = self.ident_table.get_ident_id(&$id.to_string());
                             let info = MethodInfo::BuiltinFunc {
                                 name: $id.to_string(),
                                 func: $func
                             };
                             self.env_info().method_table.insert(id, info);
                         )+
                     };
                 }

        reg_method_table! {
            "puts" => VM::builtin_puts,
            "new" => VM::builtin_new,
            "to_i" => VM::builtin_to_i,
            "to_s" => VM::builtin_to_s,
            "assert" => VM::builtin_assert,
            "class" => VM::builtin_class,
            "times" => VM::builtin_times,
            "len" => VM::builtin_len,
            "each" => VM::builtin_each,
            "instance_variables" => VM::builtin_instance_variables
        }
    }

    fn env(&mut self) -> Env {
        self.env.last().unwrap().to_owned()
    }

    fn env_info(&mut self) -> &mut ClassInfo {
        let env = self.env();
        let env = match env {
            Env::ClassRef(r) => r,
            Env::InstanceRef(r) => self.class_ref_with_instance(r),
        };
        self.class_info_with_ref(env)
    }

    pub fn builtin_puts(&mut self, _receiver: Value, args: Args) -> Value {
        let args = args.args;
        for arg in args {
            println!("{}", self.val_to_s(&arg));
        }
        Value::Nil
    }

    pub fn builtin_new(&mut self, receiver: Value, _args: Args) -> Value {
        match receiver {
            Value::Class(class_ref) => {
                let instance = self.new_instance(class_ref);
                Value::Instance(instance)
            }
            _ => unimplemented!(),
        }
    }
    pub fn builtin_to_i(&mut self, receiver: Value, _args: Args) -> Value {
        let i = receiver.to_i();
        Value::FixNum(i)
    }

    pub fn builtin_to_s(&mut self, receiver: Value, _args: Args) -> Value {
        let s = self.val_to_s(&receiver);
        Value::String(s)
    }

    pub fn builtin_assert(&mut self, _receiver: Value, args: Args) -> Value {
        let args = args.args;
        if args.len() != 2 {
            unimplemented!();
        };
        let god = &args[0];
        let expected = &args[1];
        if god == expected {
            Value::Nil
        } else {
            panic!("assertion fail! Expected {:?}, bad god {:?}", god, expected)
        }
    }

    pub fn builtin_class(&mut self, receiver: Value, _args: Args) -> Value {
        let class = receiver.to_class();
        Value::SelfClass(class)
    }

    pub fn builtin_times(&mut self, receiver: Value, args: Args) -> Value {
        match receiver {
            Value::FixNum(n) => {
                for i in 0..n {
                    self.new_propagated_local_var_stack();

                    let imm_value = Value::FixNum(i);
                    let val = args.table as usize;
                    self.lvar_table_as_mut().insert(IdentId(val), imm_value);

                    self.eval_seq().unwrap_or_else(|err| {
                        panic!("Builtin#times: error occured while eval_node. {:?};", err)
                    });
                    self.init_eval_stack();
                    let local_scope = self.local_scope().clone();
                    self.scope_stack.pop();
                    for (id, n) in local_scope.lvar_table.into_iter() {
                        if self.local_scope().lvar_table.contains_key(&id) {
                            *self.local_scope().lvar_table.get_mut(&id).unwrap() = n;
                        }
                    }
                }
            }
            _ => panic!(
                "Builtin#times : must has array reciver, bud god {:?}.",
                receiver
            ),
        }
        Value::Nil
    }

    pub fn builtin_len(&mut self, receiver: Value, _args: Args) -> Value {
        match receiver {
            Value::Array(contents) => {
                let i = contents.len() as i64;
                Value::FixNum(i)
            }
            _ => unimplemented!(),
        }
    }

    pub fn builtin_each(&mut self, receiver: Value, args: Args) -> Value {
        match receiver {
            Value::Array(contents) => {
                for c in contents {
                    self.set_stack_pos(0);
                    self.new_propagated_local_var_stack();
                    self.lvar_table_as_mut()
                        .insert(IdentId(args.table.into()), c);
                    self.eval_seq().unwrap_or_else(|err| {
                        panic!("Builtin#each: error occured while eval_node. {:?}.", err)
                    });
                }
            }
            _ => panic!(
                "Builtin#each: must has array reciver, bud god {:?}.",
                receiver
            ),
        }
        Value::Nil
    }

    pub fn builtin_instance_variables(&mut self, receiver: Value, _args: Args) -> Value {
        match receiver {
            Value::Instance(instance_ref) => {
                let mut names = vec![];
                for key in self.instance_ref(instance_ref).instance_var.clone().keys() {
                    names.push(Value::String(format!(
                        "@{}",
                        self.ident_table.get_name(*key)
                    )));
                }
                Value::Array(names)
            }
            _ => panic!(
                "Builtin#instance_variables: must has array reciver, bud god {:?}.",
                receiver
            ),
        }
    }
}

impl VM {
    pub fn gen_array_with_len(&mut self, array: &Vec<Node>) {
        self.gen_nodes(array.to_vec());
        let len = array.len();
        self.gen_comp_usize(len);
    }

    pub fn gen_num_option(&mut self, id_option: Option<IdentId>) {
        match id_option {
            Some(id) => self.gen_comp_usize(*id),
            None => self.push_iseq(Inst::NIL),
        }
    }

    pub fn gen(&mut self, node: &Node) {
        // println!("&node.kind: {:?}", &node.kind);
        match &node.kind {
            NodeKind::None => self.push_iseq(Inst::NIL),
            NodeKind::SelfValue => self.push_iseq(Inst::SELF_VALUE),
            NodeKind::Number(num) => self.gen_comp_fixnum(*num),
            NodeKind::String(s) => {
                let id = self.stack.ident_table.get_ident_id(s);
                self.gen_comp_usize(*id);
                self.push_iseq(Inst::STRING);
            }
            NodeKind::BinOp(op, lhs, rhs) => match op {
                BinOp::Add => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.push_iseq(Inst::ADD);
                }
                BinOp::Sub => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.push_iseq(Inst::SUB);
                }
                BinOp::Mul => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.push_iseq(Inst::MUL);
                }
                BinOp::Div => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.push_iseq(Inst::DIV);
                }
                BinOp::Eq => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.push_iseq(Inst::EQ);
                }
                BinOp::Ne => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.push_iseq(Inst::NE);
                }
                BinOp::GT => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.push_iseq(Inst::GT);
                }
                BinOp::GE => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.push_iseq(Inst::GE);
                }
                BinOp::LT => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.push_iseq(Inst::LT);
                }
                BinOp::LE => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.push_iseq(Inst::LE);
                }
                BinOp::LAnd => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.push_iseq(Inst::LAND);
                }
                BinOp::LOr => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.push_iseq(Inst::LOR);
                }
            },
            NodeKind::Ident(id) => {
                let num = id.deref();
                self.gen_ident(*num);
            }
            NodeKind::Const(id) => {
                let num = id.deref();
                self.gen_const(*num);
            }
            NodeKind::TableIdent(id) => {
                let num = id.deref();
                self.gen_table_ident(*num);
            }
            NodeKind::CompStmt(nodes) => {
                self.gen_nodes(nodes.to_vec());
            }
            NodeKind::BlockDecl(body) => self.gen(body),
            NodeKind::Array(nodes) => {
                self.gen_array_with_len(nodes);
                self.push_iseq(Inst::ARRAY);
            }
            NodeKind::ArrayIndex(nodes, num) => {
                self.gen(nodes);
                self.gen_comp_fixnum(*num);
                self.push_iseq(Inst::ARRAY_INDEX);
            }
            NodeKind::If(cond_, then_, else_) => {
                self.gen_body(else_);
                self.gen_body(then_);
                self.gen_body(cond_);
                self.push_iseq(Inst::IF);
            }
            NodeKind::FuncDecl(id, params, body) => {
                let num = id.deref();
                self.gen_comp_usize(*num);
                self.gen_array_with_len(params);
                self.push_iseq(Inst::INIT_FUNC);
                self.gen_comp_usize(*num);
                self.gen_body(body);
                self.push_iseq(Inst::FUNC_DECL);
            }
            NodeKind::ClassDecl(id, body, inheritence_class_id) => {
                let num = id.deref();
                self.gen_comp_usize(*num);
                self.gen_body(body);
                self.gen_num_option(*inheritence_class_id);
                self.push_iseq(Inst::CLASS_DECL);
            }
            NodeKind::Assign(lhs, rhs) => {
                self.gen(rhs);
                match lhs.kind {
                    NodeKind::Ident(id) => self.gen_comp_fixnum(*id as i64),
                    _ => unimplemented!(),
                }
                self.push_iseq(Inst::ASSIGN);
            }
            NodeKind::Send(receiver, method, args) => {
                self.gen(receiver);
                let id = match method.kind {
                    NodeKind::Ident(id) => id,
                    _ => unimplemented!(),
                };
                self.gen_comp_usize(*id);
                self.gen(&args.table);
                self.gen_body(&args.node);
                self.gen_nodes_with_len(args.args.to_owned());
                self.push_iseq(Inst::SEND);
            }
            _ => {
                println!("&node.kind: {:?}", &node.kind);
                println!("{:?}", self.ident_table);
                unimplemented!();
            }
        }
    }

    fn gen_nodes(&mut self, nodes: Vec<Node>) {
        for node in nodes.clone() {
            self.gen(&node);
        }
    }

    fn gen_nodes_with_len(&mut self, nodes: Vec<Node>) {
        self.gen_nodes(nodes.to_vec());
        let len = nodes.len();
        self.gen_comp_usize(len);
    }

    fn gen_comp_fixnum(&mut self, num: i64) {
        self.push_iseq(Inst::FIXNUM);
        self.push_iseq((num >> 56) as u8);
        self.push_iseq((num >> 48) as u8);
        self.push_iseq((num >> 40) as u8);
        self.push_iseq((num >> 32) as u8);
        self.push_iseq((num >> 24) as u8);
        self.push_iseq((num >> 16) as u8);
        self.push_iseq((num >> 8) as u8);
        self.push_iseq(num as u8);
    }

    pub fn gen_ident(&mut self, num: usize) {
        self.push_iseq(Inst::IDENT);
        self.push_iseq((num >> 56) as u8);
        self.push_iseq((num >> 48) as u8);
        self.push_iseq((num >> 40) as u8);
        self.push_iseq((num >> 32) as u8);
        self.push_iseq((num >> 24) as u8);
        self.push_iseq((num >> 16) as u8);
        self.push_iseq((num >> 8) as u8);
        self.push_iseq(num as u8);
    }

    pub fn gen_const(&mut self, num: usize) {
        self.push_iseq(Inst::CONST);
        self.push_iseq((num >> 56) as u8);
        self.push_iseq((num >> 48) as u8);
        self.push_iseq((num >> 40) as u8);
        self.push_iseq((num >> 32) as u8);
        self.push_iseq((num >> 24) as u8);
        self.push_iseq((num >> 16) as u8);
        self.push_iseq((num >> 8) as u8);
        self.push_iseq(num as u8);
    }

    pub fn gen_table_ident(&mut self, num: usize) {
        self.push_iseq(Inst::TABLE_IDENT);
        self.push_iseq((num >> 56) as u8);
        self.push_iseq((num >> 48) as u8);
        self.push_iseq((num >> 40) as u8);
        self.push_iseq((num >> 32) as u8);
        self.push_iseq((num >> 24) as u8);
        self.push_iseq((num >> 16) as u8);
        self.push_iseq((num >> 8) as u8);
        self.push_iseq(num as u8);
    }

    pub fn gen_comp_usize(&mut self, num: usize) {
        let num = num as i64;
        self.gen_comp_fixnum(num);
    }

    pub fn lvar_table_as_mut(&mut self) -> &mut ValueTable {
        &mut self.scope_stack.last_mut().unwrap().lvar_table
    }

    pub fn local_scope(&mut self) -> &mut LocalScope {
        self.scope_stack.last_mut().unwrap()
    }

    fn new_class_info(&mut self, id: IdentId, ptr: usize) -> ClassRef {
        let name = self.ident_table.get_name(id);
        self.class_table.new_class(id, name, ptr)
    }

    // fn new_class_info(&mut self, id: IdentId, body: Node) -> ClassRef {
    //     let name = self.ident_table.get_name(id).clone();
    //     self.class_table.new_class(id, name, body)
    // }

    fn new_propagated_local_var_stack(&mut self) {
        let mut last_scope_stack = self.scope_stack.last_mut().unwrap().to_owned();
        last_scope_stack.propagated_table = last_scope_stack.clone().lvar_table;
        self.scope_stack.push(last_scope_stack);
    }

    pub fn init_iseq(&mut self, node: Node) {
        self.stack.stack_poses.push(0);
        self.stack.iseq_poses.push(0);
        self.stack.iseqs.insert(0, vec![]);
        self.stack.exec_stack.push(vec![]);
        self.gen(&node);
        self.push_iseq(Inst::END);
        // println!("{:?}", self.stack.iseqs);
        // println!("{:?}", self.env_info().method_table);
    }

    fn get_val(&mut self) -> usize {
        match self.iseq() {
            Inst::FIXNUM | Inst::IDENT | Inst::TABLE_IDENT | Inst::CONST => self.plus_stack_pos(1),
            _ => unimplemented!(),
        }
        let mut num = 0;
        num += (self.iseq() as usize) << 56;
        num += (self.iseq_idx(1) as usize) << 48;
        num += (self.iseq_idx(2) as usize) << 40;
        num += (self.iseq_idx(3) as usize) << 32;
        num += (self.iseq_idx(4) as usize) << 24;
        num += (self.iseq_idx(5) as usize) << 16;
        num += (self.iseq_idx(6) as usize) << 8;
        num += self.iseq_idx(7) as usize;
        self.plus_stack_pos(8);
        num as usize
    }

    pub fn insert_num(&mut self, num: usize) {
        let pos = self.stack_pos();
        self.current_iseq().insert(pos, Inst::FIXNUM as u8);
        self.current_iseq().insert(pos + 1, (num >> 56) as u8);
        self.current_iseq().insert(pos + 2, (num >> 48) as u8);
        self.current_iseq().insert(pos + 3, (num >> 40) as u8);
        self.current_iseq().insert(pos + 4, (num >> 32) as u8);
        self.current_iseq().insert(pos + 5, (num >> 24) as u8);
        self.current_iseq().insert(pos + 6, (num >> 16) as u8);
        self.current_iseq().insert(pos + 7, (num >> 8) as u8);
        self.current_iseq().insert(pos + 8, num as u8);
    }

    pub fn insert_iseq(&mut self, inst: u8) {
        let pos = self.stack_pos();
        self.current_iseq().insert(pos, inst);
    }

    pub fn push_iseq(&mut self, inst: u8) {
        self.current_iseq().push(inst);
    }

    pub fn plus_stack_pos(&mut self, num: usize) {
        *self.stack.stack_poses.last_mut().unwrap() += num;
    }

    pub fn set_stack_pos(&mut self, num: usize) {
        *self.stack.stack_poses.last_mut().unwrap() = num;
    }

    pub fn stack_pos_as_mut(&mut self) -> usize {
        *self.stack.stack_poses.last_mut().unwrap()
    }

    pub fn stack_pos(&mut self) -> usize {
        *self.stack.stack_poses.last().unwrap()
    }

    pub fn iseq_pos(&mut self) -> usize {
        *self.stack.iseq_poses.last().unwrap()
    }

    pub fn iseq_pos_mut(&mut self) -> &mut usize {
        &mut *self.stack.iseq_poses.last_mut().unwrap()
    }

    pub fn current_iseq(&mut self) -> &mut Vec<ISeq> {
        let pos = self.iseq_pos();
        self.stack.iseqs.get_mut(&pos).unwrap()
    }

    pub fn gen_body(&mut self, node: &Node) {
        match node.kind {
            NodeKind::None => self.push_iseq(Inst::NIL),
            _ => {
                let ptr = node as *const _ as usize;
                self.stack.iseq_poses.push(ptr);
                let iseq_pos = self.iseq_pos();
                self.stack.iseqs.insert(iseq_pos, vec![]);
                self.gen(node);
                self.current_iseq().push(Inst::END);
                self.stack.iseq_poses.pop().unwrap();
                self.gen_comp_usize(ptr);
            }
        }
    }

    fn copy_ptr(&mut self) -> usize {
        match self.copy_exec_stack() {
            Value::FixNum(n) => n as usize,
            Value::Nil => 0,
            _ => unimplemented!(),
        }
    }

    fn get_ptr(&mut self) -> usize {
        match self.pop_value() {
            Value::FixNum(n) => n as usize,
            Value::Nil => 0,
            _ => unimplemented!(),
        }
    }

    fn push_fixnum(&mut self) -> Value {
        let val = self.get_val();
        Value::FixNum(val as i64)
    }

    pub fn copy_exec_stack(&mut self) -> Value {
        self.exec_stack().last().unwrap().to_owned()
    }

    pub fn pop_value(&mut self) -> Value {
        self.exec_stack().pop().unwrap()
    }

    fn class_stack(&mut self) -> Value {
        let classref = self.class_stack.last().unwrap_or_else(|| {
            panic!("Evaluator#eval: class stack is empty");
        });
        Value::Class(*classref)
    }

    fn get_params(&mut self) -> Vec<IdentId> {
        let mut arr: Vec<IdentId> = vec![];
        let len = self.pop_value().value();
        for _ in 0..len {
            arr.push(self.pop_value().ident());
        }
        arr.reverse();
        arr
    }

    fn get_array(&mut self) -> Vec<Value> {
        let mut arr: Vec<Value> = vec![];
        let len = self.pop_value().value();
        for _ in 0..len {
            arr.push(self.pop_value());
        }
        arr.reverse();
        arr
    }

    fn save_exec_stack(&mut self) {
        self.stack.exec_stack.push(vec![]);
    }

    fn save_eval_info(&mut self) {
        self.stack.stack_poses.push(0);
        self.stack.iseq_poses.push(0);
    }

    fn eval_body(&mut self) -> Value {
        let ptr = self.get_ptr();
        if ptr == 0 {
            Value::Nil
        } else {
            self.save_eval_info();
            *self.iseq_pos_mut() = ptr;
            self.eval_seq().unwrap();
            self.return_stack();
            self.pop_value()
        }
    }

    fn eval_body_with_args_ptr(&mut self, ptr: usize) {
        if ptr == 0 {
            unimplemented!();
        } else {
            self.save_eval_info();
            *self.iseq_pos_mut() = ptr;
            self.eval_seq().unwrap();
            self.return_stack();
        }
    }

    fn get_body(&mut self) -> Vec<ISeq> {
        let ptr = self.get_ptr();
        if ptr == 0 {
            vec![]
        } else {
            *self.iseq_pos_mut() = ptr;
            self.current_iseq().clone()
        }
    }

    pub fn iseq(&mut self) -> u8 {
        let iseq_pos = self.iseq_pos();
        let stack_pos = self.stack_pos();
        self.stack.iseqs[&iseq_pos][stack_pos]
    }

    pub fn iseq_idx(&mut self, num: i64) -> u8 {
        let iseq_pos = self.iseq_pos();
        let stack_pos = self.stack_pos();
        self.stack.iseqs[&iseq_pos][(stack_pos as i64 + num) as usize]
    }

    pub fn init_eval_stack(&mut self) {
        self.set_stack_pos(0);
    }

    pub fn clear_last_exec_stack(&mut self) {
        self.stack.exec_stack.pop().unwrap();
    }

    pub fn push_last_eval_stack(&mut self) {
        let val = self.exec_stack().last().unwrap().clone();
        self.clear_last_exec_stack();
        self.exec_stack().push(val);
    }

    pub fn return_stack(&mut self) {
        self.stack.stack_poses.pop().unwrap();
        self.stack.iseq_poses.pop().unwrap();
    }

    pub fn exec_stack(&mut self) -> &mut Vec<Value> {
        &mut *self.stack.exec_stack.last_mut().unwrap()
    }

    pub fn eval_seq(&mut self) -> Result<(), RuntimeError> {
        // println!("self.stack.iseqs: {:?}", self.stack.iseqs);
        loop {
            // println!(
            //     "self.iseq(): {:?}, self.iseq_pos(): {:?}, self.stack.iseq_poses: {:?}",
            //     self.iseq(),
            //     self.iseq_pos(),
            //     self.stack.iseq_poses
            // );
            // println!("self.stack.exec_stack: {:?}", self.stack.exec_stack);
            match self.iseq() {
                Inst::NIL => {
                    self.plus_stack_pos(1);
                    self.exec_stack().push(Value::Nil);
                }
                Inst::SELF_VALUE => {
                    self.plus_stack_pos(1);
                    let val = self.class_stack();
                    self.exec_stack().push(val);
                }
                Inst::ASSIGN => {
                    self.plus_stack_pos(1);
                    let id = self.pop_value().ident();
                    let rhs = self.pop_value();
                    match self.lvar_table_as_mut().get_mut(&id) {
                        Some(val) => {
                            *val = rhs.clone();
                        }
                        None => {
                            self.lvar_table_as_mut().insert(id, rhs.clone());
                        }
                    }
                    self.exec_stack().push(rhs);
                }
                Inst::END => {
                    self.plus_stack_pos(1);
                    if let Some(val) = self.exec_stack().pop() {
                        self.exec_stack().push(val);
                    } else {
                        self.exec_stack().push(Value::Nil);
                    }
                    return Ok(());
                }
                Inst::FIXNUM => {
                    let val = self.push_fixnum();
                    self.exec_stack().push(val);
                }
                Inst::STRING => {
                    self.plus_stack_pos(1);
                    let id = self.pop_value().ident();
                    let name = self.stack.ident_table.get_name(IdentId(*id));
                    self.env_info().ident_table.get_ident_id(&name);
                    let val = Value::String(name);
                    self.exec_stack().push(val);
                }
                Inst::ADD => {
                    self.plus_stack_pos(1);
                    let rhs = self.pop_value();
                    let lhs = self.pop_value();
                    let val = self.eval_add(lhs, rhs)?;
                    self.exec_stack().push(val);
                }
                Inst::SUB => {
                    self.plus_stack_pos(1);
                    let rhs = self.pop_value();
                    let lhs = self.pop_value();
                    let val = self.eval_sub(lhs, rhs)?;
                    self.exec_stack().push(val);
                }
                Inst::MUL => {
                    self.plus_stack_pos(1);
                    let rhs = self.pop_value();
                    let lhs = self.pop_value();
                    let val = self.eval_mul(lhs, rhs)?;
                    self.exec_stack().push(val);
                }
                Inst::DIV => {
                    self.plus_stack_pos(1);
                    let rhs = self.pop_value();
                    let lhs = self.pop_value();
                    let val = self.eval_div(lhs, rhs)?;
                    self.exec_stack().push(val);
                }
                Inst::EQ => {
                    self.plus_stack_pos(1);
                    let rhs = self.pop_value();
                    let lhs = self.pop_value();
                    let val = self.eval_eq(lhs, rhs)?;
                    self.exec_stack().push(val);
                }
                Inst::NE => {
                    self.plus_stack_pos(1);
                    let rhs = self.pop_value();
                    let lhs = self.pop_value();
                    let val = self.eval_neq(lhs, rhs)?;
                    self.exec_stack().push(val);
                }
                Inst::GT => {
                    self.plus_stack_pos(1);
                    let rhs = self.pop_value();
                    let lhs = self.pop_value();
                    let val = self.eval_gt(lhs, rhs)?;
                    self.exec_stack().push(val);
                }
                Inst::GE => {
                    self.plus_stack_pos(1);
                    let rhs = self.pop_value();
                    let lhs = self.pop_value();
                    let val = self.eval_ge(lhs, rhs)?;
                    self.exec_stack().push(val);
                }
                Inst::SEND => {
                    self.plus_stack_pos(1);
                    self.save_eval_info();
                    let args = self.get_array();
                    let body = self.get_body();
                    let table = self.pop_value().value() as u8;
                    let id = self.pop_value().ident();
                    let info = self.get_method_info(id);
                    let receiver = self.pop_value();
                    let f = self.push_env(receiver.clone());
                    self.push_env(receiver.clone());

                    match info {
                        MethodInfo::RubyFunc {
                            params,
                            ptr,
                            local_scope,
                        } => {
                            self.new_propagated_local_var_stack();
                            let args_len = args.len();
                            self.scope_stack.push(local_scope);
                            for (i, param) in params.iter().enumerate() {
                                let arg = if args_len > i {
                                    args[i].clone()
                                } else {
                                    Value::Nil
                                };
                                self.lvar_table_as_mut().insert(*param, arg);
                            }
                            self.save_exec_stack();
                            self.eval_body_with_args_ptr(ptr);
                            self.push_last_eval_stack();
                            self.pop_env_if_true(f);
                            self.scope_stack.pop();
                        }
                        MethodInfo::BuiltinFunc { func, .. } => {
                            let args = Args { body, args, table };
                            self.pop_env_if_true(f);
                            let val = func(self, receiver, args);
                            self.exec_stack().push(val);
                        }
                    }
                    self.return_stack();
                }
                Inst::ARRAY => {
                    self.plus_stack_pos(1);
                    let arr = self.get_array();
                    self.exec_stack().push(Value::Array(arr));
                }
                Inst::ARRAY_INDEX => {
                    self.plus_stack_pos(1);
                    let idx = self.pop_value().value() as usize;
                    match self.pop_value() {
                        Value::Array(arr) => {
                            let val = arr[idx].clone();
                            self.exec_stack().push(val);
                        }
                        _ => unreachable!(),
                    }
                }
                Inst::IF => {
                    self.plus_stack_pos(1);
                    let val = if self.eval_body().bool() {
                        let val = self.eval_body();
                        self.pop_value();
                        val
                    } else {
                        self.pop_value();
                        self.eval_body()
                    };
                    self.exec_stack().push(val);
                }
                Inst::INIT_FUNC => {
                    self.plus_stack_pos(1);
                    let params = self.get_params();
                    let id = self.pop_value().ident();
                    let mut local_scope = LocalScope {
                        lvar_table: FxHashMap::default(),
                        propagated_table: FxHashMap::default(),
                    };

                    self.new_propagated_local_var_stack();
                    let last_scope_stack = self.scope_stack.last_mut().unwrap().to_owned();
                    local_scope.lvar_table = last_scope_stack.lvar_table;
                    local_scope.propagated_table = last_scope_stack.propagated_table;

                    for n in &params {
                        self.lvar_table_as_mut()
                            .insert(IdentId(**n), Value::FixNum(0));
                        local_scope.lvar_table.insert(*n, Value::FixNum(0));
                    }
                    self.env_info().method_table.insert(
                        id,
                        MethodInfo::RubyFunc {
                            params,
                            ptr: 0,
                            local_scope,
                        },
                    );
                }
                Inst::FUNC_DECL => {
                    self.plus_stack_pos(1);
                    self.save_eval_info();
                    let ptr_ = self.copy_ptr();
                    self.get_body();
                    self.return_stack();
                    let id = self.pop_value().ident();

                    match self.env_info().method_table.get_mut(&id).unwrap() {
                        MethodInfo::RubyFunc { ptr, .. } => {
                            *ptr = ptr_;
                        }
                        _ => unreachable!(),
                    }
                    self.scope_stack.pop();
                }
                Inst::CLASS_DECL => {
                    self.plus_stack_pos(1);
                    let inheritence_class_id = self.pop_value().option_ident();
                    self.save_eval_info();
                    let ptr = self.copy_ptr();
                    self.get_body();
                    self.return_stack();
                    let id = self.pop_value().ident();
                    let mut info = self.new_class_info(id, ptr);
                    let val = info.to_val();
                    self.add_subclass(info, inheritence_class_id);
                    self.const_table.insert(id, val);
                }
                Inst::IDENT => {
                    let mut id = self.push_fixnum();
                    match self.lvar_table_as_mut().clone().get(&id.ident()) {
                        Some(val) => self.exec_stack().push(val.to_owned()),
                        None => {
                            println!("self.lvar_table_as_mut(): {:?}", self.lvar_table_as_mut());
                            println!("self.ident_table: {:?}", self.ident_table);
                            println!("id: {:?}", id);
                            panic!("undefined local variable.")
                        }
                    }
                }
                Inst::CONST => {
                    let id = self.push_fixnum().ident();
                    match self.const_table.clone().get(&id) {
                        Some(val) => self.exec_stack().push(val.to_owned()),
                        None => {
                            println!("self.lvar_table_as_mut(): {:?}", self.lvar_table_as_mut());
                            println!("self.ident_table: {:?}", self.ident_table);
                            println!("id: {:?}", id);
                            panic!("undefined local variable.")
                        }
                    }
                }
                Inst::TABLE_IDENT => {
                    let val = self.push_fixnum();
                    self.exec_stack().push(val);
                }
                _ => unimplemented!(),
            }
        }
    }

    pub fn eval(&mut self) -> EvalResult {
        self.eval_seq()?;
        Ok(self.pop_value())
    }
    fn class_info_with_ref(&mut self, class_ref: ClassRef) -> &mut ClassInfo {
        self.class_table.table.get_mut(&class_ref).unwrap()
    }

    fn class_info_with_id(&mut self, ident_id: IdentId) -> ClassInfo {
        for t in self.class_table.table.to_owned() {
            if t.1.id == ident_id {
                return t.1;
            }
        }
        unimplemented!()
    }

    // fn class_ref_with_id(&mut self, ident_id: IdentId) -> ClassRef {
    //     for t in self.class_table.table.to_owned() {
    //         if t.1.id == ident_id {
    //             return t.0;
    //         }
    //     }
    //     unimplemented!()
    // }

    fn class_info_with_instance(&mut self, instance_ref: InstanceRef) -> &mut ClassInfo {
        let class_ref = self.instance_ref(instance_ref).class_id;
        self.class_info_with_ref(class_ref)
    }

    fn class_ref_with_instance(&mut self, instance_ref: InstanceRef) -> ClassRef {
        self.instance_ref(instance_ref).class_id
    }

    fn instance_ref(&mut self, instance_ref: InstanceRef) -> &mut InstanceInfo {
        self.instance_table.get_mut(instance_ref)
    }

    // fn instance_value(&mut self, instance_ref: InstanceRef, id: IdentId) -> Value {
    //     self.instance_table
    //         .get_mut(instance_ref)
    //         .instance_var
    //         .get(&id)
    //         .unwrap()
    //         .to_owned()
    // }

    // fn class_value(&mut self, class_ref: ClassRef, id: IdentId) -> Value {
    //     self.class_info_with_ref(class_ref)
    //         .class_var
    //         .get_mut(&id)
    //         .unwrap()
    //         .to_owned()
    // }

    // fn class_value_with_instance(&mut self, instance_ref: InstanceRef, id: IdentId) -> Value {
    //     self.class_info_with_instance(instance_ref)
    //         .class_var
    //         .get_mut(&id)
    //         .unwrap()
    //         .to_owned()
    // }

    fn add_subclass(&mut self, info: ClassRef, inheritence_class_id: Option<IdentId>) {
        if let Some(inheritence_class_id) = inheritence_class_id {
            let class = self.class_info_with_ref(info);
            class
                .subclass
                .insert(inheritence_class_id, ClassRef(*inheritence_class_id + 1));
        }
    }

    // fn add_subclass(&mut self, info: ClassRef, inheritence_class_id: Option<IdentId>) {
    //     if let Some(inheritence_class_id) = inheritence_class_id {
    //         let class = self.class_info_with_ref(info);
    //         class
    //             .subclass
    //             .insert(inheritence_class_id, ClassRef(*inheritence_class_id + 1));
    //     }
    // }

    fn get_method_info(&mut self, id: IdentId) -> MethodInfo {
        for env in self.env.clone().iter_mut().rev() {
            let r = match env {
                Env::ClassRef(ClassRef(r)) => *self.class_info_with_ref(ClassRef(*r)).id,
                Env::InstanceRef(InstanceRef(r)) => {
                    *self.class_info_with_instance(InstanceRef(*r)).id
                }
            };
            let class_ref = self.class_info_with_id(IdentId(r)).clone();
            // println!("id: {:?}", id);
            // println!("class_ref.method_table: {:?}", class_ref.method_table);
            match class_ref.method_table.get(&id) {
                Some(info) => return info.to_owned(),
                None => {
                    for r in class_ref.subclass.values() {
                        if let Some(info) = self.class_info_with_ref(*r).method_table.get(&id) {
                            return info.to_owned();
                        }
                    }
                }
            }
        }
        unimplemented!("undefined function.");
    }

    fn push_env(&mut self, val: Value) -> bool {
        match val {
            Value::Class(r) => {
                self.env.push(Env::ClassRef(r));
                true
            }
            Value::Instance(r) => {
                self.env.push(Env::InstanceRef(r));
                true
            }
            _ => false,
        }
    }

    fn pop_env_if_true(&mut self, b: bool) {
        if b {
            self.env.pop().unwrap();
        }
    }

    fn eval_add(&mut self, lhs: Value, rhs: Value) -> EvalResult {
        match (lhs, rhs) {
            (Value::FixNum(lhs), Value::FixNum(rhs)) => Ok(Value::FixNum(lhs + rhs)),
            (Value::FixDecimalNum(lhs), Value::FixNum(rhs)) => {
                Ok(Value::FixDecimalNum(lhs + (rhs as f64)))
            }
            (Value::FixNum(lhs), Value::FixDecimalNum(rhs)) => {
                Ok(Value::FixDecimalNum((lhs as f64) + rhs))
            }
            (Value::FixDecimalNum(lhs), Value::FixDecimalNum(rhs)) => {
                Ok(Value::FixDecimalNum(lhs + rhs))
            }
            (_, _) => unimplemented!(),
        }
    }

    fn eval_sub(&mut self, lhs: Value, rhs: Value) -> EvalResult {
        match (lhs, rhs) {
            (Value::FixNum(lhs), Value::FixNum(rhs)) => Ok(Value::FixNum(lhs - rhs)),
            (Value::FixDecimalNum(lhs), Value::FixNum(rhs)) => {
                Ok(Value::FixDecimalNum(lhs - (rhs as f64)))
            }
            (Value::FixNum(lhs), Value::FixDecimalNum(rhs)) => {
                Ok(Value::FixDecimalNum((lhs as f64) - rhs))
            }
            (Value::FixDecimalNum(lhs), Value::FixDecimalNum(rhs)) => {
                Ok(Value::FixDecimalNum(lhs - rhs))
            }
            (_, _) => unimplemented!(),
        }
    }

    fn eval_mul(&mut self, lhs: Value, rhs: Value) -> EvalResult {
        match (lhs, rhs) {
            (Value::FixNum(lhs), Value::FixNum(rhs)) => Ok(Value::FixNum(lhs * rhs)),
            (Value::FixDecimalNum(lhs), Value::FixNum(rhs)) => {
                Ok(Value::FixDecimalNum(lhs * (rhs as f64)))
            }
            (Value::FixNum(lhs), Value::FixDecimalNum(rhs)) => {
                Ok(Value::FixDecimalNum((lhs as f64) * rhs))
            }
            (Value::FixDecimalNum(lhs), Value::FixDecimalNum(rhs)) => {
                Ok(Value::FixDecimalNum(lhs * rhs))
            }
            (_, _) => unimplemented!(),
        }
    }

    fn eval_div(&mut self, lhs: Value, rhs: Value) -> EvalResult {
        match (lhs, rhs) {
            (Value::FixNum(lhs), Value::FixNum(rhs)) => Ok(Value::FixNum(lhs / rhs)),
            (Value::FixDecimalNum(lhs), Value::FixNum(rhs)) => {
                Ok(Value::FixDecimalNum(lhs / (rhs as f64)))
            }
            (Value::FixNum(lhs), Value::FixDecimalNum(rhs)) => {
                Ok(Value::FixDecimalNum((lhs as f64) / rhs))
            }
            (Value::FixDecimalNum(lhs), Value::FixDecimalNum(rhs)) => {
                Ok(Value::FixDecimalNum(lhs / rhs))
            }
            (_, _) => unimplemented!(),
        }
    }

    fn eval_eq(&mut self, lhs: Value, rhs: Value) -> EvalResult {
        match (lhs, rhs) {
            (Value::FixNum(lhs), Value::FixNum(rhs)) => Ok(Value::Bool(lhs == rhs)),
            (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs == rhs)),
            (_, _) => unimplemented!(),
        }
    }

    fn eval_neq(&mut self, lhs: Value, rhs: Value) -> EvalResult {
        match (lhs, rhs) {
            (Value::FixNum(lhs), Value::FixNum(rhs)) => Ok(Value::Bool(lhs != rhs)),
            (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs != rhs)),
            (_, _) => unimplemented!("NoMethodError: '!='"),
        }
    }

    fn eval_ge(&mut self, lhs: Value, rhs: Value) -> EvalResult {
        match (lhs, rhs) {
            (Value::FixNum(lhs), Value::FixNum(rhs)) => Ok(Value::Bool(lhs >= rhs)),
            (_, _) => unimplemented!("NoMethodError: '>='"),
        }
    }

    fn eval_gt(&mut self, lhs: Value, rhs: Value) -> EvalResult {
        match (lhs, rhs) {
            (Value::FixNum(lhs), Value::FixNum(rhs)) => Ok(Value::Bool(lhs > rhs)),
            (_, _) => unimplemented!("NoMethodError: '>'"),
        }
    }
}

impl VM {
    pub fn new_class(&mut self, id: IdentId, ptr: usize) -> ClassRef {
        let name = self.ident_table.get_name(id);
        let class_ref = self.class_table.new_class(id, name, ptr);
        let id = self.ident_table.get_ident_id(&"new".to_string());
        let info = MethodInfo::BuiltinFunc {
            name: "new".to_string(),
            func: VM::builtin_new,
        };

        self.class_table
            .get_mut(class_ref)
            .add_class_method(id, info);
        class_ref
    }
    pub fn new_instance(&mut self, class_id: ClassRef) -> InstanceRef {
        let class_info = self.class_table.get(class_id);
        let class_name = &class_info.name;
        let subclass = &class_info.subclass;
        let instance_var = class_info.instance_var.to_owned();
        self.instance_table.new_instance(
            class_id,
            class_name.to_owned(),
            instance_var,
            subclass.to_owned(),
        )
    }
}

impl VM {
    pub fn val_to_s(&mut self, val: &Value) -> String {
        match val {
            Value::Nil => "".to_string(),
            Value::Bool(b) => match b {
                true => "true".to_string(),
                false => "false".to_string(),
            },
            Value::FixNum(i) => i.to_string(),
            Value::FixDecimalNum(f) => f.to_string(),
            Value::String(s) => s.clone(),
            Value::Class(class) => {
                let class_info = self.class_table.get(*class);
                format!("#<{}:{:?}>", class_info.name, class)
            }
            Value::Instance(instance) => {
                let info = self.instance_table.get(*instance);
                format!("#<{}:{:?}>", info.class_name, instance)
            }
            Value::SelfClass(c) => {
                format!("{:?}", c)
            }
            Value::Array(v) => {
                format!("{:?}", v)
            }
        }
    }
}
