use taiko::lexer::lexer::*;

fn main() {
    let prog = "\na = 0;\nb=3;";
    println!("{}", prog);

    let mut lex = Lexer::new(prog);
    match lex.tokenize() {
        Err(e) => println!("{:?}", e),
        Ok(tokens) => {
            for token in tokens {
                println!("{:?}", token);
            }
        }
    }
}
