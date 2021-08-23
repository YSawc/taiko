use clap::{App, Arg};
use taiko::parser::parser::*;
use taiko::vm::vm::*;
extern crate clap;
extern crate rustyline;

fn main() {
    let app = App::new("taiko")
        .version("0.0.1")
        .author("ysawc")
        .about("A toy ruby interpreter")
        .arg(Arg::new("file"));
    let app_matches = app.get_matches();
    match app_matches.value_of("file") {
        Some(file_name) => {
            file_read(file_name);
            return;
        }
        None => {
            repl();
            return;
        }
    };
}

fn repl() {
    let mut rl = rustyline::Editor::<()>::new();
    let mut program = String::new();
    let mut parser = Parser::new();
    let mut vm = VM::new();
    vm.repl_init_method(parser.lexer.source_info.clone(), parser.ident_table.clone());
    loop {
        let prompt = if program.len() == 0 { ">" } else { "*" };
        let readline = rl.readline(&format!("irb:{} ", prompt).to_string());
        println!("readline: {:?}", readline);
        let mut line = match readline {
            Ok(line) => line,
            Err(_) => return,
        };
        line.push('\n');
        rl.add_history_entry(line.clone());
        program = format!("{}{}", program, line);

        let source_info = parser.lexer.source_info.clone();
        let ident_table = parser.ident_table.clone();
        match parser.parse_program(program.clone()) {
            Ok(node) => {
                vm.repl_init_method(parser.lexer.source_info.clone(), parser.ident_table.clone());
                vm.init_iseq(node);
                match vm.eval() {
                    Ok(result) => {
                        parser.lexer.source_info = vm.source_info.clone();
                        parser.ident_table = vm.ident_table.clone();
                        println!("=> {:?}", result);
                    }
                    Err(_) => {
                        parser.lexer.source_info = source_info;
                        parser.ident_table = ident_table;
                        println!("{}", program);
                    }
                }
            }
            Err(err) => {
                parser.lexer.source_info = source_info;
                parser.ident_table = ident_table;
                if ParseErrorKind::EOF == err.kind {
                    continue;
                }
                println!("ParseError: {:?}", err.kind);
            }
        }
        program = String::new();
    }
}

fn file_read(file_name: impl Into<String>) {
    use std::fs::*;
    use std::io::Read;
    let file_name = file_name.into();
    let path = std::path::Path::new(&file_name).with_extension("rb");
    let absolute_path = match path.canonicalize() {
        Ok(path) => path,
        Err(ioerr) => {
            let msg = format!("{}", ioerr);
            println!("No such file or directory --- {} (LoadError)", &file_name);
            println!("{}", msg);
            return;
        }
    };

    let mut file_body = String::new();

    match OpenOptions::new().read(true).open(&absolute_path) {
        Ok(mut ok) => ok
            .read_to_string(&mut file_body)
            .ok()
            .expect("cannot read file"),
        Err(ioerr) => {
            let msg = format!("{}", ioerr);
            println!("Error: Cannot find module file. '{}", &file_name);
            println!("{}", msg);
            return;
        }
    };

    let mut parser = Parser::new();
    let res = parser.parse_program(file_body);
    match res {
        Ok(node) => {
            let mut vm = VM::new();
            vm.repl_init_method(parser.lexer.source_info.clone(), parser.ident_table.clone());
            vm.init_iseq(node);
            match vm.eval() {
                Ok(result) => println!("-> {:?}", &result),
                Err(_) => {}
            }
        }
        Err(err) => println!("ParseError: {:?}", err.kind),
    }
}
