use crate::args::args::*;
use crate::class::class::*;
use crate::instance::instance::*;
use crate::node::node::*;
use crate::util::annot::*;
use crate::util::util::*;
use crate::value::value::*;
use crate::vm::inst::*;
use rustc_hash::FxHashMap;

pub type ISeq = u8;

#[derive(Debug, Clone)]
pub struct VM {
    pub iseq: Vec<ISeq>,
    pub stack_pos: usize,
    pub exec_stack: Vec<Value>,
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
        params: ISeq,
        body: Box<u8>,
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
            MethodInfo::RubyFunc { params, body, .. } => {
                write!(f, "RubyFunc {:?} {:?}", params, body)
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
            iseq: vec![],
            stack_pos: 0,
            exec_stack: vec![],
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
        let classref = self.new_class(id, Node::new_comp_stmt());
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
            // "times" => VM::builtin_times,
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
        let i = self.val_to_i(&receiver);
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
        let class = self.val_to_class(&receiver);
        Value::SelfClass(class)
    }

    // pub fn builtin_times(&mut self, receiver: Value, args: Args) -> Value {
    //     match receiver {
    //         Value::FixNum(n) => {
    //             for i in 0..n {
    //                 self.new_propagated_local_var_stack();

    //                 if let NodeKind::Ident(id) = args.table.kind {
    //                     let imm_value = Value::FixNum(i);
    //                     let val = args.table as u8;
    //                     self.lvar_table_as_mut().insert(val, imm_value);
    //                 }

    //                 self.eval_seq().unwrap_or_else(|err| {
    //                     panic!("Builtin#times: error occured while eval_node. {:?};", err)
    //                 });
    //                 let local_scope = self.local_scope().clone();
    //                 self.scope_stack.pop();
    //                 for (id, n) in local_scope.lvar_table.into_iter() {
    //                     if self.local_scope().lvar_table.contains_key(&id) {
    //                         *self.local_scope().lvar_table.get_mut(&id).unwrap() = n;
    //                     }
    //                 }
    //             }
    //         }
    //         _ => panic!(
    //             "Builtin#times : must has array reciver, bud god {:?}.",
    //             receiver
    //         ),
    //     }
    //     Value::Nil
    // }

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
    pub fn lvar_table_as_mut(&mut self) -> &mut ValueTable {
        &mut self.scope_stack.last_mut().unwrap().lvar_table
    }

    pub fn local_scope(&mut self) -> &mut LocalScope {
        self.scope_stack.last_mut().unwrap()
    }

    // fn new_class_info(&mut self, id: IdentId, body: Node) -> ClassRef {
    //     let name = self.ident_table.get_name(id).clone();
    //     self.class_table.new_class(id, name, body)
    // }

    fn new_propagated_local_var_stack(&mut self) {
        let mut last_scope_stack = self.scope_stack.last().unwrap().to_owned();
        last_scope_stack.propagated_table = last_scope_stack.clone().lvar_table;
        self.scope_stack.push(last_scope_stack);
    }

    pub fn init_iseq(&mut self, node: Node) {
        self.gen(&node);
        self.iseq.push(Inst::END);
        println!("iseq: {:?}", self.iseq);
    }

    fn get_val(&mut self) -> usize {
        match self.iseq[self.stack_pos] {
            Inst::FIXNUM => self.stack_pos += 1,
            _ => unimplemented!(),
        }
        let mut num = 0;
        num += (self.iseq[self.stack_pos] as usize) << 56;
        num += (self.iseq[self.stack_pos + 1] as usize) << 48;
        num += (self.iseq[self.stack_pos + 2] as usize) << 40;
        num += (self.iseq[self.stack_pos + 3] as usize) << 32;
        num += (self.iseq[self.stack_pos + 4] as usize) << 24;
        num += (self.iseq[self.stack_pos + 5] as usize) << 16;
        num += (self.iseq[self.stack_pos + 6] as usize) << 8;
        num += self.iseq[self.stack_pos + 7] as usize;
        self.stack_pos += 8;
        num as usize
    }

