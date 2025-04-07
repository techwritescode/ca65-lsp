use parser::{Instructions, Tokenizer, TokenizerError};
use std::fs::File;
use std::io::Read;

fn main() {
    let mut args = std::env::args();
    let instructions = Instructions::load();

    if args.len() < 2 {
        eprintln!("Usage: parser <file>");
        std::process::exit(1);
    }

    let mut file = File::open(args.nth(1).unwrap()).expect("Failed to open file");
    let mut buf = String::new();
    file.read_to_string(&mut buf).expect("Failed to read file");

    let mut tokenizer = Tokenizer::new(&buf, &instructions);
    match tokenizer.parse() {
        Ok(tokens) => {
            let mut parser = parser::Parser::new(&tokens);
            let ast = parser.parse();
            println!("{:#?}", ast);
        }
        Err(e) => {
            print_error(&buf, e);
        }
    }
}

fn print_error(file: &String, error: TokenizerError) {
    let file = codespan::File::new("test file", file.to_string());
    let pos = file.byte_index_to_position(error.offset).unwrap();
    let line = file.get_line(pos.line).unwrap();
    let line_str = file.get_line_source(line).unwrap();

    let line_number_str = pos.line.to_string();

    println!("{} at {}:{}", error.kind, pos.line+1, pos.character+1);
    println!();
    print!("{line_number_str}| {line_str}");

    let marker = std::iter::repeat_n(' ', line_number_str.len() + 2)
        .chain(std::iter::repeat_n('~', pos.character))
        .collect::<String>();
    print!("{marker}^");
    println!();
}
