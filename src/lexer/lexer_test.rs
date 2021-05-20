#[cfg(test)]
use crate::lexer::lexer::*;
#[cfg(test)]
use crate::token::token::*;
#[cfg(test)]
use crate::util::annot::*;

#[cfg(test)]
fn assert_lexer(lexer: Lexer, ans: Vec<Annot<TokenKind>>) {
    match lexer.tokenize() {
        Ok(lexer_result) => assert_eq!(lexer_result.tokens, ans),
        Err(err) => panic!("{:?}", err),
    }
}

#[cfg(test)]
macro_rules! Token (
    (Ident($item:expr), ($loc_0:expr, $loc_1:expr, $loc_2:expr, $loc_3:expr)) => {
        Token::new_ident($item, Loc($loc_0, $loc_1, $loc_2, $loc_3))
    };
    (Space, ($loc_0:expr, $loc_1:expr, $loc_2:expr, $loc_3:expr)) => {
        Token::new_space(Loc($loc_0, $loc_1, $loc_2, $loc_3))
    };
    (Punct($item:path), ($loc_0:expr, $loc_1:expr, $loc_2:expr, $loc_3:expr)) => {
        Token::new_punct($item, Loc($loc_0, $loc_1, $loc_2, $loc_3))
    };
    (Reserved($item:path), ($loc_0:expr, $loc_1:expr, $loc_2:expr, $loc_3:expr)) => {
        Token::new_reserved($item, Loc($loc_0, $loc_1, $loc_2, $loc_3))
    };
    (NumLit($num:expr), ($loc_0:expr, $loc_1:expr, $loc_2:expr, $loc_3:expr)) => {
        Token::new_numlit($num, Loc($loc_0, $loc_1, $loc_2, $loc_3))
    };
    (Line, ($loc_0:expr, $loc_1:expr, $loc_2:expr, $loc_3:expr)) => {
        Token::new_line(Loc($loc_0, $loc_1, $loc_2, $loc_3))
    };
    (EOF, ($loc_0:expr, $loc_1:expr, $loc_2:expr, $loc_3:expr)) => {
        Token::new_eof(Loc($loc_0, $loc_1, $loc_2, $loc_3))
    };
);

#[test]
fn lexer_test() {
    let program = "a = 0;\n if a == 1_000 then 5 else 10;# comment_line";
    let ans = vec![
        Token![Ident("a".to_string()), (0, 0, 0, 0)],
        Token![Space, (1, 1, 0, 0)],
        Token![Punct(Punct::Equal), (2, 2, 0, 0)],
        Token![Space, (3, 3, 0, 0)],
        Token![NumLit(0), (4, 4, 0, 0)],
        Token![Punct(Punct::Semi), (5, 5, 0, 0)],
        Token![Line, (6, 6, 0, 0)],
        Token![Space, (0, 0, 1, 1)],
        Token![Reserved(Reserved::If), (1, 2, 1, 1)],
        Token![Space, (3, 3, 1, 1)],
        Token![Ident("a".to_string()), (4, 4, 1, 1)],
        Token![Space, (5, 5, 1, 1)],
        Token![Punct(Punct::Equal), (6, 6, 1, 1)],
        Token![Punct(Punct::Equal), (7, 7, 1, 1)],
        Token![Space, (8, 8, 1, 1)],
        Token![NumLit(1000), (9, 13, 1, 1)],
        Token![Space, (14, 14, 1, 1)],
        Token![Reserved(Reserved::Then), (15, 18, 1, 1)],
        Token![Space, (19, 19, 1, 1)],
        Token![NumLit(5), (20, 20, 1, 1)],
        Token![Space, (21, 21, 1, 1)],
        Token![Reserved(Reserved::Else), (22, 25, 1, 1)],
        Token![Space, (26, 26, 1, 1)],
        Token![NumLit(10), (27, 28, 1, 1)],
        Token![Punct(Punct::Semi), (29, 29, 1, 1)],
        Token![Punct(Punct::Comment), (30, 43, 1, 1)],
        Token![EOF, (44, 44, 1, 1)],
    ];

    println!("{}", program);
    let lexer = Lexer::new(program);
    assert_lexer(lexer, ans);
}

#[test]
fn lexer_forbidden_tab() {
    let program = "\ta = 0";
    println!("{}", program);
    let lexer = Lexer::new(program);
    let lexer_result = lexer.tokenize().unwrap_err();
    assert_eq!(lexer_result, Error::ForbiddenTab);
}
