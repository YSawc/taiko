use crate::parser::parser::*;
use crate::util::annot::*;
use crate::util::util::*;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    None,
    SelfValue,
    Number(i64),
    DecimalNumber(f64),
    String(String),
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    Assign(Box<Node>, Box<Node>),
    BinOp(BinOp, Box<Node>, Box<Node>),
    CompStmt(Vec<Node>),
    If(Box<Node>, Box<Node>, Box<Node>),
    Ident(IdentId),
    InstanceVar(IdentId),
    ClassVar(IdentId),
    GlobalIdent(IdentId),
    Const(IdentId),
    Param(IdentId),
    FuncDecl(IdentId, NodeVec, Box<Node>),
    ClassDecl(IdentId, Box<Node>),
    BlockDecl(Box<Node>),
    Send(Box<Node>, Box<Node>, Box<ParsedArgs>),
    Table(Box<Node>),
    Vec(Vec<Node>),
    ArrayIndex(Box<Node>, i64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    GT,
    GE,
    LT,
    LE,
    LAnd,
    LOr,
}

pub type Node = Annot<NodeKind>;
pub type NodeVec = Vec<Node>;

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            NodeKind::BinOp(op, lhs, rhs) => write!(f, "[{:?} ( {}, {}  )]", op, lhs, rhs),
            NodeKind::Ident(id) => write!(f, "(LocalVar {:?})", id),
            NodeKind::GlobalIdent(id) => write!(f, "(GlobalVar {:?})", id),
            NodeKind::Send(receiver, method_name, args) => {
                write!(f, "[ Send [{}]: [{}] ", receiver, method_name)?;
                for node in &args.args {
                    write!(f, "({:?})", node)?;
                }
                write!(f, "{}", args.node)?;
                write!(f, "]")?;
                Ok(())
            }
            NodeKind::CompStmt(nodes) => {
                write!(f, "[ CompStmt ")?;
                for node in nodes {
                    write!(f, "({}) ", node)?;
                }
                write!(f, "]")?;
                Ok(())
            }
            NodeKind::FuncDecl(id, args, body) => {
                write!(f, "[ FuncDecl {:?}: PARAM(", id)?;
                for arg in args {
                    write!(f, "({}) ", arg)?;
                }
                write!(f, ") BODY({})]", body)?;
                Ok(())
            }
            NodeKind::If(cond_, then_, else_) => {
                write!(f, "[ If COND({}) THEN({}) ELSE({}) ]", cond_, then_, else_)
            }
            NodeKind::BlockDecl(body) => {
                write!(f, "[{}]", body)
            }
            _ => write!(f, "[{:?}]", self.kind),
        }
    }
}
impl Node {
    pub fn new_none() -> Self {
        Node::new(NodeKind::None, Loc(0, 0))
    }

    pub fn new_number(num: i64, loc: Loc) -> Self {
        Node::new(NodeKind::Number(num), loc)
    }

    pub fn new_decimal_number(decimal_num: f64, loc: Loc) -> Self {
        Node::new(NodeKind::DecimalNumber(decimal_num), loc)
    }

    pub fn new_string(s: String, loc: Loc) -> Self {
        Node::new(NodeKind::String(s), loc)
    }

    pub fn new_comp_stmt() -> Self {
        Node::new(NodeKind::CompStmt(vec![]), Loc(0, 0))
    }

    pub fn new_binop(op: BinOp, lhs: Node, rhs: Node) -> Self {
        let loc = (lhs.loc()).merge(rhs.loc());
        let kind = NodeKind::BinOp(op, Box::new(lhs), Box::new(rhs));
        Node::new(kind, loc)
    }

    pub fn new_identifier(id: IdentId, loc: Loc) -> Self {
        Node::new(NodeKind::Ident(id), loc)
    }

    pub fn new_instance_var(id: IdentId, loc: Loc) -> Self {
        Node::new(NodeKind::InstanceVar(id), loc)
    }

    pub fn new_class_var(id: IdentId, loc: Loc) -> Self {
        Node::new(NodeKind::ClassVar(id), loc)
    }

    pub fn new_global_identifier(id: IdentId, loc: Loc) -> Self {
        Node::new(NodeKind::GlobalIdent(id), loc)
    }

    pub fn new_assign(lhs: Node, rhs: Node) -> Self {
        let loc_merge = lhs.loc.merge(rhs.loc);
        let loc = Loc::new(loc_merge);
        Node::new(NodeKind::Assign(Box::new(lhs), Box::new(rhs)), loc)
    }

    pub fn new_method_decl(id: IdentId, params: Vec<Node>, body: Node) -> Self {
        let loc = Loc::new(body.loc());
        Node::new(NodeKind::FuncDecl(id, params, Box::new(body)), loc)
    }

    pub fn new_class_decl(id: IdentId, body: Node) -> Self {
        let loc = Loc::new(body.loc());
        Node::new(NodeKind::ClassDecl(id, Box::new(body)), loc)
    }

    pub fn new_block_decl(body: Node) -> Self {
        let loc = Loc::new(body.loc());
        Node::new(NodeKind::BlockDecl(Box::new(body)), loc)
    }

    pub fn new_const(id: IdentId, loc: Loc) -> Self {
        Node::new(NodeKind::Const(id), loc)
    }

    pub fn new_send(receiver: Node, method_name: Node, args: ParsedArgs, loc: Loc) -> Self {
        Node::new(
            NodeKind::Send(Box::new(receiver), Box::new(method_name), Box::new(args)),
            loc,
        )
    }

    pub fn new_vec(contents: Vec<Node>, loc: Loc) -> Self {
        Node::new(NodeKind::Vec(Box::new(contents).to_vec()), loc)
    }

    pub fn new_array_index(receiver: Node, num: i64, loc: Loc) -> Self {
        Node::new(NodeKind::ArrayIndex(Box::new(receiver), num), loc)
    }

    pub fn new_table(table: Node) -> Self {
        let loc = Loc::new(table.loc());
        Node::new(NodeKind::Table(Box::new(table)), loc)
    }
}

impl Node {
    pub fn pick_number(&self) -> i64 {
        match self.kind {
            NodeKind::Number(i) => i,
            _ => unimplemented!(),
        }
    }
}
