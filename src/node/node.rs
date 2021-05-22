use crate::util::annot::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub kind: NodeKind,
    pub loc: Loc,
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            NodeKind::BinOp(op, lhs, rhs) => write!(f, "[{:?} ( {}, {}  )]", op, lhs, rhs),
            NodeKind::CompStmt(nodes) => write!(f, "[{:?}]", nodes),
            _ => write!(f, "[{:?}]", self.kind),
        }
    }
}

impl Node {
    pub fn new(kind: NodeKind, loc: Loc) -> Self {
        Node {
            kind,
            loc
        }

    }

    pub fn new_number(num: i64, loc: Loc) -> Self {
        Node {
            kind: NodeKind::Number(num),
            loc,
        }
    }

    pub fn new_comp_stmt() -> Self {
        Node {
            kind: NodeKind::CompStmt(vec![]),
            loc: Loc(0, 0),
        }
    }

    pub fn new_binop(op: BinOp, lhs: Node, rhs: Node, loc: Loc) -> Self {
        let kind = NodeKind::BinOp(op, Box::new(lhs), Box::new(rhs));
        Node::new(kind, loc)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Number(i64),
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Assign(Box<Node>, Box<Node>),
    BinOp(BinOp, Box<Node>, Box<Node>),
    CompStmt(Vec<Node>),
    If(Box<Node>, Box<Node>, Box<Node>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Eq,
}
