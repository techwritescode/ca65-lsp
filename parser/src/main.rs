use parser::{Instructions, ParseError, Tokenizer, TokenizerError};
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
    let cs_file = codespan::File::new("test", buf);

    let mut tokenizer = Tokenizer::new(&cs_file.source, &instructions);
    match tokenizer.parse() {
        Ok(tokens) => {
            let mut parser = parser::Parser::new(&tokens);
            let ast = parser.parse();
            match ast {
                Ok(ast) => {
                    println!("{:#?}", ast);
                }
                Err(e) => {
                    print_parse_error(&cs_file, e);
                }
            }
        }
        Err(e) => {
            print_error(&cs_file, e);
        }
    }
}

fn print_error(file: &codespan::File, error: TokenizerError) {
    println!("{}", error.kind);
    print_error_offset(file, error.offset);
}

fn print_parse_error(file: &codespan::File, error: ParseError) {
    match error {
        ParseError::EOF => println!("Unexpected end of file"),
        ParseError::Expected { expected, received } => {
            println!(
                "Expected {:?} but received {:?}",
                expected, received.token_type
            );
            print_error_offset(file, received.span.start);
        }
        ParseError::UnexpectedToken(token) => {
            println!("Unexpected token {:?}", token);
            print_error_offset(file, token.span.start);
        }
    }
}

fn print_error_offset(file: &codespan::File, offset: usize) {
    let pos = file.byte_index_to_position(offset).unwrap();
    let line = file.get_line(pos.line).unwrap();
    let line_str = file.get_line_source(line).unwrap();

    let line_number_str = pos.line.to_string();

    print!("{line_number_str}| {line_str}");

    let marker = std::iter::repeat_n(' ', line_number_str.len() + 2)
        .chain(std::iter::repeat_n('~', pos.character))
        .collect::<String>();
    print!("{marker}^");
    println!();
}
