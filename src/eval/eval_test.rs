#[cfg(test)]
mod test {
    use crate::eval::eval::*;
    use crate::instance::instance::*;
    use crate::parser::parser::*;
    use crate::value::value::Value::Instance;
    use crate::value::value::*;

    fn eval_script(script: impl Into<String>, expected: Value) {
        let mut parser = Parser::new();
        let node = parser.parse_program(script.into()).unwrap();

        let mut eval = Evaluator::new();
        eval.init(parser.lexer.source_info, parser.ident_table);
        match eval.eval_node(&node) {
            Ok(res) => {
                if res != expected {
                    panic!("Expected:{:?} Got:{:?}", expected, res);
                }
            }
            Err(err) => panic!("Got runtime error: {:?}", err),
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

    #[test]
    fn func3() {
        let program = "
        a = 1
        def foo
            a
        end
        foo()
        ";
        let expected = Value::FixNum(1);
        eval_script(program, expected);
    }

    #[test]
    fn func4() {
        let program = "
        a = 1
        def foo
            a = 2
        end
        foo()
        ";
        let expected = Value::FixNum(2);
        eval_script(program, expected);
    }

    #[test]
    fn new_fn1() {
        let program = "
        class Foo
        end
        Foo.new
        Foo.new
        Foo.new
        ";
        let expected = Instance(InstanceRef(2));
        eval_script(program, expected);
    }

    #[test]
    fn propagated_local_var1() {
        let program = r#"
            a = 1
            class Foo
              a = 2
              def bar(b)
                b*2
              end

              def bar2
                a
              end
            end

            assert(Foo.new.bar(5) == 10, "must pass")
            Foo.new.bar2
            "#;
        let expected = Value::FixNum(2);
        eval_script(program, expected);
    }

    #[test]
    fn assert1() {
        let program = "
        assert(1 == 1, 'must pass.')
        ";
        let expected = Value::Nil;
        eval_script(program, expected);
    }

    #[test]
    fn assert2() {
        let program = r#"
        a = 1
        class Foo
          a = 2
          def bar(b)
            b*2
          end
        end

        assert(Foo.new.bar(5) == 10, "must pass")
        "#;
        let expected = Value::Nil;
        eval_script(program, expected);
    }
}
