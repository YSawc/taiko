use crate::lexer::lexer::*;
use crate::node::node::*;
use crate::token::token::*;
use crate::util::annot::*;
use crate::util::util::*;
use crate::value::value::*;
use rustc_hash::FxHashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Parser {
    tokens: Vec<Token>,
    pub source_info: SourceInfo,
    cursor: usize,
    pub ident_table: IdentifierTable,
    ident_id: usize,
}

pub type ParseError = Annot<ParseErrorKind>;

pub type IdentifierTable = FxHashMap<String, usize>;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    UnexpectedToken,
    EOF,
}

impl Parser {
    pub fn new(result: LexerResult) -> Self {
        Parser {
            tokens: result.tokens,
            cursor: 0,
            source_info: result.source_info,
            ident_table: FxHashMap::default(),
            ident_id: 0,
        }
    }

    fn skip_space(&mut self) {
        if self.tokens[self.cursor].kind == TokenKind::Space {
            self.cursor += 1;
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

    fn peek(&self) -> &Token {
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

    fn get_if_reserved(&mut self, expect: Reserved) -> bool {
        match &self.peek().kind {
            TokenKind::Reserved(reserved) => {
                if *reserved == expect {
                    self.get();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
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

    fn error_unexpected<T>(&self, annot: &Annot<T>) -> ParseError {
        self.source_info.show_loc(&annot.loc);
        ParseError::new(ParseErrorKind::UnexpectedToken, annot.loc)
    }

    fn peek_no_skip_line_term(&self) -> &Token {
        &self.tokens[self.cursor]
    }

    fn get_if_punct(&mut self, expect: Punct) -> bool {
        match &self.peek().kind {
            TokenKind::Punct(punct) => {
                if *punct == expect {
                    self.get();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
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

    pub fn parse_program(&mut self) -> Result<Node, ParseError> {
        self.parse_comp_stmt()
    }

    pub fn parse_comp_stmt(&mut self) -> Result<Node, ParseError> {
        let mut nodes = vec![];
        let loc = self.peek().loc;
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
            let node = self.parse_expr()?;
            loc.merge(node.loc);
            nodes.push(node);
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

    pub fn parse_if_then(&mut self) -> Result<Node, ParseError> {
        let loc = self.peek().loc;
        let cond = self.parse_expr()?;
        println!("if cond {}", cond);
        self.parse_then()?;
        let then_ = self.parse_comp_stmt()?;
        println!("if then {}", then_);
        let mut else_ = Node::new_comp_stmt();
        println!("if else_ {}", else_);
        if self.get_if_reserved(Reserved::Elsif) {
            else_ = self.parse_if_then()?;
        } else if self.get_if_reserved(Reserved::Else) {
            else_ = self.parse_comp_stmt()?;
        }
        let loc = loc.merge(then_.loc);
        Ok(Node::new(
            NodeKind::If(Box::new(cond), Box::new(then_), Box::new(else_)),
            loc,
        ))
    }

    pub fn parse_expr(&mut self) -> Result<Node, ParseError> {
        self.parse_arg()
    }

    fn parse_arg(&mut self) -> Result<Node, ParseError> {
        self.parse_arg_assign()
    }

    fn parse_arg_assign(&mut self) -> Result<Node, ParseError> {
        let lhs = self.parse_arg_comp()?;
        if self.get_if_punct(Punct::Assign) {
            let rhs = self.parse_arg()?;
            Ok(Node::new_assign(lhs, rhs))
        } else {
            Ok(lhs)
        }
    }

    pub fn parse_arg_comp(&mut self) -> Result<Node, ParseError> {
        let lhs = self.parse_arg_add()?;
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::Punct(Punct::Equal) => {
                self.get();
                let rhs = self.parse_arg_comp()?;
                let loc = lhs.loc.merge(rhs.loc);
                Ok(Node::new(
                    NodeKind::BinOp(BinOp::Eq, Box::new(lhs), Box::new(rhs)),
                    loc,
                ))
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
                    Ok(Node::new(
                        NodeKind::BinOp(BinOp::Add, Box::new(lhs), Box::new(rhs)),
                        loc,
                    ))
                }
                Punct::Minus => {
                    self.get();
                    let rhs = self.parse_arg_add()?;
                    let loc = lhs.loc.merge(rhs.loc);
                    Ok(Node::new(
                        NodeKind::BinOp(BinOp::Sub, Box::new(lhs), Box::new(rhs)),
                        loc,
                    ))
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
                Ok(Node::new(
                    NodeKind::BinOp(BinOp::Mul, Box::new(lhs), Box::new(rhs)),
                    loc,
                ))
            }
            TokenKind::Punct(Punct::Div) => {
                self.get();
                let rhs = self.parse_arg_mul()?;
                let loc = lhs.loc.merge(rhs.loc);
                Ok(Node::new(
                    NodeKind::BinOp(BinOp::Div, Box::new(lhs), Box::new(rhs)),
                    loc,
                ))
            }
            _ => Ok(lhs),
        }
    }

    fn get_local_var_id(&mut self, name: &str) -> usize {
        match self.ident_table.get(name) {
            Some(id) => *id,
            None => {
                let id = self.ident_id;
                self.ident_table.insert(name.to_string(), id);
                self.ident_id += 1;
                id
            }
        }
    }

    fn parse_primary(&mut self) -> Result<Node, ParseError> {
        let tok = self.get().clone();
        match &tok.kind {
            TokenKind::Ident(name) => {
                let id = self.get_local_var_id(name);
                Ok(Node::new_local_var(id, tok.loc))
            }
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
                let node = self.parse_if_then();
                self.expect_reserved(Reserved::End)?;
                node
            }
            TokenKind::EOF => Err(ParseError::new(ParseErrorKind::EOF, tok.loc)),
            _ => Err(self.error_unexpected(&tok)),
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
            NodeKind::Div(lhs, rhs) => {
                let lhs = Parser::eval_node(lhs);
                let rhs = Parser::eval_node(rhs);
                match (lhs, rhs) {
                    (Value::FixNum(lhs), Value::FixNum(rhs)) => Value::FixNum(lhs / rhs),
                    (_, _) => unimplemented!(),
                }
            }

            _ => unimplemented!(),
        }
    }
}
