use crate::token::token::*;
use crate::util::annot::*;
use crate::util::util::*;
use rustc_hash::FxHashMap;

#[derive(Debug, Clone)]
pub struct Lexer {
    source_info: SourceInfo,
    len: usize,
    line_start_pos: usize,
    token_start_pos: usize,
    absolute_column_pos: usize,
    relative_column_pos: usize,
    line_pos: usize,
    reserved: FxHashMap<String, Reserved>,
}

#[derive(Debug, Clone)]
pub struct LexerResult {
    pub source_info: SourceInfo,
    pub tokens: Vec<Token>,
}

impl LexerResult {
    fn new(tokens: Vec<Token>, lexer: Lexer) -> Self {
        LexerResult {
            source_info: lexer.source_info,
            tokens,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
    EOF,
    UnexpectedChar,
    NotMatchPunctuation,
    ForbiddenTab,
}

impl Lexer {
    pub fn new(code_text: impl Into<String>) -> Self {
        let mut reserved = FxHashMap::default();
        macro_rules! reg_reserved {
            ( $($id:expr => $variant:path),+ ) => {
                $(
                    reserved.insert($id.to_string(), $variant);
                )+
            };
        }

        reg_reserved! {
            "BEGIN" => Reserved::BEGIN,
            "END" => Reserved::END,
            "alias" => Reserved::Alias,
            "begin" => Reserved::Begin,
            "break" => Reserved::Break,
            "case" => Reserved::Case,
            "class" => Reserved::Class,
            "def" => Reserved::Def,
            "defined?" => Reserved::Defined,
            "do" => Reserved::Do,
            "else" => Reserved::Else,
            "elsif" => Reserved::Elsif,
            "end" => Reserved::End,
            "false" => Reserved::False,
            "if" => Reserved::If,
            "return" => Reserved::Return,
            "then" => Reserved::Then,
            "true" => Reserved::True
        };

        let source_info = SourceInfo::new(code_text);
        let len = source_info.code.len();

        Lexer {
            source_info,
            len,
            line_start_pos: 0,
            token_start_pos: 0,
            absolute_column_pos: 0,
            relative_column_pos: 0,
            line_pos: 0,
            reserved,
        }
    }

    fn push_line_coordinate(&mut self) {
        self.source_info.coordinates.push((
            self.line_start_pos,
            self.absolute_column_pos,
            self.line_pos,
        ));
    }

    fn push_last_coordinate(&mut self) {
        let last_line_pos = self.line_pos;
        let last_absolute_column_start_pos = self.line_start_pos;
        let last_absolute_column_last_pos =
            self.source_info.code.len() - last_absolute_column_start_pos;
        self.source_info.coordinates.push((
            last_absolute_column_start_pos,
            last_absolute_column_start_pos + last_absolute_column_last_pos,
            last_line_pos,
        ));
    }

    fn read_eol(&mut self) -> Token {
        self.push_line_coordinate();
        let tok = Token::new_line(Loc(self.absolute_column_pos, self.absolute_column_pos));
        self.line_pos += 1;
        self.absolute_column_pos += 1;
        self.line_start_pos = self.absolute_column_pos;
        self.relative_column_pos = 0;
        tok
    }

    fn read_space(&mut self) -> Token {
        self.token_start_pos = self.absolute_column_pos;
        loop {
            match self.peek() {
                Ok(c) => match c {
                    ' ' => {
                        self.absolute_column_pos += 1;
                        self.relative_column_pos += 1;
                    }
                    _ => break,
                },
                Err(_) => break,
            }
        }
        Token::new_space(Loc(self.token_start_pos, self.absolute_column_pos - 1))
    }

    fn skip_whitespace(&mut self) -> Result<Option<Token>, Error> {
        for absolute_column_pos in self.absolute_column_pos..self.len {
            let ch = self.source_info.code[absolute_column_pos];
            if ch == '\n' {
                return Ok(Some(self.read_eol()));
            } else if ch == '\t' {
                return Err(Error::ForbiddenTab);
            } else if ch == ' ' {
                return Ok(Some(self.read_space()));
            } else if !ch.is_ascii_whitespace() {
                self.absolute_column_pos = absolute_column_pos;
                return Ok(None);
            }
        }
        self.absolute_column_pos = self.len;
        Ok(None)
    }

    fn get(&mut self) -> Result<char, Error> {
        if self.absolute_column_pos >= self.len {
            Err(Error::EOF)
        } else {
            let ch = self.source_info.code[self.absolute_column_pos];
            if ch == '\n' {
                self.push_line_coordinate();
                self.line_pos += 1;
                self.line_start_pos = self.absolute_column_pos + 1;
                self.relative_column_pos = 0;
            }
            self.absolute_column_pos += 1;
            self.relative_column_pos += 1;
            Ok(ch)
        }
    }

    fn peek(&self) -> Result<char, Error> {
        if self.absolute_column_pos >= self.len {
            Err(Error::EOF)
        } else {
            Ok(self.source_info.code[self.absolute_column_pos])
        }
    }

    fn cur_loc(&self) -> Loc {
        Loc(self.token_start_pos, self.absolute_column_pos - 1)
    }

    fn read_number_literal(&mut self, ch: char) -> Result<Token, Error> {
        let mut tok = ch.to_string();
        loop {
            let ch: char;
            match self.peek() {
                Ok(_ch) => ch = _ch,
                Err(_) => break,
            }
            if ch.is_numeric() {
                tok.push(self.get()?);
            } else if ch == '_' {
                self.get()?;
            } else {
                break;
            }
        }
        let i = tok.parse::<i64>().unwrap();
        Ok(self.new_numlit(i))
    }

    fn read_ascii_alphabetic(&mut self, ch: char) -> Result<Token, Error> {
        let mut tok = ch.to_string();
        loop {
            let ch: char;
            match self.peek() {
                Ok(_ch) => ch = _ch,
                Err(_) => {
                    break;
                }
            };
            if ch.is_ascii_alphanumeric() || ch == '_' {
                tok.push(self.get()?);
            } else {
                break;
            }
        }
        match self.reserved.get(&tok) {
            Some(reserved) => Ok(self.new_reserved(*reserved)),
            None => Ok(self.new_ident(tok)),
        }
    }

    fn read_ascii_punct(&mut self, ch: char) -> Result<Token, Error> {
        let mut punct = FxHashMap::default();
        macro_rules! reg_punct {
                    ( $($id:expr => $variant:path),+ ) => {
                        $(
                            punct.insert($id.to_string(), $variant);
                        )+
                    };
                }

        reg_punct! {
            "+" => Punct::Plus,
            "-" => Punct::Minus,
            "*" => Punct::Mul,
            "/" => Punct::Div,
            "(" => Punct::LParen,
            ")" => Punct::RParen,
            ";" => Punct::Semi,
            ":" => Punct::Colon,
            "," => Punct::Comma
        }

        punct.insert("=".to_string(), {
            let ch = self.peek()?;
            if ch == '=' {
                self.get()?;
                Punct::Equal
            } else {
                Punct::Assign
            }
        });

        match punct.contains_key(&ch.to_string()) {
            true => Ok(self.new_punct(*punct.get(&ch.to_string()).unwrap())),
            false => Err(Error::NotMatchPunctuation),
        }
    }

    fn read_comment(&mut self) -> Token {
        let line_end_pos = match self.goto_eol() {
            None => self.line_start_pos - self.last_coordinate().0 - 2,
            Some(Error::EOF) => self.absolute_column_pos - 1,
            _ => unimplemented!(),
        };

        Token::new_comment(Loc(self.token_start_pos, line_end_pos))
    }

    fn goto_eol(&mut self) -> Option<Error> {
        loop {
            match self.get() {
                Ok('\n') => return None,
                Err(Error::EOF) => return Some(Error::EOF),
                _ => (),
            }
        }
    }

    fn last_coordinate(&self) -> (usize, usize, usize) {
        *self.source_info.coordinates.last().unwrap()
    }

    pub fn tokenize(mut self) -> Result<LexerResult, Error> {
        let mut tokens: Vec<Token> = vec![];
        loop {
            while let Some(tok) = self.skip_whitespace()? {
                tokens.push(tok);
            }
            self.token_start_pos = self.absolute_column_pos;
            let ch: char;
            match self.get() {
                Ok(_ch) => ch = _ch,
                Err(Error::EOF) => {
                    self.push_last_coordinate();
                    tokens.push(self.new_eof(self.token_start_pos));
                    break;
                }
                Err(_) => unimplemented!(),
            };

            let token = if ch.is_ascii_alphabetic() || ch == '_' {
                self.read_ascii_alphabetic(ch)?
            } else if ch.is_numeric() {
                self.read_number_literal(ch)?
            } else if ch.is_ascii_punctuation() {
                if ch == '#' {
                    self.token_start_pos = self.absolute_column_pos - 1;
                    self.read_comment()
                } else {
                    self.read_ascii_punct(ch)?
                }
            } else {
                return Err(Error::UnexpectedChar);
            };
            tokens.push(token);
        }
        Ok(LexerResult::new(tokens, self))
    }
}

#[allow(unused)]
impl Lexer {
    fn new_ident(&self, ident: String) -> Token {
        Annot::new(TokenKind::Ident(ident), self.cur_loc())
    }

    fn new_reserved(&self, ident: Reserved) -> Token {
        Annot::new(TokenKind::Reserved(ident), self.cur_loc())
    }

    fn new_numlit(&self, num: i64) -> Token {
        Annot::new(TokenKind::NumLit(num), self.cur_loc())
    }

    fn new_punct(&self, punct: Punct) -> Token {
        Annot::new(TokenKind::Punct(punct), self.cur_loc())
    }

    fn new_space(&self) -> Token {
        Annot::new(TokenKind::Space, self.cur_loc())
    }

    fn new_line(&self) -> Token {
        Token::new_line(self.cur_loc())
    }

    fn new_eof(&self, pos: usize) -> Token {
        Annot::new_eof(Loc(pos, pos))
    }
}