    fn push_fixnum(&mut self) -> Value {
        let val = self.get_val();
        Value::FixNum(val as i64)
    }

    pub fn exec_stack(&mut self) -> Value {
        self.exec_stack.pop().unwrap()
    }

    fn class_stack(&mut self) -> Value {
        let classref = self.class_stack.last().unwrap_or_else(|| {
            panic!("Evaluator#eval: class stack is empty");
        });
        Value::Class(*classref)
    }

    fn get_array(&mut self) -> Vec<Value> {
        let mut arr: Vec<Value> = vec![];
        let len = self.exec_stack().value();
        for _ in 0..len {
            arr.push(self.exec_stack());
        }
        arr.reverse();
        arr
    }

    pub fn eval_seq(&mut self) -> Result<(), RuntimeError> {
        loop {
            println!(
                "self.stack.iseq[self.stack_pos]: {}",
                self.iseq[self.stack_pos]
            );
            match self.iseq[self.stack_pos] {
                Inst::NIL => {
                    self.stack_pos += 1;
                    self.exec_stack.push(Value::Nil);
                }
                Inst::END => {
                    if let Some(val) = self.exec_stack.pop() {
                        // return Ok(val);
                        self.exec_stack.push(val);
                    } else {
                        // return Ok(Value::Nil);
                        self.exec_stack.push(Value::Nil);
                    }
                    return Ok(());
                }
                // Inst::FIXNUM => Ok(Value::FixNum(self.get_val() as i64)),
                Inst::FIXNUM => {
                    let val = self.push_fixnum();
                    self.exec_stack.push(val);
                }
                Inst::SELF_VALUE => {
                    self.stack_pos += 1;
                    let val = self.class_stack();
                    self.exec_stack.push(val);
                }
                Inst::STRING => {
                    self.stack_pos += 1;
                    let id = self.get_val();
                    let name = self.ident_table.get_name(IdentId(id));
                    self.env_info().ident_table.get_ident_id(&name);
                    let val = Value::String(name);
                    self.exec_stack.push(val);
                }
                Inst::ADD => {
                    self.stack_pos += 1;
                    let rhs = self.exec_stack();
                    let lhs = self.exec_stack();
                    let val = self.eval_add(rhs, lhs)?;
                    self.exec_stack.push(val);
                }
                Inst::SUB => {
                    self.stack_pos += 1;
                    let rhs = self.exec_stack();
                    let lhs = self.exec_stack();
                    let val = self.eval_sub(rhs, lhs)?;
                    self.exec_stack.push(val);
                }
                Inst::MUL => {
                    self.stack_pos += 1;
                    let rhs = self.exec_stack();
                    let lhs = self.exec_stack();
                    let val = self.eval_mul(rhs, lhs)?;
                    self.exec_stack.push(val);
                }
                Inst::DIV => {
                    self.stack_pos += 1;
                    let rhs = self.exec_stack();
                    let lhs = self.exec_stack();
                    let val = self.eval_div(rhs, lhs)?;
                    self.exec_stack.push(val);
                }
                Inst::EQ => {
                    self.stack_pos += 1;
                    let rhs = self.exec_stack();
                    let lhs = self.exec_stack();
                    let val = self.eval_eq(rhs, lhs)?;
                    self.exec_stack.push(val);
                }
                Inst::NE => {
                    self.stack_pos += 1;
                    let rhs = self.exec_stack();
                    let lhs = self.exec_stack();
                    let val = self.eval_neq(rhs, lhs)?;
                    self.exec_stack.push(val);
                }
                Inst::GT => {
                    self.stack_pos += 1;
                    let rhs = self.exec_stack();
                    let lhs = self.exec_stack();
                    let val = self.eval_gt(rhs, lhs)?;
                    self.exec_stack.push(val);
                }
                Inst::GE => {
                    let rhs = self.exec_stack();
                    let lhs = self.exec_stack();
                    let val = self.eval_ge(rhs, lhs)?;
                    self.exec_stack.push(val);
                }
                // Inst::IF => {
                //     self.stack_pos += 1;
                //     unimplemented!();
                // }
                // Inst::DEF_FUNC => {
                //     self.stack_pos += 1;
                //     unimplemented!();
                // }
                // Inst::CALL_FUNC => {
                //     self.stack_pos += 1;
                //     unimplemented!();
                // }
                // Inst::DEF_CLASS => {
                //     self.stack_pos += 1;
                //     unimplemented!();
                // }
                // Inst::BLOCK_DECL => {
                //     self.stack_pos += 1;
                //     unimplemented!();
                // }
                // Inst::GET_INSTANCE_VAL => {
                //     self.stack_pos += 1;
                //     unimplemented!();
                // }
                // Inst::GET_CLASS_VAL => {
                //     self.stack_pos += 1;
                //     unimplemented!();
                // }
                // Inst::GET_GLOBAL_VAL => {
                //     self.stack_pos += 1;
                //     unimplemented!();
                // }
                // Inst::GET_CONST_VAL => {
                //     self.stack_pos += 1;
                //     unimplemented!();
                // }
                // Inst::PARAM => {
                //     self.stack_pos += 1;
                //     unimplemented!();
                // }
                // Inst::FUNC_DECL => {
                //     self.stack_pos += 1;
                //     unimplemented!();
                // }
                // Inst::CLASS_DECL => {
                //     self.stack_pos += 1;
                //     unimplemented!();
                // }
                // Inst::SEND => {
                //     self.stack_pos += 1;
                //     let args = self.get_array();
                //     let body = self.exec_stack().value() as u8;
                //     let table = self.exec_stack().value() as u8;
                //     let id = self.exec_stack().ident();
                //     let info = self.get_method_info(id);
                //     let receiver = self.exec_stack();
                //     let f = self.push_env(receiver);
                //     self.push_env(receiver);

                //     match info {
                //         MethodInfo::RubyFunc {
                //             params,
                //             body,
                //             local_scope,
                //         } => {
                //             let args_len = args.len();
                //             self.scope_stack.push(local_scope);
                //             for (i, param) in params.iter().enumerate() {
                //                 let arg = if args_len > i {
                //                     args[i].clone()
                //                 } else {
                //                     Value::Nil
                //                 };
                //                 // self.lvar_table_as_mut().insert(param_id, arg);
                //             }
                //             self.pop_env_if_true(f);
                //             self.scope_stack.pop();
                //             let val = self.eval_seq(*body)?;
                //             Ok(val)
                //         }
                //         MethodInfo::BuiltinFunc { func, .. } => {
                //             // let node = &args.node; : body
                //             // let table = &args.table;
                //             let args = Args { body, args, table };
                //             self.pop_env_if_true(f);
                //             let val = func(self, receiver, args);
                //             Ok(val)
                //         }
                //     }
                // }
                Inst::ARRAY => {
                    self.stack_pos += 1;
                    let arr = self.get_array();
                    self.exec_stack.push(Value::Array(arr));
                }
                Inst::ARRAY_INDEX => {
                    self.stack_pos += 1;
                    let idx = self.exec_stack().value() as usize;
                    match self.exec_stack() {
                        Value::Array(arr) => {
                            let val = arr[idx].clone();
                            self.exec_stack.push(val);
                        }
                        _ => unreachable!(),
                    }
                }
                _ => unimplemented!(),
            }
        }
    }

