use crate::node::node::*;
use crate::util::util::*;
use crate::vm::inst::*;
use crate::vm::vm::*;

#[derive(Debug, Clone)]
pub struct Stack {
    pub iseq: Vec<u8>,
    pub ident_table: IdentifierTable,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            iseq: vec![],
            ident_table: IdentifierTable::default(),
        }
    }
}

impl VM {
    pub fn gen(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::None => self.stack.iseq.push(Inst::NIL),
            NodeKind::SelfValue => self.stack.iseq.push(Inst::SELF_VALUE),
            NodeKind::Number(num) => self.gen_comp_fixnum(*num),
            NodeKind::String(s) => {
                let id = self.stack.ident_table.get_ident_id(s);
                self.gen_comp_usize(*id);
                self.stack.iseq.push(Inst::STRING);
            }
            NodeKind::BinOp(op, lhs, rhs) => match op {
                BinOp::Add => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.stack.iseq.push(Inst::ADD);
                }
                BinOp::Sub => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.stack.iseq.push(Inst::SUB);
                }
                BinOp::Mul => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.stack.iseq.push(Inst::MUL);
                }
                BinOp::Div => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.stack.iseq.push(Inst::DIV)
                }
                BinOp::Eq => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.stack.iseq.push(Inst::EQ)
                }
                BinOp::Ne => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.stack.iseq.push(Inst::NE)
                }
                BinOp::GT => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.stack.iseq.push(Inst::GT)
                }
                BinOp::GE => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.stack.iseq.push(Inst::GE)
                }
                BinOp::LT => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.stack.iseq.push(Inst::LT)
                }
                BinOp::LE => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.stack.iseq.push(Inst::LE)
                }
                BinOp::LAnd => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.stack.iseq.push(Inst::LAND)
                }
                BinOp::LOr => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.stack.iseq.push(Inst::LOR)
                }
            },
            NodeKind::CompStmt(nodes) => self.gen_nodes(nodes.to_vec()),
            NodeKind::Array(nodes) => {
                self.gen_nodes(nodes.to_vec());
                let len = nodes.len();
                self.gen_comp_usize(len);
                self.stack.iseq.push(Inst::ARRAY)
            }
            NodeKind::ArrayIndex(nodes, num) => {
                self.gen(nodes);
                self.gen_comp_fixnum(*num);
                self.stack.iseq.push(Inst::ARRAY_INDEX);
            }
            // NodeKind::If(cond_, then_, else_) => {
            //     self.gen(&cond_)?;
            //     let src1 = self.gen_jmp_if_false();
            //     self.gen(&then_)?;
            //     let src2 = self.gen_jmp();
            //     self.write_disp_from_cur(src1);
            //     self.gen(&else_)?;
            //     self.write_disp_from_cur(src2);
            // }
            // NodeKind::For(id, iter, body) => {
            //     let id = match id.kind {
            //         NodeKind::Ident(id) => id,
            //         _ => return Err(self.error_nomethod("Expected an identifier.")),
            //     };
            //     let (start, end) = match &iter.kind {
            //         NodeKind::Range(start, end) => (start, end),
            //         _ => return Err(self.error_nomethod("Expected Range.")),
            //     };
            //     self.gen(start)?;
            //     self.gen_set_local(id);
            //     let p = self.current();
            //     self.gen(end)?;
            //     self.gen_get_local(id);
            //     self.iseq.push(Inst::GE);
            //     let src = self.gen_jmp_if_false();
            //     self.gen(body)?;
            //     self.gen_get_local(id);
            //     self.gen_fixnum(1);
            //     self.iseq.push(Inst::ADD);
            //     self.gen_set_local(id);

            //     self.gen_jmp_back(p);
            //     self.write_disp_from_cur(src);
            // }
            // NodeKind::Assign(lhs, rhs) => {
            //     self.gen(rhs)?;
            //     match lhs.kind {
            //         NodeKind::Ident(id) => {
            //             self.gen_set_local(id);
            //         }
            //         _ => (),
            //     }
            // }
            NodeKind::Send(receiver, method, args) => {
                self.gen(receiver);
                let id = match method.kind {
                    NodeKind::Ident(id) => id,
                    _ => unimplemented!(),
                };
                self.gen_comp_usize(*id);
                self.gen(&args.table);
                self.gen(&args.node);
                self.gen_nodes_with_len(args.args.to_owned());
                self.stack.iseq.push(Inst::SEND)
            }
            _ => {
                println!("&node.kind: {:?}", &node.kind);
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
        self.stack.iseq.push(Inst::FIXNUM);
        self.stack.iseq.push((num >> 56) as u8);
        self.stack.iseq.push((num >> 48) as u8);
        self.stack.iseq.push((num >> 40) as u8);
        self.stack.iseq.push((num >> 32) as u8);
        self.stack.iseq.push((num >> 24) as u8);
        self.stack.iseq.push((num >> 16) as u8);
        self.stack.iseq.push((num >> 8) as u8);
        self.stack.iseq.push(num as u8);
    }

    fn gen_comp_usize(&mut self, num: usize) {
        let num = num as i64;
        self.gen_comp_fixnum(num);
    }
}
