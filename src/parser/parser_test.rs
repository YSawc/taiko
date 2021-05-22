#[allow(unused_imports, dead_code)]
mod test {
    use crate::eval::eval::*;
    use crate::lexer::lexer::*;
    use crate::parser::parser::*;
    use crate::value::value::*;

    fn eval_script(script: impl Into<String>, expected: Value) {
        let lexer = Lexer::new(script);
        let result = lexer.tokenize().unwrap();
        let mut parser = Parser::new(result);
        let node = parser.parse_comp_stmt().unwrap();
        let res = eval_node(&node);
        if res != expected {
            panic!("Expected:{:?} Got:{:?}", expected, res);
        }
    }

    #[test]
    fn if1() {
        let program = "if 5*4==16 +4 then 7; end";
        let expected = Value::FixNum(7);
        eval_script(program, expected);
    }

    #[test]
    fn if2() {
        let program = "if
        5*4==16 +4
        7 end";
        let expected = Value::FixNum(7);
        eval_script(program, expected);
    }

    #[test]
    fn if3() {
        let program = "if 5*9==16 +4
        7 elsif 4==4+9 then 8 elsif 3==1+2 then 10
        else 12 end";
        let expected = Value::FixNum(10);
        eval_script(program, expected);
    }
}