    pub fn eval(&mut self) -> EvalResult {
        self.eval_seq()?;
        Ok(self.exec_stack())
    }

    // pub fn eval(&mut self) -> EvalResult {
    //     match self.eval_seq() {
    //         Ok(res) => Ok(res),
    //         Err(err) => {
    //             self.source_info.show_loc(&err.loc);
    //             match &err.kind {
    //                 RuntimeErrorKind::Name(s) => println!("NameError ({})", s),
    //                 RuntimeErrorKind::NoMethod(s) => println!("NoMethodError ({})", s),
    //                 RuntimeErrorKind::Unimplemented(s) => println!("Unimplemented ({})", s),
    //                 RuntimeErrorKind::Unreachable(s) => println!("Unreachable ({})", s),
    //             }
    //             Err(err)
    //         }
    //     }
    // }

    // pub fn eval(&mut self, node: &Node) -> EvalResult {
    //     match self.eval_node(node) {
    //         Ok(res) => Ok(res),
    //         Err(err) => {
    //             self.source_info.show_loc(&err.loc);
    //             match &err.kind {
    //                 RuntimeErrorKind::Name(s) => println!("NameError ({})", s),
    //                 RuntimeErrorKind::NoMethod(s) => println!("NoMethodError ({})", s),
    //                 RuntimeErrorKind::Unimplemented(s) => println!("Unimplemented ({})", s),
    //                 RuntimeErrorKind::Unreachable(s) => println!("Unreachable ({})", s),
    //             }
    //             Err(err)
    //         }
    //     }
    // }

