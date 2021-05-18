use taiko::lexer::lexer::*;

fn main() {
    let prog = "\na  = 0;\nbb=3;\n# comment line\n c = 3;";
    // let prog = "a = 0;\n if a == 1_000 then 5 else 10;# comment_line";
    println!("{}", prog);

    let lexer = Lexer::new(prog);
    match lexer.tokenize() {
        Err(e) => println!("{:?}", e),
        Ok(lexer_result) => {
            println!("{:?}", lexer_result.coordinates);
            for token in lexer_result.tokens {
                println!("{:?}", token);
            }
        }
    }
}
