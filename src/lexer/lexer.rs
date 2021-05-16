use crate::token::token::*;
use crate::util::annot::*;
use rustc_hash::FxHashMap;

#[derive(Debug, Clone)]
pub struct Lexer {
    code: Vec<char>,
    len: usize,
    absolute_column_pos: usize,
    relative_column_pos: usize,
    line_pos: usize,
    coordinates: Vec<(usize, usize, usize)>,
    reserved: FxHashMap<String, Reserved>,
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
        let code = code_text.into().chars().collect::<Vec<char>>();
        let len = code.len();
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

        Lexer {
            code,
            len,
            absolute_column_pos: 0,
            relative_column_pos: 0,
            line_pos: 0,
            coordinates: vec![],
            reserved,
        }
    }

    fn skip_whitespace(&mut self) -> Result<Option<Token>, Error> {
        for absolute_column_pos in self.absolute_column_pos..self.len {
            let ch = self.code[absolute_column_pos];
            if ch == '\n' {
                if let Some(coordinate) = self.coordinates.clone().last() {
                    self.coordinates.push((
                        coordinate.0 + 1,
                        self.absolute_column_pos,
                        self.line_pos,
                    ));
                } else {
                    self.coordinates
                        .push((0, absolute_column_pos, self.line_pos));
                }
                let tok = Token::new_line(Loc(
                    self.relative_column_pos,
                    self.relative_column_pos,
                    self.line_pos,
                ));
                self.line_pos += 1;
                self.absolute_column_pos += 1;
                self.relative_column_pos = 0;
                return Ok(Some(tok));
            } else if ch == '\t' {
                return Err(Error::ForbiddenTab);
            } else if ch == ' ' {
                let tok = Token::new_space(Loc(
                    self.relative_column_pos,
                    self.relative_column_pos,
                    self.line_pos,
                ));
                self.absolute_column_pos += 1;
                self.relative_column_pos += 1;
                return Ok(Some(tok));
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
            let ch = self.code[self.absolute_column_pos];
            if ch == '\n' {
                self.line_pos += 1;
                self.relative_column_pos = 0;
            }
            self.absolute_column_pos += 1;
            self.relative_column_pos += 1;
            Ok(ch)
        }
    }

    fn peek(&mut self) -> Result<char, Error> {
        if self.absolute_column_pos >= self.len {
            Err(Error::EOF)
        } else {
            Ok(self.code[self.absolute_column_pos])
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, Error> {
        let mut tokens: Vec<Token> = vec![];
        loop {
            while let Some(tok) = self.skip_whitespace()? {
                tokens.push(tok);
            }
            let relative_column_pos = self.relative_column_pos;
            let ch: char;
            match self.get() {
                Ok(_ch) => ch = _ch,
                Err(_) => break,
            };
            macro_rules! cur_loc {
                () => {
                    Loc(
                        relative_column_pos,
                        self.relative_column_pos - 1,
                        self.line_pos,
                    )
                };
            }
            let token = if ch.is_ascii_alphabetic() || ch == '_' {
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
                    Some(reserved) => Token::new_reserved(*reserved, cur_loc!()),
                    None => Token::new_ident(tok, cur_loc!()),
                }
            } else if ch.is_numeric() {
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
                Token::new_numlit(i, cur_loc!())
            } else if ch.is_ascii_punctuation() {
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
                    "=" => Punct::Equal
                }

                match punct.contains_key(&ch.to_string()) {
                    true => Token::new_punct(*punct.get(&ch.to_string()).unwrap(), cur_loc!()),
                    false => return Err(Error::NotMatchPunctuation),
                }
            } else {
                return Err(Error::UnexpectedChar);
            };
            tokens.push(token);
        }
        let eof_line_pos = self.coordinates.last().unwrap().2 + 1;
        let eof_absolute_column_start_pos = self.coordinates.last().unwrap().1 + 1;
        let eof_absolute_column_last_pos = self.code.len() - eof_absolute_column_start_pos;
        self.coordinates.push((
            eof_absolute_column_start_pos,
            eof_absolute_column_start_pos + eof_absolute_column_last_pos,
            eof_line_pos,
        ));
        Ok(tokens)
    }

    pub fn show_loc(&self, loc: &Loc) {
        if let Some(line) = self.coordinates.iter().find(|x| x.2 == loc.2) {
            println!(
                "{}",
                self.code[(line.0)..(line.1)].iter().collect::<String>()
            );
            println!("{}{}", " ".repeat(loc.0), "^".repeat(loc.1 - loc.0 + 1));
        } else {
            panic!("no location found!");
        };
    }
}