    // pub fn eval_node(&mut self, node: &Node) -> EvalResult {
    //     match &node.kind {
    //         NodeKind::Number(num) => Ok(Value::FixNum(*num)),
    //         NodeKind::DecimalNumber(decimal_num) => Ok(Value::FixDecimalNum(*decimal_num)),
    //         NodeKind::String(s) => Ok(Value::String(s.to_string())),
    //         NodeKind::SelfValue => {
    //             let classref = self.class_stack.last().unwrap_or_else(|| {
    //                 self.clone().show_source(node);
    //                 panic!("Evaluator#eval: class stack is empty");
    //             });
    //             Ok(Value::Class(*classref))
    //         }
    //         NodeKind::Ident(id) => match self.lvar_table().get(id) {
    //             Some(val) => Ok(val.clone()),
    //             None => {
    //                 self.source_info.show_loc(&node.loc);
    //                 self.show_source(node);
    //                 panic!("undefined local variable.")
    //             }
    //         },
    //         NodeKind::Const(id) => match self.const_table.get(id) {
    //             Some(val) => Ok(val.clone()),
    //             None => {
    //                 self.source_info.show_loc(&node.loc());
    //                 self.clone().show_source(node);
    //                 panic!("NameError: uninitialized constant.");
    //             }
    //         },
    //         NodeKind::Vec(contents) => {
    //             let contents: Vec<Value> = contents
    //                 .iter()
    //                 .map(|x| self.eval_node(x).unwrap())
    //                 .collect();
    //             Ok(Value::Array(contents))
    //         }
    //         NodeKind::ArrayIndex(receiver, num) => match &receiver.as_ref().kind {
    //             NodeKind::Vec(vec) => {
    //                 let value = self.eval_node(&(*vec)[*num as usize])?;
    //                 Ok(value)
    //             }
    //             _ => {
    //                 self.clone().show_source(node);
    //                 unimplemented!();
    //             }
    //         },
    //         NodeKind::BinOp(op, lhs, rhs) => {
    //             match op {
    //                 BinOp::LAnd => {
    //                     let lhs_v = self.eval_node(lhs)?;
    //                     if let Value::Bool(b) = lhs_v {
    //                         if !b {
    //                             return Ok(Value::Bool(false));
    //                         }
    //                         let rhs_v = self.eval_node(rhs)?;
    //                         if let Value::Bool(b) = rhs_v {
    //                             return Ok(Value::Bool(b));
    //                         } else {
    //                             self.source_info.show_loc(&rhs.loc());
    //                             panic!("Expected bool.");
    //                         }
    //                     } else {
    //                         self.source_info.show_loc(&lhs.loc());
    //                         panic!("Expected bool.");
    //                     }
    //                 }
    //                 BinOp::LOr => {
    //                     let lhs_v = self.eval_node(lhs)?;
    //                     if let Value::Bool(b) = lhs_v {
    //                         if b {
    //                             return Ok(Value::Bool(true));
    //                         }
    //                         let rhs_v = self.eval_node(rhs)?;
    //                         if let Value::Bool(b) = rhs_v {
    //                             return Ok(Value::Bool(b));
    //                         } else {
    //                             self.source_info.show_loc(&rhs.loc());
    //                             panic!("Expected bool.");
    //                         }
    //                     } else {
    //                         self.source_info.show_loc(&lhs.loc());
    //                         panic!("Expected bool.");
    //                     }
    //                 }
    //                 _ => {}
    //             }
    //             let lhs = self.eval_node(lhs)?;
    //             let rhs = self.eval_node(rhs)?;
    //             match op {
    //                 BinOp::Add => self.eval_add(lhs, rhs),
    //                 BinOp::Sub => self.eval_sub(lhs, rhs),
    //                 BinOp::Mul => self.eval_mul(lhs, rhs),
    //                 BinOp::Div => self.eval_div(lhs, rhs),
    //                 BinOp::Eq => self.eval_eq(lhs, rhs),
    //                 BinOp::Ne => self.eval_neq(lhs, rhs),
    //                 BinOp::GE => self.eval_ge(lhs, rhs),
    //                 BinOp::GT => self.eval_gt(lhs, rhs),
    //                 BinOp::LE => self.eval_ge(rhs, lhs),
    //                 BinOp::LT => self.eval_gt(rhs, lhs),
    //                 _ => {
    //                     self.clone().show_source(node);
    //                     unimplemented!("{:?}", op);
    //                 }
    //             }
    //         }
    //         NodeKind::Assign(lhs, rhs) => match lhs.kind {
    //             NodeKind::Ident(id) => {
    //                 let rhs = self.eval_node(&rhs.clone())?;
    //                 match self.lvar_table().get_mut(&id) {
    //                     Some(val) => {
    //                         *val = rhs.clone();
    //                     }
    //                     None => {
    //                         self.lvar_table().insert(id, rhs.clone());
    //                     }
    //                 }
    //                 Ok(rhs)
    //             }
    //             NodeKind::InstanceVar(id) => {
    //                 let rhs = self.eval_node(&rhs.clone())?;
    //                 match self.env() {
    //                     Env::ClassRef(r) => {
    //                         let instance_var =
    //                             self.class_info_with_ref(r).instance_var.get_mut(&id);
    //                         match instance_var {
    //                             Some(val) => {
    //                                 *val = rhs.clone();
    //                             }
    //                             None => {
    //                                 self.class_info_with_ref(r)
    //                                     .instance_var
    //                                     .insert(id, rhs.clone());
    //                             }
    //                         }
    //                     }
    //                     Env::InstanceRef(r) => {
    //                         match self.instance_ref(r).instance_var.get_mut(&id) {
    //                             Some(val) => {
    //                                 *val = rhs.clone();
    //                             }
    //                             None => {
    //                                 self.instance_ref(r).instance_var.insert(id, rhs.clone());
    //                             }
    //                         }
    //                     }
    //                 };
    //                 Ok(rhs)
    //             }
    //             NodeKind::ClassVar(id) => {
    //                 let rhs = self.eval_node(&rhs.clone())?;
    //                 match self.env() {
    //                     Env::ClassRef(r) => {
    //                         let class_var = self.class_info_with_ref(r).class_var.get_mut(&id);
    //                         match class_var {
    //                             Some(val) => {
    //                                 *val = rhs;
    //                                 Ok(val.to_owned())
    //                             }
    //                             None => {
    //                                 self.class_info_with_ref(r)
    //                                     .class_var
    //                                     .insert(id, rhs.clone());
    //                                 Ok(rhs)
    //                             }
    //                         }
    //                     }
    //                     Env::InstanceRef(r) => {
    //                         let class_info = self.class_info_with_instance(r);
    //                         let class_var = class_info.class_var.get_mut(&id);
    //                         match class_var {
    //                             Some(val) => {
    //                                 *val = rhs;
    //                                 Ok(val.to_owned())
    //                             }
    //                             None => {
    //                                 self.clone().show_source(node);
    //                                 panic!("This class variable not defined. {:?}", id);
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //             NodeKind::GlobalIdent(id) => {
    //                 let rhs = self.eval_node(&rhs.clone())?;
    //                 match self.lvar_table().get_mut(&id) {
    //                     Some(val) => {
    //                         *val = rhs.clone();
    //                     }
    //                     None => {
    //                         self.lvar_table().insert(id, rhs.clone());
    //                     }
    //                 }
    //                 Ok(rhs)
    //             }
    //             _ => {
    //                 self.show_source(node);
    //                 unimplemented!();
    //             }
    //         },
    //         NodeKind::CompStmt(nodes) => {
    //             let mut val = Value::Nil;
    //             for node in nodes {
    //                 val = self.eval_node(node)?;
    //             }
    //             Ok(val)
    //         }
    //         NodeKind::If(cond_, then_, else_) => {
    //             let cond_val = self.eval_node(cond_)?;
    //             if self.val_to_bool(&cond_val) {
    //                 self.eval_node(then_)
    //             } else {
    //                 self.eval_node(else_)
    //             }
    //         }
    //         NodeKind::FuncDecl(id, params, body) => {
    //             let scope_stack = self.scope_stack.last().unwrap().to_owned();
    //             self.new_propagated_local_var_stack();
    //             self.env_info().method_table.insert(
    //                 *id,
    //                 MethodInfo::RubyFunc {
    //                     params: params.clone(),
    //                     body: body.clone(),
    //                     local_scope: scope_stack,
    //                 },
    //             );
    //             self.scope_stack.pop();
    //             Ok(Value::Nil)
    //         }
    //         NodeKind::ClassDecl(id, body, inheritence_class_id) => {
    //             let info = self.new_class_info(*id, *body.clone());
    //             let val = Value::Class(info);
    //             self.add_subclass(info, *inheritence_class_id);
    //             self.const_table.insert(*id, val);
    //             self.new_propagated_local_var_stack();
    //             self.class_stack.push(info);
    //             let class_ref = self.class_ref_with_id(*id);
    //             self.env.push(Env::ClassRef(class_ref));
    //             self.eval_node(body)?;
    //             self.env.pop().unwrap();
    //             self.class_stack.pop();
    //             self.scope_stack.pop();
    //             Ok(Value::Nil)
    //         }
    //         NodeKind::BlockDecl(body) => self.eval_node(body),
    //         NodeKind::Send(receiver, method, args) => {
    //             let id = match method.kind {
    //                 NodeKind::Ident(id) => id,
    //                 _ => {
    //                     self.show_source(node);
    //                     unimplemented!("method must be identifer.");
    //                 }
    //             };
    //             let receiver = self.eval_node(receiver)?;
    //             let f = self.push_env(receiver.clone());
    //             let args_val: Vec<Value> = args
    //                 .args
    //                 .iter()
    //                 .map(|x| self.eval_node(x).unwrap())
    //                 .collect();
    //             let info = self.get_method_info(id);
    //             match info {
    //                 MethodInfo::RubyFunc {
    //                     params,
    //                     body,
    //                     local_scope,
    //                 } => {
    //                     let args_value_len = args.args.len();
    //                     self.scope_stack.push(local_scope);
    //                     for (i, param) in params.0.iter() {
    //                     //for (i, param) in params.iter().enumerate() {
    //                         if let Node {
    //                             kind: NodeKind::Param(param_id),
    //                             ..
    //                         } = param.clone()
    //                         {
    //                             let arg = if args_value_len > i {
    //                                 args_val[i].clone()
    //                             } else {
    //                                 Value::Nil
    //                             };
    //                             self.lvar_table().insert(param_id, arg);
    //                         } else {
    //                             self.show_source(node);
    //                             unimplemented!("Illegal parameter.");
    //                         }
    //                     }
    //                     let val = self.eval_node(&body);
    //                     self.pop_env_if_true(f);
    //                     self.scope_stack.pop();
    //                     val
    //                 }
    //                 MethodInfo::BuiltinFunc { func, .. } => {
    //                     let node = &args.node;
    //                     let table = &args.table;
    //                     let args = Args {
    //                         node: node.to_owned(),
    //                         value: args_val,
    //                         table: table.to_owned(),
    //                     };
    //                     self.pop_env_if_true(f);
    //                     Ok(func(self, receiver, args))
    //                 }
    //             }
    //         }
    //         NodeKind::InstanceVar(id) => match self.env() {
    //             Env::InstanceRef(r) => Ok(self.instance_value(r, *id)),
    //             _ => {
    //                 self.show_source(node);
    //                 unimplemented!();
    //             }
    //         },
    //         NodeKind::ClassVar(id) => match self.env() {
    //             Env::ClassRef(r) => Ok(self.class_value(r, *id)),
    //             Env::InstanceRef(r) => Ok(self.class_value_with_instance(r, *id)),
    //         },
    //         _ => {
    //             self.show_source(node);
    //             unimplemented!("{:?}", node.kind);
    //         }
    //     }
    // }

