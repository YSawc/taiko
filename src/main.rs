use taiko::lexer::lexer::*;
use taiko::util::annot::*;

fn main() {
    let prog = "\na  = 0;\nbb=3;";
    println!("{}", prog);

    let mut lex = Lexer::new(prog);
    match lex.tokenize() {
        Err(e) => println!("{:?}", e),
        Ok(tokens) => {
            lex.show_loc(&Loc(0, 1, 2));
            for token in tokens {
                println!("{:?}", token);
            }
        }
    }
}
