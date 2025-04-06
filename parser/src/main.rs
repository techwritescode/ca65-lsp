use std::fs::File;
use parser::{Instructions, Tokenizer};
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

    let mut tokenizer = Tokenizer::new(buf, &instructions);
    let tokens = tokenizer.parse().expect("Failed to parse tokens");
    println!("{:#?}", tokens);

    let mut parser = parser::Parser::new(&tokens);
    let ast = parser.parse();
    println!("{:#?}", ast);
}