    fn class_info_with_ref(&mut self, class_ref: ClassRef) -> &mut ClassInfo {
        self.class_table.table.get_mut(&class_ref).unwrap()
    }

    // fn class_info_with_id(&mut self, ident_id: IdentId) -> ClassInfo {
    //     for t in self.class_table.table.to_owned() {
    //         if t.1.id == ident_id {
    //             return t.1;
    //         }
    //     }
    //     unimplemented!()
    // }

    // fn class_ref_with_id(&mut self, ident_id: IdentId) -> ClassRef {
    //     for t in self.class_table.table.to_owned() {
    //         if t.1.id == ident_id {
    //             return t.0;
    //         }
    //     }
    //     unimplemented!()
    // }

    // fn class_info_with_instance(&mut self, instance_ref: InstanceRef) -> &mut ClassInfo {
    //     let class_ref = self.instance_ref(instance_ref).class_id;
    //     self.class_info_with_ref(class_ref)
    // }

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

    // fn add_subclass(&mut self, info: ClassRef, inheritence_class_id: Option<IdentId>) {
    //     if let Some(inheritence_class_id) = inheritence_class_id {
    //         let class = self.class_info_with_ref(info);
    //         class
    //             .subclass
    //             .insert(inheritence_class_id, ClassRef(*inheritence_class_id + 1));
    //     }
    // }

