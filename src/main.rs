use taiko::lexer::lexer::*;
use taiko::util::annot::*;

fn main() {
    let prog = "\na  = 0;\nbb=3;";
    println!("{}", prog);

    let lexer = Lexer::new(prog);
    match lexer.tokenize() {
        Err(e) => println!("{:?}", e),
        Ok(lexer_result) => {
            lexer_result.clone().show_loc(&Loc(0, 1, 2));
            for token in lexer_result.tokens {
                println!("{:?}", token);
            }
        }
    }
}
