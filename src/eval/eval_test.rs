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
    fn func1() {
        let program = "
            def fact(a)
                puts(a)
                if a == 1
                    1
                else
                    a * fact(a-1)
                end
            end

            fact(5)
        ";
        let expected = Value::FixNum(120);
        eval_script(program, expected);
    }

    #[test]
    fn func2() {
        let program = "
        def self1
        puts(self)
        end

        self1()

        class Foo
            puts(self)
            class Bar
                puts(self)
            end
        end

        self1()
        ";
        let expected = Value::Nil;
        eval_script(program, expected);
    }
}