    // fn get_method_info(&mut self, id: IdentId) -> MethodInfo {
    //     for env in self.env.clone().iter_mut().rev() {
    //         let r = match env {
    //             Env::ClassRef(ClassRef(r)) => *self.class_info_with_ref(ClassRef(*r)).id,
    //             Env::InstanceRef(InstanceRef(r)) => {
    //                 *self.class_info_with_instance(InstanceRef(*r)).id
    //             }
    //         };
    //         let class_ref = self.class_info_with_id(IdentId(r)).clone();
    //         match class_ref.method_table.get(&id) {
    //             Some(info) => return info.to_owned(),
    //             None => {
    //                 for r in class_ref.subclass.values() {
    //                     if let Some(info) = self.class_info_with_ref(*r).method_table.get(&id) {
    //                         return info.to_owned();
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     unimplemented!("undefined function.");
    // }

    // fn push_env(&mut self, val: Value) -> bool {
    //     match val {
    //         Value::Class(r) => {
    //             self.env.push(Env::ClassRef(r));
    //             true
    //         }
    //         Value::Instance(r) => {
    //             self.env.push(Env::InstanceRef(r));
    //             true
    //         }
    //         _ => false,
    //     }
    // }

    // fn pop_env_if_true(&mut self, b: bool) {
    //     if b {
    //         self.env.pop().unwrap();
    //     }
    // }

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
    pub fn new_class(&mut self, id: IdentId, body: Node) -> ClassRef {
        let name = self.ident_table.get_name(id).clone();
        let class_ref = self.class_table.new_class(id, name, body);
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
    pub fn val_to_bool(&self, val: &Value) -> bool {
        match val {
            Value::Nil => false,
            Value::Bool(b) => *b,
            Value::FixNum(n) => {
                if n > &0 {
                    true
                } else {
                    false
                }
            }
            Value::String(_) => true,
            _ => unimplemented!(),
        }
    }

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

    pub fn val_to_i(&mut self, val: &Value) -> i64 {
        match val {
            Value::String(s) => match s.parse() {
                Ok(i) => i,
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }

    pub fn val_to_class(&mut self, val: &Value) -> Class {
        match val {
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
