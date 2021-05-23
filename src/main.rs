use taiko::eval::eval::*;
use taiko::lexer::lexer::*;
use taiko::parser::parser::*;

fn main() {
    let prog = "
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
