use crate::util::annot::*;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident(String),
    Const(String),
    NumLit(i64),
    StringLit(String),
    Reserved(Reserved),
    Punct(Punct),
    Space,
    Line,
    EOF,
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Reserved {
    BEGIN,
    END,
    Alias,
    And,
    Begin,
    Break,
    Case,
    Class,
    Def,
    Defined,
    Do,
    Else,
    Elsif,
    End,
    Ensure,
    False,
    For,
    If,
    In,
    Module,
    Next,
    Nil,
    Not,
    Or,
    Redo,
    Rescue,
    Retry,
    Return,
    // Self,
    Super,
    Then,
    True,
    Undef,
    Unless,
    Until,
    When,
    While,
    Yield,
    __LINE__,
    __FILE__,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Punct {
    Plus,
    Minus,
    Mul,
    Div,
    And,
    Or,
    LParen,
    RParen,
    Semi,
    Colon,
    Comma,
    Eq,
    NE,
    GT,
    GE,
    LT,
    LE,
    LAnd,
    LOr,
    Assign,
    Comment,
    Dot,
}

pub type Token = Annot<TokenKind>;

impl Token {
    pub fn new_ident(ident: String, loc: Loc) -> Self {
        Annot::new(TokenKind::Ident(ident), loc)
    }

    pub fn new_const(ident: String, loc: Loc) -> Self {
        Annot::new(TokenKind::Const(ident), loc)
    }

    pub fn new_reserved(ident: Reserved, loc: Loc) -> Self {
        Annot::new(TokenKind::Reserved(ident), loc)
    }

    pub fn new_numlit(num: i64, loc: Loc) -> Self {
        Annot::new(TokenKind::NumLit(num), loc)
    }

    pub fn new_comment(loc: Loc) -> Self {
        Annot::new(TokenKind::Punct(Punct::Comment), loc)
    }

    pub fn new_string(loc: Loc) -> Self {
        Annot::new(TokenKind::String, loc)
    }

    pub fn new_punct(punct: Punct, loc: Loc) -> Self {
        Annot::new(TokenKind::Punct(punct), loc)
    }

    pub fn new_space(loc: Loc) -> Self {
        Annot::new(TokenKind::Space, loc)
    }

    pub fn new_line(loc: Loc) -> Self {
        Annot::new(TokenKind::Line, loc)
    }

    pub fn new_eof(loc: Loc) -> Self {
        Annot::new(TokenKind::EOF, loc)
    }
}

impl Token {
    pub fn is_space(&self) -> bool {
        self.kind == TokenKind::Space
    }

    pub fn is_line_term(&self) -> bool {
        self.kind == TokenKind::Line
    }

    pub fn is_comment(&self) -> bool {
        self.kind == TokenKind::Punct(Punct::Comment)
    }

    pub fn is_eof(&self) -> bool {
        self.kind == TokenKind::EOF
    }

    pub fn is_term(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::Line | TokenKind::EOF | TokenKind::Punct(Punct::Semi)
        )
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            TokenKind::EOF => write!(f, "Token!{:?}, {}]", self.kind, self.loc.0),
            TokenKind::Punct(punct) => write!(
                f,
                "Token![Punct(Punct::{:?}), {}, {}]",
                punct, self.loc.0, self.loc.1
            ),
            TokenKind::Reserved(reserved) => write!(
                f,
                "Token![Reserved(Reserved::{:?}), {}, {}]",
                reserved, self.loc.0, self.loc.1
            ),
            _ => write!(
                f,
                "Token![{:?}, {}, {}],",
                self.kind, self.loc.0, self.loc.1
            ),
        }
    }
}
