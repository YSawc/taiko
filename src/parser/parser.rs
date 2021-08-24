use crate::lexer::lexer::*;
use crate::node::node::*;
use crate::token::token::*;
use crate::util::annot::*;
use crate::util::util::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Parser {
    pub lexer: Lexer,
    tokens: Vec<Token>,
    cursor: usize,
    block_context_stack: Vec<BlockContext>,
    line_context_stack: Vec<LineContext>,
    pub ident_table: IdentifierTable,
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
enum Literal {
    String,
    Number,
    Array,
}

#[derive(Debug, Clone, PartialEq)]
enum BlockContext {
    Class,
    Method,
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
enum LineContext {
    Literal(Literal),
    Class,
    Method,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedArgs {
    pub table: Node,
    pub node: Node,
    pub args: Vec<Node>,
}

impl ParsedArgs {
    pub fn new() -> Self {
        Self {
            table: Node::new_none(),
            node: Node::new_none(),
            args: vec![],
        }
    }
}

impl Parser {
    pub fn is_first_line_context(&self) -> bool {
        self.line_context_stack.last().is_none()
    }

    pub fn expect_first_line_context(&self) -> Result<(), ParseError> {
        match self.line_context_stack.last().is_none() {
            true => Ok(()),
            false => {
                let error_loc = self.tokens[self.cursor - 3].loc;
                self.lexer.source_info.show_loc(&error_loc);
                Err(ParseError::new(
                    ParseErrorKind::LiteralBeforeDefinition,
                    error_loc,
                ))
            }
        }
    }

    fn expect_line_context_literal(
        &mut self,
        expected_context: LineContext,
    ) -> Result<(), ParseError> {
        let got_literal = self.line_context_stack.last().unwrap();
        if *got_literal == expected_context {
            self.line_context_stack.pop().unwrap();
            Ok(())
        } else {
            panic!("expect {:?}, but god {:?}", expected_context, got_literal)
        }
    }

    pub fn reset_line_context(&mut self) {
        self.line_context_stack = vec![];
    }

    pub fn is_out_of_method_block_context(&self) -> bool {
        !self.block_context_stack.contains(&BlockContext::Method)
    }

    pub fn expect_out_of_method_block_context(&self) -> Result<(), ParseError> {
        match self.is_out_of_method_block_context() {
            true => Ok(()),
            false => {
                let error_loc = self.tokens[self.cursor - 6].loc;
                self.lexer.source_info.show_loc(&error_loc);
                Err(ParseError::new(
                    ParseErrorKind::InnerClassDefinitionInMethodDefinition,
                    error_loc,
                ))
            }
        }
    }
}

pub type ParseError = Annot<ParseErrorKind>;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    UnexpectedToken,
    LiteralBeforeDefinition,
    InnerClassDefinitionInMethodDefinition,
    EOF,
}

impl Parser {
    pub fn new() -> Self {
        let lexer = Lexer::new();
        Self {
            lexer,
            tokens: vec![],
            cursor: 0,
            block_context_stack: vec![],
            line_context_stack: vec![],
            ident_table: IdentifierTable::new(),
        }
    }

    fn skip_space(&mut self) {
        if self.tokens[self.cursor].kind == TokenKind::Space {
            self.cursor += 1;
        }
    }

    #[allow(unused)]
    fn is_line_term(&self) -> bool {
        self.peek_no_skip_line_term().is_line_term()
    }

    fn loc(&self) -> Loc {
        self.tokens[self.cursor].loc()
    }

    pub fn get(&mut self) -> Token {
        loop {
            let token = self.tokens[self.cursor].clone();
            if token.is_eof() {
                return token;
            }
            self.cursor += 1;
            if !token.is_line_term() && !token.is_space() && !token.is_comment() {
                return token;
            } else if token.is_line_term() {
                self.reset_line_context();
            }
        }
    }

    fn peek(&mut self) -> (Token, Loc) {
        let mut c = self.cursor;
        loop {
            let tok = self.tokens[c].clone();
            if tok.is_line_term() {
                self.reset_line_context();
                c += 1;
            } else if tok.is_eof() {
                self.cursor = c;
                return (tok.clone(), tok.loc);
            } else if !tok.is_space() && !tok.is_comment() {
                return (tok.clone(), tok.loc);
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
        let tok = self.get();
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

    fn expect_punct(&mut self, expect: Punct) -> Result<(), ParseError> {
        let tok = self.get();
        let loc = self.loc();
        match &tok.kind {
            TokenKind::Punct(punct) => {
                if *punct == expect {
                    Ok(())
                } else {
                    Err(self.error_unexpected(loc))
                }
            }
            _ => Err(self.error_unexpected(loc)),
        }
    }

    fn expect_number(&mut self) -> Result<i64, ParseError> {
        let tok = self.get();
        match tok.kind {
            TokenKind::NumLit(num) => Ok(num),
            _ => panic!("expected number but god {:?}", tok),
        }
    }

    fn error_unexpected(&self, loc: Loc) -> ParseError {
        self.lexer.source_info.show_loc(&loc);
        ParseError::new(ParseErrorKind::UnexpectedToken, loc)
    }

    fn error_eof(&self, loc: Loc) -> ParseError {
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

    pub fn parse_program(&mut self, program: String) -> Result<Node, ParseError> {
        self.tokens = self.lexer.tokenize(program).unwrap().tokens;
        self.cursor = 0;
        let mut node;
        loop {
            node = self.parse_comp_stmt()?;
            self.skip_space();
            if self.tokens[self.cursor].is_eof() {
                break;
            }
        }
        Ok(node)
    }

    pub fn parse_comp_stmt(&mut self) -> Result<Node, ParseError> {
        let mut nodes = vec![];
        let mut loc = self.loc();
        loop {
            let (tok, _) = self.peek();
            match tok.kind {
                TokenKind::Punct(punct) => match punct {
                    Punct::Comment => continue,
                    Punct::Semi => break,
                    _ => {}
                },
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
        // println!("if cond {}", cond);
        self.parse_then()?;
        let then_ = self.parse_comp_stmt()?;
        // println!("if then {}", then_);
        let mut else_ = Node::new_comp_stmt();
        // println!("if else_ {}", else_);
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
        if self.get_if_punct(Punct::Assign) {
            let rhs = self.parse_arg()?;
            Ok(Node::new_assign(lhs, rhs))
        } else {
            Ok(lhs)
        }
    }

    fn parse_arg_logical_or(&mut self) -> Result<Node, ParseError> {
        let lhs = self.parse_arg_logical_and()?;
        if self.get_if_punct(Punct::LAnd) {
            let rhs = self.parse_arg_logical_or()?;
            Ok(Node::new_binop(BinOp::LAnd, lhs, rhs))
        } else {
            Ok(lhs)
        }
    }

    fn parse_arg_logical_and(&mut self) -> Result<Node, ParseError> {
        let lhs = self.parse_arg_eq()?;
        if self.get_if_punct(Punct::LAnd) {
            let rhs = self.parse_arg_logical_and()?;
            Ok(Node::new_binop(BinOp::LAnd, lhs, rhs))
        } else {
            Ok(lhs)
        }
    }

    fn parse_arg_eq(&mut self) -> Result<Node, ParseError> {
        let lhs = self.parse_arg_comp()?;
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
            let lhs = self.parse_primary_ext()?;
            let loc = loc.merge(lhs.loc());
            let lhs = Node::new_binop(BinOp::Mul, lhs, Node::new_number(-1, loc));
            Ok(lhs)
        } else {
            let lhs = self.parse_primary_ext()?;
            Ok(lhs)
        }
    }

    fn parse_primary_ext(&mut self) -> Result<Node, ParseError> {
        let loc = self.loc();
        let mut node = self.parse_primary()?;
        let tok = self.peek_no_skip_line_term();
        if tok.kind == TokenKind::Punct(Punct::LParen) {
            self.get();
            let mut args = ParsedArgs::new();
            args.args = self.parse_parenthesize_args()?;
            let end_loc = self.loc();
            // println!("node: {:?}", node);

            return Ok(Node::new_send(
                Node::new(NodeKind::SelfValue, loc),
                node,
                args,
                loc.merge(end_loc),
            ));
        } else if tok.kind == TokenKind::Punct(Punct::LBoxBrackets) {
            self.expect_line_context_literal(LineContext::Literal(Literal::Array))?;
            let num = self.parse_array_index()?;
            let end_loc = self.loc();
            let node = Node::new_array_index(node, num, loc.merge(end_loc));
            return Ok(node);
        };
        loop {
            let tok = self.peek_no_skip_line_term();
            node = match tok.kind {
                TokenKind::Punct(Punct::Dot) => {
                    self.get();
                    let tok = self.get().clone();
                    match &tok.kind {
                        TokenKind::Ident(s) => {
                            let method = s;
                            let mut args = ParsedArgs::new();
                            let id = self.ident_table.get_ident_id(method);
                            self.skip_space();
                            match self.peek_no_skip_line_term().kind {
                                TokenKind::Punct(Punct::LParen) => {
                                    self.get();
                                    let param_args = self.parse_parenthesize_args()?;
                                    args.args = param_args;

                                    Node::new_send(
                                        node,
                                        Node::new_identifier(id, tok.loc()),
                                        args,
                                        loc.merge(self.loc()),
                                    )
                                }
                                TokenKind::Reserved(Reserved::Do) => {
                                    self.get();
                                    self.skip_space();

                                    let (args_node, args_table) = self.parse_do()?;
                                    args.node = args_node;
                                    args.table = args_table.clone();
                                    if let NodeKind::Ident(id) = args.table.kind {
                                        args.table.kind = NodeKind::TableIdent(id)
                                    };

                                    Node::new_send(
                                        node,
                                        Node::new_identifier(id, tok.loc()),
                                        args,
                                        loc.merge(self.loc()),
                                    )
                                }
                                _ => Node::new_send(
                                    node,
                                    Node::new_identifier(id, tok.loc()),
                                    args,
                                    loc.merge(self.loc()),
                                ),
                            }
                        }
                        TokenKind::NumLit(i) => {
                            let receive_i = node.pick_number();
                            Node::new_decimal_number(
                                receive_i as f64 + ((*i as f64) * 0.1),
                                loc.merge(self.loc()),
                            )
                        }
                        TokenKind::Reserved(Reserved::Class) => {
                            let method = "class".to_string();
                            let id = self.ident_table.get_ident_id(&method);
                            let args = ParsedArgs::new();

                            Node::new_send(
                                node,
                                Node::new_identifier(id, tok.loc()),
                                args,
                                loc.merge(self.loc()),
                            )
                        }
                        _ => panic!("method name must be an identifer."),
                    }
                }
                _ => return Ok(node),
            }
        }
    }

    fn parse_parenthesize_args(&mut self) -> Result<Vec<Node>, ParseError> {
        let mut args = vec![];
        if self.get_if_punct(Punct::RParen) {
            return Ok(args);
        }
        loop {
            args.push(self.parse_arg()?);
            if !self.get_if_punct(Punct::Comma) {
                break;
            }
        }
        if self.get_if_punct(Punct::RParen) {
            Ok(args)
        } else {
            Err(self.error_unexpected(self.loc()))
        }
    }

    fn parse_box_brackets_contents(&mut self) -> Result<Vec<Node>, ParseError> {
        let mut args = vec![];
        if self.get_if_punct(Punct::RBoxBrackets) {
            return Ok(args);
        }
        loop {
            args.push(self.parse_arg()?);
            if !self.get_if_punct(Punct::Comma) {
                break;
            }
        }
        if self.get_if_punct(Punct::RBoxBrackets) {
            self.line_context_stack
                .push(LineContext::Literal(Literal::Array));
            Ok(args)
        } else {
            Err(self.error_unexpected(self.loc()))
        }
    }

    fn parse_array_index(&mut self) -> Result<i64, ParseError> {
        self.get();
        let num = self.expect_number()?;

        if self.get_if_punct(Punct::RBoxBrackets) {
            Ok(num)
        } else {
            Err(self.error_unexpected(self.loc()))
        }
    }

    fn parse_primary(&mut self) -> Result<Node, ParseError> {
        let tok = self.get();
        let loc = tok.loc();
        match &tok.kind {
            TokenKind::Punct(Punct::AtAt) => {
                let tok = self.get();
                let loc = tok.loc();
                match &tok.kind {
                    TokenKind::Ident(name) => {
                        let id = self.ident_table.get_ident_id(name);
                        Ok(Node::new_class_var(id, loc))
                    }
                    _ => unimplemented!(),
                }
            }
            TokenKind::Punct(Punct::At) => {
                let tok = self.get();
                let loc = tok.loc();
                match &tok.kind {
                    TokenKind::Ident(name) => {
                        let id = self.ident_table.get_ident_id(name);
                        Ok(Node::new_instance_var(id, loc))
                    }
                    _ => unimplemented!(),
                }
            }
            TokenKind::Ident(name) => {
                let id = self.ident_table.get_ident_id(name);
                if name == "self" {
                    Ok(Node::new(NodeKind::SelfValue, loc))
                } else if name == "nil" {
                    Ok(Node::new(NodeKind::None, loc))
                } else {
                    Ok(Node::new_identifier(id, loc))
                }
            }
            TokenKind::Const(name) => {
                let id = self.ident_table.get_ident_id(name);
                Ok(Node::new_const(id, loc))
            }
            TokenKind::NumLit(num) => {
                self.line_context_stack
                    .push(LineContext::Literal(Literal::Number));
                Ok(Node::new_number(*num, loc))
            }
            TokenKind::StringLit(s) => Ok(Node::new_string(s.to_string(), loc)),
            TokenKind::Punct(Punct::LParen) => {
                let node = self.parse_comp_stmt()?;
                let tok = self.get();
                if tok.kind == TokenKind::Punct(Punct::RParen) {
                    Ok(node)
                } else {
                    Err(self.error_unexpected(self.loc()))
                }
            }
            TokenKind::Punct(Punct::LBoxBrackets) => {
                if self.is_first_line_context() {
                    let contents = self.parse_box_brackets_contents()?;
                    let end_loc = self.loc();
                    let node = Node::new_array(contents, loc.merge(end_loc));
                    Ok(node)
                } else {
                    unimplemented!();
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
            TokenKind::Reserved(Reserved::Do) => {
                let (node, _) = self.parse_do()?;
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
        self.expect_out_of_method_block_context()?;
        self.block_context_stack.push(BlockContext::Class);
        self.expect_first_line_context()?;
        let name = self.parse_const()?;

        self.skip_space();
        let inheritance_class_id = match self.peek_no_skip_line_term().kind {
            TokenKind::Punct(Punct::LT) => {
                self.get();
                let inheritance_class_name = self.parse_const()?;
                Some(self.ident_table.get_ident_id(&inheritance_class_name))
            }
            _ => None,
        };
        let id = self.ident_table.get_ident_id(&name);

        let body = self.parse_comp_stmt()?;
        self.expect_reserved(Reserved::End)?;
        self.block_context_stack.pop().unwrap();
        self.reset_line_context();

        Ok(Node::new_class_decl(id, body, inheritance_class_id))
    }

    pub fn parse_line(&mut self) -> Result<Node, ParseError> {
        let loc = self.loc();
        match &self.get().kind {
            TokenKind::Line => Ok(Node::new_line(loc)),
            _ => Err(self.error_unexpected(loc)),
        }
    }

    pub fn parse_const(&mut self) -> Result<String, ParseError> {
        let loc = self.loc();
        match &self.get().kind {
            TokenKind::Const(s) => Ok(s.clone()),
            _ => Err(self.error_unexpected(loc)),
        }
    }

    pub fn parse_then(&mut self) -> Result<(), ParseError> {
        if self.get_if_term() {
            return Ok(());
        }
        self.expect_reserved(Reserved::Then)?;
        Ok(())
    }

    fn parse_def(&mut self) -> Result<Node, ParseError> {
        self.block_context_stack.push(BlockContext::Method);
        self.expect_first_line_context()?;
        let loc = self.loc();
        let name = match &self.get().kind {
            TokenKind::Ident(s) => s.clone(),
            _ => return Err(self.error_unexpected(loc)),
        };
        let id = self.ident_table.get_ident_id(&name);

        let args = self.parse_params()?;
        let body = self.parse_comp_stmt()?;
        self.expect_reserved(Reserved::End)?;
        self.block_context_stack.pop().unwrap();
        self.reset_line_context();

        Ok(Node::new_method_decl(id, args, body))
    }

    fn parse_do(&mut self) -> Result<(Node, Node), ParseError> {
        let table = match self.peek_no_skip_line_term().kind {
            TokenKind::Punct(Punct::Pipe) => {
                self.get();
                let node = self.parse_ident()?;
                self.expect_punct(Punct::Pipe)?;
                node
            }
            _ => Node::new_none(),
        };

        self.block_context_stack.push(BlockContext::Method);
        let body = self.parse_comp_stmt()?;
        self.expect_reserved(Reserved::End)?;
        self.block_context_stack.pop().unwrap();
        self.reset_line_context();

        Ok((Node::new_block_decl(body), table))
    }

    pub fn parse_params(&mut self) -> Result<Vec<Node>, ParseError> {
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
            args.push(Node::new(NodeKind::TableIdent(id), loc));
            if !self.get_if_punct(Punct::Comma) {
                break;
            }
        }
        if self.get_if_punct(Punct::RParen) {
            Ok(args)
        } else {
            let tok = self.peek().1;
            Err(self.error_unexpected(tok))
        }
    }

    pub fn parse_ident(&mut self) -> Result<Node, ParseError> {
        let tok = self.get();
        let loc = tok.loc();
        match &tok.kind {
            TokenKind::Ident(name) => {
                let id = self.ident_table.get_ident_id(name);
                Ok(Node::new_identifier(id, tok.loc()))
            }
            _ => Err(self.error_unexpected(loc)),
        }
    }
}
