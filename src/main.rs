use taiko::eval::eval::*;
use taiko::lexer::lexer::*;
use taiko::parser::parser::*;

fn main() {
    let prog = "
    a = 7;
    b = 3; c = 2;
    a * b - 2";

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
                    parser.source_info.show_loc(&node.loc);
                    let mut evaluator = Evaluator::new(parser.source_info, parser.ident_table);
                    println!("{:?}", evaluator.eval_node(&node));
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        }
    }
}
