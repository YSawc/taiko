use crate::parser::parser::*;
use crate::value::value::*;

pub fn eval_node(node: &Node) -> Value {
    //println!("node.kind: {:?}", node.kind);
    match &node.kind {
        NodeKind::Number(num) => Value::FixNum(*num),
        NodeKind::BinOp(op, lhs, rhs) => {
            let lhs = eval_node(lhs);
            let rhs = eval_node(rhs);
            match op {
                BinOp::Add => eval_add(lhs, rhs),
                BinOp::Sub => eval_sub(lhs, rhs),
                BinOp::Mul => eval_mul(lhs, rhs),
                BinOp::Eq => eval_eq(lhs, rhs),
            }
        }
        NodeKind::CompStmt(nodes) => {
            let mut val = Value::Bool(false);
            for node in nodes {
                val = eval_node(node);
            }
            val
        }
        NodeKind::If(cond_, then_, else_) => {
            if eval_node(&cond_).to_bool() {
                eval_node(&then_)
            } else {
                eval_node(&else_)
            }
        }
        _ => unimplemented!(),
    }
}

fn eval_add(lhs: Value, rhs: Value) -> Value {
    match (lhs, rhs) {
        (Value::FixNum(lhs), Value::FixNum(rhs)) => Value::FixNum(lhs + rhs),
        (_, _) => unimplemented!(),
    }
}

fn eval_sub(lhs: Value, rhs: Value) -> Value {
    match (lhs, rhs) {
        (Value::FixNum(lhs), Value::FixNum(rhs)) => Value::FixNum(lhs - rhs),
        (_, _) => unimplemented!(),
    }
}

fn eval_mul(lhs: Value, rhs: Value) -> Value {
    match (lhs, rhs) {
        (Value::FixNum(lhs), Value::FixNum(rhs)) => Value::FixNum(lhs * rhs),
        (_, _) => unimplemented!(),
    }
}

fn eval_eq(lhs: Value, rhs: Value) -> Value {
    match (lhs, rhs) {
        (Value::FixNum(lhs), Value::FixNum(rhs)) => Value::Bool(lhs == rhs),
        (Value::Bool(lhs), Value::Bool(rhs)) => Value::Bool(lhs == rhs),
        (_, _) => unimplemented!(),
    }
}
