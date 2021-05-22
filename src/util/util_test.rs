#[cfg(test)]
mod test {
    use crate::lexer::lexer::*;
    use crate::parser::parser::*;

    fn eval_script(script: impl Into<String>, expected: Vec<(usize, usize, usize)>) {
        let lexer = Lexer::new(script);
        let result = lexer.tokenize().unwrap();
        let parser = Parser::new(result);
        let coordinates = parser.source_info.coordinates;
        if coordinates != expected {
            panic!("Expected:{:?} Got:{:?}", expected, coordinates);
        }
    }

    #[test]
    fn coordinates1() {
        let prog = "\na  = 0;\nbb=3;\n# comment line\n c = 3;# comment_line";
        let expected = vec![(0, 0, 0), (1, 8, 1), (9, 14, 2), (15, 29, 3), (30, 51, 4)];
        eval_script(prog, expected);
    }
}
