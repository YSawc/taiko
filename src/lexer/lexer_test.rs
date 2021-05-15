#[cfg(test)]
use crate::lexer::lexer::*;
#[cfg(test)]
use crate::token::token::*;
#[cfg(test)]
use crate::util::annot::*;

#[cfg(test)]
fn assert_lexer(lex: &mut Lexer, ans: Vec<Annot<TokenKind>>) {
    match lex.tokenize() {
        Ok(l) => assert_eq!(l, ans),
        Err(err) => panic!("{:?}", err),
    }
}

#[test]
fn lexer_test() {
    let program = "a = 0;\n if a == 1_000 then 5 else 10";
    let ans = vec![
        Annot {
            value: TokenKind::Ident("a".to_string()),
            loc: Loc(0, 0, 0),
        },
        Annot {
            value: TokenKind::Punct(Punct::Equal),
            loc: Loc(2, 2, 0),
        },
        Annot {
            value: TokenKind::NumLit(0),
            loc: Loc(4, 4, 0),
        },
        Annot {
            value: TokenKind::Punct(Punct::Semi),
            loc: Loc(5, 5, 0),
        },
        Annot {
            value: TokenKind::Reserved(Reserved::If),
            loc: Loc(1, 2, 1),
        },
        Annot {
            value: TokenKind::Ident("a".to_string()),
            loc: Loc(4, 4, 1),
        },
        Annot {
            value: TokenKind::Punct(Punct::Equal),
            loc: Loc(6, 6, 1),
        },
        Annot {
            value: TokenKind::Punct(Punct::Equal),
            loc: Loc(7, 7, 1),
        },
        Annot {
            value: TokenKind::NumLit(1000),
            loc: Loc(9, 13, 1),
        },
        Annot {
            value: TokenKind::Reserved(Reserved::Then),
            loc: Loc(15, 18, 1),
        },
        Annot {
            value: TokenKind::NumLit(5),
            loc: Loc(20, 20, 1),
        },
        Annot {
            value: TokenKind::Reserved(Reserved::Else),
            loc: Loc(22, 25, 1),
        },
        Annot {
            value: TokenKind::NumLit(10),
            loc: Loc(27, 28, 1),
        },
    ];

    println!("{}", program);
    let mut lex = Lexer::new(program);
    assert_lexer(&mut lex, ans);
}
