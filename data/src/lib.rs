use std::io::BufRead;

use proc_macro2::TokenStream;
use quote::quote;

enum DocParserState {
    Opcodes,
    Description,
}

#[proc_macro]
pub fn include_documentation(_token_stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let doc_file = std::fs::File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/65816-opcodes.md")).expect("Could not open opcode documentation file.");
    let doc_file_lines = std::io::BufReader::new(doc_file)
        .lines()
        .map_while(Result::ok);
    let mut opcode_setup_str: TokenStream = TokenStream::new();

    let mut state = DocParserState::Opcodes;
    let mut curr_opcodes: Vec<String> = vec![];
    let mut curr_description = String::new();
    for line in doc_file_lines {
        match state {
            DocParserState::Opcodes => {
                if line == "{:}" {
                    state = DocParserState::Description;
                } else if let Some(opcode) = line.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
                    curr_opcodes.push(opcode.to_string());
                }
            },
            DocParserState::Description => {
                if line == "{.}" {
                    // opcode_setup_str.extend(#curr_opcodes.map(...)) isn't possible because Vec<String> (curr_opcodes) doesn't implement the quote::ToTokens trait
                    for opcode in curr_opcodes.drain(..) {
                        opcode_setup_str.extend(quote! {
                            instruction_map.insert(#opcode.to_owned(), #curr_description.to_owned());
                        })
                    }
                    curr_description.clear();
                    state = DocParserState::Opcodes;
                } else {
                    curr_description.push_str(&line);
                    curr_description.push('\n');
                }
            }
        }
    }
    
    quote! {
        pub static OPCODE_DOCUMENTATION: std::sync::OnceLock<std::collections::HashMap<String, String>> = std::sync::OnceLock::new();
        fn documentation_init() { 
            let mut instruction_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            #opcode_setup_str
            OPCODE_DOCUMENTATION.set(instruction_map).expect("Failed to set map");
        } 
    }.into()
}
