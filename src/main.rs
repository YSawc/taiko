use taiko::eval::eval::*;
use taiko::lexer::lexer::*;
use taiko::parser::parser::*;

fn main() {
    let prog = r#"
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

    let lexer = Lexer::new(prog);
    match lexer.tokenize() {
        Err(e) => println!("{:?}", e),
        Ok(lexer_result) => {
            for token in &lexer_result.tokens {
                println!("{}", token);
            }
            let mut parser = Parser::new(lexer_result);
            match parser.parse_program() {
                Ok(node) => {
                    println!("node: {}", node);
                    let mut evaluator = Evaluator::new(parser.source_info, parser.ident_table);
                    println!("result: {:?}", evaluator.eval_node(&node));
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        }
    }
}
