use taiko::eval::eval::*;
use taiko::lexer::lexer::*;
use taiko::parser::parser::*;

fn main() {
    let prog = "
        a = 1
        class Foo
          assert(a==1, 'must true')
          a = 2
          def bar
            puts(a)
            a = 3
            assert(a==3, 'must true')
          end
          assert(a==2, 'must true')
        end

        assert(a==1, 'must true')
        puts(Foo.new)
        puts(Foo.new.bar)
        ";

    println!("{}", prog);

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
