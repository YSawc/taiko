use taiko::lexer::lexer::*;

fn main() {
    let prog = "\na  = 0;\nbb=3;\n# comment line\n c = 3;# comment_line";
    println!("{}", prog);

    let lexer = Lexer::new(prog);
    match lexer.tokenize() {
        Err(e) => println!("{:?}", e),
        Ok(lexer_result) => {
            println!("{:?}", lexer_result.source_info.coordinates);
            for token in &lexer_result.tokens {
                println!("{:?}", token);
            }
        }
    }
}
