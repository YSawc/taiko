#[cfg(test)]
mod test {
    use crate::lexer::lexer::*;
    use crate::token::token::*;
    use crate::util::annot::*;

    fn assert_lexer(program: impl Into<String>, ans: Vec<Annot<TokenKind>>) {
        let mut lexer = Lexer::new();
        match lexer.tokenize(program.into()) {
            Ok(lexer_result) => assert_eq!(lexer_result.tokens, ans),
            Err(err) => panic!("{:?}", err),
        }
    }

    macro_rules! Token (
    (Ident($item:expr), ($loc_0:expr, $loc_1:expr)) => {
        Token::new_ident($item, Loc($loc_0, $loc_1))
    };
    (Space, ($loc_0:expr, $loc_1:expr)) => {
        Token::new_space(Loc($loc_0, $loc_1))
    };
    (Punct($item:path), ($loc_0:expr, $loc_1:expr)) => {
        Token::new_punct($item, Loc($loc_0, $loc_1))
    };
    (Reserved($item:path), ($loc_0:expr, $loc_1:expr)) => {
        Token::new_reserved($item, Loc($loc_0, $loc_1))
    };
    (NumLit($num:expr), ($loc_0:expr, $loc_1:expr)) => {
        Token::new_numlit($num, Loc($loc_0, $loc_1))
    };
    (Line, ($loc_0:expr, $loc_1:expr)) => {
        Token::new_line(Loc($loc_0, $loc_1))
    };
    (EOF, ($loc_0:expr, $loc_1:expr)) => {
        Token::new_eof(Loc($loc_0, $loc_1))
    };
);

    #[test]
    fn lexer_test() {
        let program = "a = 0;\n if a == 1_000 then 5 else 10;  # comment_line";
        let ans = vec![
            Token![Ident("a".to_string()), (0, 0)],
            Token![Space, (1, 1)],
            Token![Punct(Punct::Assign), (2, 2)],
            Token![Space, (3, 3)],
            Token![NumLit(0), (4, 4)],
            Token![Punct(Punct::Semi), (5, 5)],
            Token![Line, (6, 6)],
            Token![Space, (7, 7)],
            Token![Reserved(Reserved::If), (8, 9)],
            Token![Space, (10, 10)],
            Token![Ident("a".to_string()), (11, 11)],
            Token![Space, (12, 12)],
            Token![Punct(Punct::Eq), (13, 14)],
            Token![Space, (15, 15)],
            Token![NumLit(1000), (16, 20)],
            Token![Space, (21, 21)],
            Token![Reserved(Reserved::Then), (22, 25)],
            Token![Space, (26, 26)],
            Token![NumLit(5), (27, 27)],
            Token![Space, (28, 28)],
            Token![Reserved(Reserved::Else), (29, 32)],
            Token![Space, (33, 33)],
            Token![NumLit(10), (34, 35)],
            Token![Punct(Punct::Semi), (36, 36)],
            Token![Space, (37, 38)],
            Token![Punct(Punct::Comment), (39, 52)],
            Token![EOF, (53, 53)],
        ];
        assert_lexer(program, ans);
    }

    #[test]
    fn lexer_forbidden_tab() {
        let program = "\ta = 0";
        println!("{}", program);
        let mut lexer = Lexer::new();
        let lexer_result = lexer.tokenize(program).unwrap_err();
        assert_eq!(lexer_result, Error::ForbiddenTab);
    }

    #[test]
    fn cmp1() {
        let program = "5 > 0";
        let ans = vec![
            Token![NumLit(5), (0, 0)],
            Token![Space, (1, 1)],
            Token![Punct(Punct::GT), (2, 2)],
            Token![Space, (3, 3)],
            Token![NumLit(0), (4, 4)],
            Token![EOF, (5, 5)],
        ];
        assert_lexer(program, ans);
    }

    #[test]
    fn cmp2() {
        let program = "5 >= 0";
        let ans = vec![
            Token![NumLit(5), (0, 0)],
            Token![Space, (1, 1)],
            Token![Punct(Punct::GE), (2, 3)],
            Token![Space, (4, 4)],
            Token![NumLit(0), (5, 5)],
            Token![EOF, (6, 6)],
        ];
        assert_lexer(program, ans);
    }

    #[test]
    fn cmp3() {
        let program = "5 == 0";
        let ans = vec![
            Token![NumLit(5), (0, 0)],
            Token![Space, (1, 1)],
            Token![Punct(Punct::Eq), (2, 3)],
            Token![Space, (4, 4)],
            Token![NumLit(0), (5, 5)],
            Token![EOF, (6, 6)],
        ];
        assert_lexer(program, ans);
    }

    #[test]
    fn cmp4() {
        let program = "5 != 0";
        let ans = vec![
            Token![NumLit(5), (0, 0)],
            Token![Space, (1, 1)],
            Token![Punct(Punct::NE), (2, 3)],
            Token![Space, (4, 4)],
            Token![NumLit(0), (5, 5)],
            Token![EOF, (6, 6)],
        ];
        assert_lexer(program, ans);
    }

    #[test]
    fn cmp5() {
        let program = "5 < 0";
        let ans = vec![
            Token![NumLit(5), (0, 0)],
            Token![Space, (1, 1)],
            Token![Punct(Punct::LT), (2, 2)],
            Token![Space, (3, 3)],
            Token![NumLit(0), (4, 4)],
            Token![EOF, (5, 5)],
        ];
        assert_lexer(program, ans);
    }

    #[test]
    fn cmp6() {
        let program = "5 <= 0";
        let ans = vec![
            Token![NumLit(5), (0, 0)],
            Token![Space, (1, 1)],
            Token![Punct(Punct::LE), (2, 3)],
            Token![Space, (4, 4)],
            Token![NumLit(0), (5, 5)],
            Token![EOF, (6, 6)],
        ];
        assert_lexer(program, ans);
    }
}
