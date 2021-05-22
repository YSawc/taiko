use taiko::eval::eval::*;
use taiko::lexer::lexer::*;
use taiko::parser::parser::*;

fn main() {
    let prog = "if 5*9==16 +4
    7 elsif 4==4+9 then 8 elsif 3==1+2 then 10
    else 12 end";

    println!("{}", prog);

    let lexer = Lexer::new(prog);
    match lexer.tokenize() {
        Err(e) => println!("{:?}", e),
        Ok(lexer_result) => {
            for token in &lexer_result.tokens {
                println!("{}", token);
            }
            let mut parser = Parser::new(lexer_result);
            match parser.parse_comp_stmt() {
                Ok(node) => {
                    parser.source_info.show_loc(&node.loc);
                    println!("{:?}", eval_node(&node));
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        }
    }
}
