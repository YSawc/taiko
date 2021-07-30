use crate::node::node::*;
use crate::vm::inst::*;
use crate::vm::vm::*;

impl VM {
    pub fn gen(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::None => self.iseq.push(Inst::NIL),
            NodeKind::Number(num) => self.gen_comp_usize(*num),
            NodeKind::BinOp(op, lhs, rhs) => match op {
                BinOp::Add => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.iseq.push(Inst::ADD);
                }
                BinOp::Sub => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.iseq.push(Inst::SUB);
                }
                BinOp::Mul => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.iseq.push(Inst::MUL);
                }
                BinOp::Div => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.iseq.push(Inst::DIV)
                }
                BinOp::Eq => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.iseq.push(Inst::EQ)
                }
                BinOp::Ne => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.iseq.push(Inst::NE)
                }
                BinOp::GT => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.iseq.push(Inst::GT)
                }
                BinOp::GE => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.iseq.push(Inst::GE)
                }
                BinOp::LT => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.iseq.push(Inst::LT)
                }
                BinOp::LE => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.iseq.push(Inst::LE)
                }
                BinOp::LAnd => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.iseq.push(Inst::LAND)
                }
                BinOp::LOr => {
                    self.gen(lhs);
                    self.gen(rhs);
                    self.iseq.push(Inst::LOR)
                }
            },
            NodeKind::CompStmt(nodes) => {
                for node in nodes {
                    self.gen(&node)
                }
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
            // NodeKind::Send(receiver, method, args) => {
            //     let id = match method.kind {
            //         NodeKind::Ident(id) => id,
            //         _ => return Err(self.error_unimplemented(format!("Expected identifier."))),
            //     };
            //     for arg in args.iter().rev() {
            //         self.gen(arg)?;
            //     }
            //     self.gen(receiver)?;
            //     self.gen_send(id, args.len());
            // }
            _ => unimplemented!(),
        }
    }

    fn gen_comp_usize(&mut self, num: i64) {
        self.iseq.push(Inst::FIXNUM);
        self.iseq.push((num << 56) as u8);
        self.iseq.push((num << 48) as u8);
        self.iseq.push((num << 40) as u8);
        self.iseq.push((num << 32) as u8);
        self.iseq.push((num << 24) as u8);
        self.iseq.push((num << 16) as u8);
        self.iseq.push((num << 8) as u8);
        self.iseq.push(num as u8);
    }
}
