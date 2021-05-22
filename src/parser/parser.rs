use crate::lexer::lexer::*;
use crate::token::token::*;
use crate::util::annot::*;
use crate::util::util::*;
use crate::value::value::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub kind: NodeKind,
    pub loc: Loc,
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            NodeKind::BinOp(op, lhs, rhs) => write!(f, "[{:?} ( {}, {}  )]", op, lhs, rhs),
            _ => write!(f, "[{:?}]", self.kind),
        }
    }
}

impl Node {
    fn new_number(num: i64, loc: Loc) -> Self {
        Node {
            kind: NodeKind::Number(num),
            loc,
        }
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
    If(Box<Node>, Box<Node>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Eq,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parser {
    tokens: Vec<Token>,
    pub source_info: SourceInfo,
    cursor: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken,
    EOF,
}

impl Parser {
    pub fn new(result: LexerResult) -> Self {
        Parser {
            tokens: result.tokens,
            cursor: 0,
            source_info: result.source_info,
        }
    }

    fn skip_space(&mut self) {
        loop {
            if self.tokens[self.cursor].is_space() {
                self.cursor += 1;
            } else {
                break;
            }
        }
    }

    pub fn get(&mut self) -> &Token {
        loop {
            let token = &self.tokens[self.cursor];
            if token.is_eof() {
                return token;
            }
            self.cursor += 1;
            if !token.is_line_term() && !token.is_space() {
                return token;
            }
        }
    }

    fn peek(&mut self) -> &Token {
        let mut c = self.cursor;
        loop {
            let tok = &self.tokens[c];
            if tok.is_eof() || (!tok.is_line_term() && !tok.is_space()) {
                return tok;
            } else {
                c += 1;
            }
        }
    }

    #[allow(unused)]
    fn unget(&mut self) {
        self.cursor -= 1;
    }

    fn get_no_skip_line_term(&mut self) -> &Token {
        self.skip_space();
        let token = &self.tokens[self.cursor];
        if !token.is_eof() {
            self.cursor += 1;
        }

        token
    }

    #[allow(unused)]
    fn expect_term(&mut self) -> Result<(), ParseError> {
        let tok = self.get_no_skip_line_term().clone();
        if tok.is_term() {
            Ok(())
        } else {
            Err(self.error_unexpected(&tok))
        }
    }

    fn expect_reserved(&mut self, expect: Reserved) -> Result<(), ParseError> {
        let tok = self.get().clone();
        match tok.kind {
            TokenKind::Reserved(reserved) => {
                if reserved == expect {
                    Ok(())
                } else {
                    Err(self.error_unexpected(&tok))
                }
            }
            _ => Err(self.error_unexpected(&tok)),
        }
    }

    fn error_unexpected(&self, tok: &Token) -> ParseError {
        self.source_info.show_loc(&tok.loc);
        ParseError::UnexpectedToken
    }

    fn peek_no_skip_line_term(&mut self) -> &Token {
        &self.tokens[self.cursor]
    }

    fn get_if_term(&mut self) -> bool {
        if self.peek_no_skip_line_term().is_term() {
            self.get_no_skip_line_term();
            true
        } else {
            false
        }
    }

    fn peek_non_space(&mut self) -> &Token {
        self.skip_space();
        &self.tokens[self.cursor]
    }

    pub fn parse_comp_stmt(&mut self) -> Result<Node, ParseError> {
        let mut nodes = vec![];
        loop {
            let tok = self.peek();
            match tok.kind {
                TokenKind::EOF => break,
                TokenKind::Reserved(reserved) => match reserved {
                    Reserved::Else | Reserved::Elsif | Reserved::End => break,
                    _ => {}
                },
                _ => {}
            };
            nodes.push(self.parse_expr()?);
            if !self.get_if_term() {
                break;
            }
        }
        let mut loc;
        if nodes.is_empty() {
            loc = Loc::new(0, 0);
        } else {
            loc = nodes[0].loc;
            for node in &nodes {
                loc = loc.merge(node.loc);
            }
        }

        Ok(Node {
            kind: NodeKind::CompStmt(nodes),
            loc,
        })
    }

    pub fn parse_expr(&mut self) -> Result<Node, ParseError> {
        self.parse_arg_comp()
    }

    pub fn parse_arg_comp(&mut self) -> Result<Node, ParseError> {
        let lhs = self.parse_arg_add()?;
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::Punct(Punct::Equal) => {
                self.get();
                let rhs = self.parse_arg_comp()?;
                let loc = lhs.loc.merge(rhs.loc);
                Ok(Node {
                    kind: NodeKind::BinOp(BinOp::Eq, Box::new(lhs), Box::new(rhs)),
                    loc,
                })
            }
            _ => Ok(lhs),
        }
    }

    fn parse_arg_add(&mut self) -> Result<Node, ParseError> {
        let lhs = self.parse_arg_mul()?;
        let tok = self.peek_non_space().clone();
        match &tok.kind {
            TokenKind::Punct(ref punct) => match punct {
                Punct::Plus => {
                    self.get();
                    let rhs = self.parse_arg_add()?;
                    let loc = lhs.loc.merge(rhs.loc);
                    Ok(Node {
                        kind: NodeKind::BinOp(BinOp::Add, Box::new(lhs), Box::new(rhs)),
                        loc,
                    })
                }
                Punct::Minus => {
                    self.get();
                    let rhs = self.parse_arg_add()?;
                    let loc = lhs.loc.merge(rhs.loc);
                    Ok(Node {
                        kind: NodeKind::BinOp(BinOp::Sub, Box::new(lhs), Box::new(rhs)),
                        loc,
                    })
                }
                _ => Ok(lhs),
            },
            _ => Ok(lhs),
        }
    }

    pub fn parse_arg_mul(&mut self) -> Result<Node, ParseError> {
        let lhs = self.parse_primary()?;
        self.skip_space();
        let tok = self.peek_non_space().clone();
        match &tok.kind {
            TokenKind::Punct(Punct::Mul) => {
                self.get();
                let rhs = self.parse_arg_mul()?;
                let loc = lhs.loc.merge(rhs.loc);
                Ok(Node {
                    kind: NodeKind::BinOp(BinOp::Mul, Box::new(lhs), Box::new(rhs)),
                    loc,
                })
            }
            _ => Ok(lhs),
        }
    }

    fn parse_primary(&mut self) -> Result<Node, ParseError> {
        let tok = self.get().clone();
        match &tok.kind {
            TokenKind::NumLit(num) => Ok(Node::new_number(*num, tok.loc)),
            TokenKind::Punct(Punct::LParen) => {
                let node = self.parse_expr()?;
                let tok = self.get().clone();
                if tok.kind == TokenKind::Punct(Punct::RParen) {
                    Ok(node)
                } else {
                    Err(self.error_unexpected(&tok))
                }
            }
            TokenKind::Reserved(Reserved::If) => {
                let cond = self.parse_expr()?;
                println!("if cond {}", cond);
                self.parse_then()?;
                let then = self.parse_comp_stmt()?;
                println!("if then {}", then);
                self.expect_reserved(Reserved::End)?;
                let loc = tok.loc.merge(then.loc);
                Ok(Node {
                    kind: NodeKind::If(Box::new(cond), Box::new(then)),
                    loc,
                })
            }
            TokenKind::EOF => Err(ParseError::EOF),
            _ => {
                self.source_info.show_loc(&tok.loc);
                unimplemented!("{:?}, loc: {:?}", tok.kind, tok.loc)
            }
        }
    }

    pub fn parse_then(&mut self) -> Result<(), ParseError> {
        if self.get_if_term() {
            return Ok(());
        }
        self.expect_reserved(Reserved::Then)?;
        Ok(())
    }

    pub fn eval_node(node: &Node) -> Value {
        match &node.kind {
            NodeKind::Number(num) => Value::FixNum(*num),
            NodeKind::Add(lhs, rhs) => {
                let lhs = Parser::eval_node(lhs);
                let rhs = Parser::eval_node(rhs);
                match (lhs, rhs) {
                    (Value::FixNum(lhs), Value::FixNum(rhs)) => Value::FixNum(lhs + rhs),
                    (_, _) => unimplemented!(),
                }
            }
            NodeKind::Sub(lhs, rhs) => {
                let lhs = Parser::eval_node(lhs);
                let rhs = Parser::eval_node(rhs);
                match (lhs, rhs) {
                    (Value::FixNum(lhs), Value::FixNum(rhs)) => Value::FixNum(lhs - rhs),
                    (_, _) => unimplemented!(),
                }
            }
            NodeKind::Mul(lhs, rhs) => {
                let lhs = Parser::eval_node(lhs);
                let rhs = Parser::eval_node(rhs);
                match (lhs, rhs) {
                    (Value::FixNum(lhs), Value::FixNum(rhs)) => Value::FixNum(lhs * rhs),
                    (_, _) => unimplemented!(),
                }
            }
            _ => unimplemented!(),
        }
    }
}
