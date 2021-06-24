#[cfg(test)]
mod test {
    use crate::class::class::*;
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

    #[test]
    fn self_class1() {
        let program = "
            class Bar
            end
            Bar.class
        ";
        let expected = Value::SelfClass(Class::Class);
        eval_script(program, expected);
    }

    #[test]
    fn times1() {
        let program = "
            3.times do
              puts('hello')
            end
        ";
        let expected = Value::Nil;
        eval_script(program, expected);
    }

    #[test]
    fn times2() {
        let program = "
            3.times do |n|
              puts(n)
            end
        ";
        let expected = Value::Nil;
        eval_script(program, expected);
    }

    #[test]
    fn times3() {
        let program = "
            a = 0
            255.times do |n|
              a = a + n
            end
            a
        ";
        let expected = Value::FixNum(32385);
        eval_script(program, expected);
    }

    #[test]
    fn times4() {
        let program = "
            @g = 0
            g = 0
            24.times do |n|
              g = g + n + @g
              @g = g
            end

            @g
        ";
        let expected = Value::FixNum(16777191);
        eval_script(program, expected);
    }

    #[test]
    fn len() {
        let program = "
            [1, 'string', 3, 4].len
        ";
        let expected = Value::FixNum(4);
        eval_script(program, expected);
    }

    #[test]
    fn array_index() {
        let program = "
            [1, 'string', 3, 4][1]
        ";
        let expected = Value::String("string".to_string());
        eval_script(program, expected);
    }

    #[test]
    fn each() {
        let program = "
            v = ['one', 2, 'three', 4]
            v.each do |c|
              puts(c)
            end
        ";
        let expected = Value::Nil;
        eval_script(program, expected);
    }
}
