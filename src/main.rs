use taiko::lexer::lexer::*;
use taiko::parser::parser::*;

fn main() {
    // let prog = "\na  = 0;\nbb=3;\n# comment line\n c = 3;# comment_line";
    let prog = "(7+ 4) *
    5 -49 ; 6*7;";
    println!("{}", prog);

    let lexer = Lexer::new(prog);
    match lexer.tokenize() {
        Err(e) => println!("{:?}", e),
        Ok(lexer_result) => {
            for token in &lexer_result.tokens {
                println!("{:?}", token);
            }
            let mut parser = Parser::new(lexer_result);
            match parser.parse_comp_stmt() {
                Ok(_) => {}
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        }
    }
}
