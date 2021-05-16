use taiko::lexer::lexer::*;
use taiko::util::annot::*;

fn main() {
    let prog = "\na  = 0;\nb=3;";
    println!("{}", prog);

    let mut lex = Lexer::new(prog);
    match lex.tokenize() {
        Err(e) => println!("{:?}", e),
        Ok(tokens) => {
            lex.show_loc(&Loc(8, 8, 1));
            for token in tokens {
                println!("{:?}", token);
            }
        }
    }
}
