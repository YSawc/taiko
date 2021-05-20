use crate::util::annot::*;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident(String),
    NumLit(i64),
    Reserved(Reserved),
    Punct(Punct),
    Space,
    Line,
    EOF,
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
    LParen,
    RParen,
    Semi,
    Colon,
    Equal,
    Comment,
}

pub type Token = Annot<TokenKind>;

impl Token {
    pub fn new_ident(ident: String, loc: Loc) -> Self {
        Annot::new(TokenKind::Ident(ident), loc)
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
        self.value == TokenKind::Space
    }

    pub fn is_line_term(&self) -> bool {
        self.value == TokenKind::Line
    }

    pub fn is_eof(&self) -> bool {
        self.value == TokenKind::EOF
    }

    pub fn is_term(&self) -> bool {
        matches!(
            self.value,
            TokenKind::Line | TokenKind::EOF | TokenKind::Punct(Punct::Semi)
        )
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Token![{:?}, {}, {}],",
            self.value, self.loc.0, self.loc.1
        )
    }
}
