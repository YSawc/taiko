#[cfg(test)]
mod test {
    use crate::parser::parser::*;

    fn eval_script(script: impl Into<String>, expected: Vec<(usize, usize, usize)>) {
        let mut parser = Parser::new();
        parser.parse_program(script.into()).unwrap();
        let coordinates = parser.lexer.source_info.coordinates;
        if coordinates != expected {
            panic!("Expected:{:?} Got:{:?}", expected, coordinates);
        }
    }

    #[test]
    fn coordinates1() {
        let prog = "
a  = 0
bb=3
# comment line
c = 3
# comment_line";
        let expected = vec![
            (0, 0, 0),
            (1, 7, 1),
            (8, 12, 2),
            (13, 27, 3),
            (28, 33, 4),
            (34, 48, 5),
        ];
        eval_script(prog, expected);
    }
}
