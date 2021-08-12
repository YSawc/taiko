#[cfg(test)]
mod test {
    use crate::class::class::*;
    use crate::instance::instance::*;
    use crate::parser::parser::*;
    use crate::value::value::Value::Instance;
    use crate::value::value::*;
    use crate::vm::vm::*;

    fn eval_script(script: impl Into<String>, expected: Value) {
        let mut parser = Parser::new();
        let node = parser.parse_program(script.into()).unwrap();

        let mut vm = VM::new();
        vm.init(parser.lexer.source_info, parser.ident_table, node);
        vm.eval_seq();
        let val = vm.exec_stack();
        if val != expected {
            panic!("Expected:{:?} Got:{:?}", expected, val);
        }
    }

    // #[test]
    // fn func1() {
    //     let program = "
    //         def fact(a)
    //             puts(a)
    //             if a == 1
    //                 1
    //             else
    //                 a * fact(a-1)
    //             end
    //         end

    //         fact(5)
    //     ";
    //     let expected = Value::FixNum(120);
    //     eval_script(program, expected);
    // }

    // #[test]
    // fn func2() {
    //     let program = "
    //     def self1
    //     puts(self)
    //     end

    //     self1()

    //     class Foo
    //         puts(self)
    //         class Bar
    //             puts(self)
    //         end
    //     end

    //     self1()
    //     ";
    //     let expected = Value::Nil;
    //     eval_script(program, expected);
    // }

    // #[test]
    // fn func3() {
    //     let program = "
    //     a = 1
    //     def foo
    //         a
    //     end
    //     foo()
    //     ";
    //     let expected = Value::FixNum(1);
    //     eval_script(program, expected);
    // }

    // #[test]
    // fn func4() {
    //     let program = "
    //     a = 1
    //     def foo
    //         a = 2
    //     end
    //     foo()
    //     ";
    //     let expected = Value::FixNum(2);
    //     eval_script(program, expected);
    // }

    // #[test]
    // fn new_fn1() {
    //     let program = "
    //     class Foo
    //     end
    //     Foo.new
    //     Foo.new
    //     Foo.new
    //     ";
    //     let expected = Instance(InstanceRef(2));
    //     eval_script(program, expected);
    // }

    // #[test]
    // fn propagated_local_var1() {
    //     let program = r#"
    //         a = 1
    //         class Foo
    //           a = 2
    //           def bar(b)
    //             b*2
    //           end

    //           def bar2
    //             a
    //           end
    //         end

    //         assert(Foo.new.bar(5), 10)
    //         Foo.new.bar2
    //         "#;
    //     let expected = Value::FixNum(2);
    //     eval_script(program, expected);
    // }

    #[test]
    fn assert1() {
        let program = "
        assert(1, 1)
        ";
        let expected = Value::Nil;
        eval_script(program, expected);
    }

    // #[test]
    // fn assert2() {
    //     let program = r#"
    //     a = 1
    //     class Foo
    //       a = 2
    //       def bar(b)
    //         b*2
    //       end
    //     end

    //     assert(a, 1)
    //     assert(Foo.new.bar(5), 10)
    //     "#;
    //     let expected = Value::Nil;
    //     eval_script(program, expected);
    // }

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

    // #[test]
    // fn decimal_number1() {
    //     let program = "
    //         123.4;
    //     ";
    //     let expected = Value::FixDecimalNum(123.4);
    //     eval_script(program, expected);
    // }

    // #[test]
    // fn decimal_number2() {
    //     let program = "
    //         12.3 + 4 - 5.6 * 7.8 / 9;
    //     ";
    //     let expected = Value::FixDecimalNum(11.446666666666667);
    //     eval_script(program, expected);
    // }

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

    // #[test]
    // fn self_class1() {
    //     let program = "
    //         class Bar
    //         end
    //         Bar.class
    //     ";
    //     let expected = Value::SelfClass(Class::Class);
    //     eval_script(program, expected);
    // }

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
            a = 0
            b = 0
            24.times do |n|
              b = b + n + a
              a = b
            end

            a
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

    // #[test]
    // fn each() {
    //     let program = "
    //         v = ['one', 2, 'three', 4]
    //         v.each do |c|
    //           puts(c)
    //         end
    //     ";
    //     let expected = Value::Nil;
    //     eval_script(program, expected);
    // }

    // #[test]
    // fn class1() {
    //     let program = "
    //         class Vec
    //           @xxx=100
    //           def set_xxx(x)
    //             @xxx = x
    //           end
    //           def len(x,y)
    //             def sq(x)
    //               x*x
    //             end
    //             sq(x)+sq(y)
    //           end
    //           def get_xxx
    //             @xxx
    //           end
    //         end
    //         foo1 = Vec.new
    //         foo1.set_xxx(1)
    //         assert(25, foo1.len(3,4))
    //         foo1.set_xxx(777)
    //         foo2 = Vec.new
    //         assert(777, foo1.get_xxx)
    //         foo2.set_xxx(999)
    //         assert(777, foo1.get_xxx)
    //         assert(999, foo2.get_xxx)
    //     ";
    //     let expected = Value::Nil;
    //     eval_script(program, expected);
    // }

    // #[test]
    // fn class2() {
    //     let program = "
    //         class Car
    //           def setName(str)
    //             @name = str
    //           end

    //           def getName
    //             @name
    //           end
    //         end

    //         car1 = Car.new
    //         car1.setName('Legacy')

    //         car2 = Car.new
    //         car2.setName('XV')
    //         assert(car2.getName, 'XV')
    //         assert(car1.getName, 'Legacy')
    //     ";
    //     let expected = Value::Nil;
    //     eval_script(program, expected);
    // }

    // #[test]
    // fn instance_variables() {
    //     let program = "
    //         class Car
    //           def setName(str)
    //             @name = str
    //           end

    //           def getName
    //             @name
    //           end
    //         end

    //         car1 = Car.new
    //         car1.setName('Legacy')
    //         car1.instance_variables
    //     ";
    //     let expected_strings = Value::String("@name".to_string());
    //     let mut expected_vec = vec![];
    //     expected_vec.push(expected_strings);
    //     let expected_result = Value::Array(expected_vec);
    //     eval_script(program, expected_result);
    // }

    // #[test]
    // fn class_instance() {
    //     let program = "
    //         class Car
    //           @@class_var = 2

    //           def set_class_var(i)
    //             @@class_var = i
    //           end

    //           def get_class_var
    //             @@class_var
    //           end
    //         end

    //         car1 = Car.new
    //         car1.set_class_var(22222)
    //         car1.get_class_var
    //     ";
    //     let expected = Value::FixNum(22222);
    //     eval_script(program, expected);
    // }

    // #[test]
    // fn class_inheritance() {
    //     let program = "
    //       class A
    //         @xxx=100
    //         def set_xxx(x)
    //           @xxx = x
    //         end
    //         def len(x,y)
    //           def sq(x)
    //             x*x
    //           end
    //           sq(x)+sq(y)
    //         end
    //         def get_xxx
    //           @xxx
    //         end
    //       end

    //       class B < A
    //       end
    //       foo1 = A.new
    //       foo1.set_xxx(1)
    //       assert(25, foo1.len(3,4))
    //       foo1.set_xxx(777)
    //       foo2 = B.new
    //       assert(777, foo1.get_xxx)
    //       foo2.set_xxx(999)
    //       assert(777, foo1.get_xxx)
    //       assert(999, foo2.get_xxx)
    //     ";
    //     let expected = Value::Nil;
    //     eval_script(program, expected);
    // }
}
