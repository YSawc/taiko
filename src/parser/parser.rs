use crate::lexer::lexer::*;
use crate::node::node::*;
use crate::token::token::*;
use crate::util::annot::*;
use crate::util::util::*;
use crate::value::value::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Parser {
    tokens: Vec<Token>,
    pub source_info: SourceInfo,
    cursor: usize,
    pub ident_table: IdentifierTable,
}

pub type ParseError = Annot<ParseErrorKind>;

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
            ident_table: IdentifierTable::new(),
        }
    }

    fn skip_space(&mut self) {
        if self.tokens[self.cursor].kind == TokenKind::Space {
            self.cursor += 1;
        }
    }

    fn is_line_term(&self) -> bool {
        self.peek_no_skip_line_term().is_line_term()
    }

    fn loc(&self) -> Loc {
        self.tokens[self.cursor].loc()
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

    fn peek(&self) -> (&Token, Loc) {
        let mut c = self.cursor;
        loop {
            let tok = &self.tokens[c];
            if tok.is_eof() || (!tok.is_line_term() && !tok.is_space() && !tok.is_comment()) {
                return (tok, tok.loc);
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
        match &self.peek().0.kind {
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
            Err(self.error_unexpected(tok.loc()))
        }
    }

    fn expect_reserved(&mut self, expect: Reserved) -> Result<(), ParseError> {
        let tok = self.get().clone();
        let loc = self.loc();
        match &tok.kind {
            TokenKind::Reserved(reserved) => {
                if *reserved == expect {
                    Ok(())
                } else {
                    Err(self.error_unexpected(loc))
                }
            }
            _ => Err(self.error_unexpected(loc)),
        }
    }

    fn error_unexpected(&self, loc: Loc) -> ParseError {
        self.source_info.show_loc(&loc);
        ParseError::new(ParseErrorKind::UnexpectedToken, loc)
    }

    #[allow(unused)]
    fn error_eof(&self, loc: Loc) -> ParseError {
        self.source_info.show_loc(&loc);
        ParseError::new(ParseErrorKind::EOF, loc)
    }

    fn peek_no_skip_line_term(&self) -> &Token {
        &self.tokens[self.cursor]
    }

    fn get_if_punct(&mut self, expect: Punct) -> bool {
        match &self.peek().0.kind {
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
        let mut loc = self.loc();
        loop {
            let (tok, _) = self.peek();
            match tok.kind {
                TokenKind::Punct(Punct::Comment) => continue,
                TokenKind::EOF => break,
                TokenKind::Reserved(reserved) => match reserved {
                    Reserved::Else | Reserved::Elsif | Reserved::End => break,
                    _ => {}
                },
                _ => {}
            };
            let node = self.parse_expr()?;
            nodes.push(node);
            if !self.get_if_term() {
                break;
            }
        }

        if let Some(node) = nodes.last() {
            loc = loc.merge(node.loc())
        }

        Ok(Node {
            kind: NodeKind::CompStmt(nodes),
            loc,
        })
    }

    pub fn parse_if_then(&mut self) -> Result<Node, ParseError> {
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
        let loc = cond.loc().merge(else_.loc());
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
        let lhs = self.parse_arg_logical_or()?;
        if self.is_line_term() {
            return Ok(lhs);
        }
        if self.get_if_punct(Punct::Assign) {
            let rhs = self.parse_arg()?;
            Ok(Node::new_assign(lhs, rhs))
        } else {
            Ok(lhs)
        }
    }

    fn parse_arg_logical_or(&mut self) -> Result<Node, ParseError> {
        let lhs = self.parse_arg_logical_and()?;
        if self.is_line_term() {
            return Ok(lhs);
        }
        if self.get_if_punct(Punct::LAnd) {
            let rhs = self.parse_arg_logical_or()?;
            Ok(Node::new_binop(BinOp::LAnd, lhs, rhs))
        } else {
            Ok(lhs)
        }
    }

    fn parse_arg_logical_and(&mut self) -> Result<Node, ParseError> {
        let lhs = self.parse_arg_eq()?;
        if self.is_line_term() {
            return Ok(lhs);
        }
        if self.get_if_punct(Punct::LAnd) {
            let rhs = self.parse_arg_logical_and()?;
            Ok(Node::new_binop(BinOp::LAnd, lhs, rhs))
        } else {
            Ok(lhs)
        }
    }

    fn parse_arg_eq(&mut self) -> Result<Node, ParseError> {
        let lhs = self.parse_arg_comp()?;
        if self.is_line_term() {
            return Ok(lhs);
        }
        if self.get_if_punct(Punct::Eq) {
            let rhs = self.parse_arg_eq()?;
            Ok(Node::new_binop(BinOp::Eq, lhs, rhs))
        } else if self.get_if_punct(Punct::NE) {
            let rhs = self.parse_arg_eq()?;
            Ok(Node::new_binop(BinOp::Ne, lhs, rhs))
        } else {
            Ok(lhs)
        }
    }

    fn parse_arg_comp(&mut self) -> Result<Node, ParseError> {
        let lhs = self.parse_arg_add()?;
        if self.is_line_term() {
            return Ok(lhs);
        }
        if self.get_if_punct(Punct::GE) {
            let rhs = self.parse_arg_comp()?;
            Ok(Node::new_binop(BinOp::GE, lhs, rhs))
        } else if self.get_if_punct(Punct::GT) {
            let rhs = self.parse_arg_comp()?;
            Ok(Node::new_binop(BinOp::GT, lhs, rhs))
        } else if self.get_if_punct(Punct::LE) {
            let rhs = self.parse_arg_comp()?;
            Ok(Node::new_binop(BinOp::LE, lhs, rhs))
        } else if self.get_if_punct(Punct::LT) {
            let rhs = self.parse_arg_comp()?;
            Ok(Node::new_binop(BinOp::LT, lhs, rhs))
        } else {
            Ok(lhs)
        }
    }

    fn parse_arg_add(&mut self) -> Result<Node, ParseError> {
        let lhs = self.parse_arg_mul()?;
        if self.is_line_term() {
            return Ok(lhs);
        }
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

    fn parse_arg_mul(&mut self) -> Result<Node, ParseError> {
        let lhs = self.parse_unary_minus()?;
        if self.is_line_term() {
            return Ok(lhs);
        }
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

    fn parse_unary_minus(&mut self) -> Result<Node, ParseError> {
        let loc = self.loc();
        if self.get_if_punct(Punct::Minus) {
            let lhs = self.parse_primary()?;
            let loc = loc.merge(lhs.loc());
            let lhs = Node::new_binop(BinOp::Mul, lhs, Node::new_number(-1, loc));
            Ok(lhs)
        } else {
            let lhs = self.parse_primary()?;
            Ok(lhs)
        }
    }

    fn parse_primary(&mut self) -> Result<Node, ParseError> {
        let tok = self.get().clone();
        let loc = tok.loc();
        match &tok.kind {
            TokenKind::Ident(name) => {
                let id = self.ident_table.get_ident_id(name);
                if !self.get_if_punct(Punct::LParen) {
                    if name == "self" {
                        return Ok(Node::new(NodeKind::SelfValue, loc));
                    }
                    return Ok(Node::new_local_var(id, loc));
                }
                let mut args = vec![];
                if self.get_if_punct(Punct::RParen) {
                    return Ok(Node::new_send(id, args, loc));
                }
                loop {
                    args.push(self.parse_arg()?);
                    if !self.get_if_punct(Punct::Comma) {
                        break;
                    }
                }
                let end_loc = self.loc();
                if self.get_if_punct(Punct::RParen) {
                    Ok(Node::new_send(id, args, loc.merge(end_loc)))
                } else {
                    Err(self.error_unexpected(self.loc()))
                }
            }
            TokenKind::Const(name) => {
                let id = self.ident_table.get_ident_id(name);
                Ok(Node::new_const(id, loc))
            }
            TokenKind::NumLit(num) => Ok(Node::new_number(*num, loc)),
            TokenKind::StringLit(s) => Ok(Node::new_string(s.to_string(), loc)),
            TokenKind::Punct(Punct::LParen) => {
                let node = self.parse_comp_stmt()?;
                let tok = self.get().clone();
                if tok.kind == TokenKind::Punct(Punct::RParen) {
                    Ok(node)
                } else {
                    Err(self.error_unexpected(self.loc()))
                }
            }
            TokenKind::Reserved(Reserved::If) => {
                let node = self.parse_if_then()?;
                self.expect_reserved(Reserved::End)?;
                Ok(node)
            }
            TokenKind::Reserved(Reserved::Def) => {
                let node = self.parse_def()?;
                Ok(node)
            }
            TokenKind::Reserved(Reserved::Class) => {
                let node = self.parse_class()?;
                Ok(node)
            }
            TokenKind::EOF => Err(self.error_eof(loc)),
            _ => Err(self.error_unexpected(loc)),
        }
    }

    fn parse_class(&mut self) -> Result<Node, ParseError> {
        let loc = self.loc();
        let name = match &self.get().kind {
            TokenKind::Const(s) => s.clone(),
            _ => return Err(self.error_unexpected(loc)),
        };
        let id = self.ident_table.get_ident_id(&name);

        let body = self.parse_comp_stmt()?;
        self.expect_reserved(Reserved::End)?;

        Ok(Node::new_class_decl(id, body))
    }

    pub fn parse_then(&mut self) -> Result<(), ParseError> {
        if self.get_if_term() {
            return Ok(());
        }
        self.expect_reserved(Reserved::Then)?;
        Ok(())
    }

    fn parse_def(&mut self) -> Result<Node, ParseError> {
        let loc = self.loc();
        let name = match &self.get().kind {
            TokenKind::Ident(s) => s.clone(),
            _ => return Err(self.error_unexpected(loc)),
        };
        let id = self.ident_table.get_ident_id(&name);

        let args = self.parse_params()?;
        let body = self.parse_comp_stmt()?;
        self.expect_reserved(Reserved::End)?;

        Ok(Node::new_method_decl(id, args, body))
    }

    fn parse_params(&mut self) -> Result<Vec<Node>, ParseError> {
        if !self.get_if_punct(Punct::LParen) {
            return Ok(vec![]);
        }
        let mut args = vec![];
        if self.get_if_punct(Punct::RParen) {
            return Ok(args);
        }
        loop {
            let (arg, loc) = match self.get().clone() {
                Token {
                    kind: TokenKind::Ident(s),
                    loc,
                } => (s.clone(), loc),
                Token { loc, .. } => return Err(self.error_unexpected(loc)),
            };
            let id = self.ident_table.get_ident_id(&arg);
            args.push(Node::new(NodeKind::Param(id), loc));
            if !self.get_if_punct(Punct::Comma) {
                break;
            }
        }
        if self.get_if_punct(Punct::RParen) {
            Ok(args)
        } else {
            Err(self.error_unexpected(self.peek().1))
        }
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
