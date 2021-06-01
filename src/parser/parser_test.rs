#[cfg(test)]
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
        let mut eval = Evaluator::new(parser.source_info, parser.ident_table);
        let res = eval.eval_node(&node);
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

    #[test]
    fn decimal_number1() {
        let program = "
            123.4;
        ";
        let expected = Value::FixDecimalNum(123.4);
        eval_script(program, expected);
    }

    #[test]
    fn decimal_number2() {
        let program = "
            12.3 + 4 - 5.6 * 7.8 / 9;
        ";
        let expected = Value::FixDecimalNum(11.446666666666667);
        eval_script(program, expected);
    }

    #[test]
    fn local_var1() {
        let program = "
            a = 6;
            b = 2;c = 1;
            a/b-c;
        ";
        let expected = Value::FixNum(2);
        eval_script(program, expected);
    }

    #[test]
    fn to_i1() {
        let program = "
            '34'.to_i
        ";
        let expected = Value::FixNum(34);
        eval_script(program, expected);
    }

    #[test]
    fn to_s1() {
        let program = "
            34.to_s
        ";
        let expected = Value::String("34".to_string());
        eval_script(program, expected);
    }
}
