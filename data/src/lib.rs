use quote::quote;
use proc_macro2::{TokenStream};
use regex;
use std::fmt::format;

#[proc_macro]
pub fn include_documentation(token_stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let source_str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/65816-opcodes.md"));
    
    let instr_re = regex::Regex::new(r#"\{([a-z]{3})}"#).unwrap();
    let description_re = regex::Regex::new(r#"(?s)((\{[a-z]{3}}\n)+)\{:}(.*?)\{\.}"#).unwrap();
    let descriptions = description_re
        .captures_iter(source_str)
        .map(|c| match c.extract() {
            (_, [name, _, docs]) => (name, docs.trim()),
            _ => unreachable!("Failed to parse description"),
        });
    
    let mut opcode_setup_str: TokenStream = TokenStream::new();
    
    for (opcodes, description) in descriptions {
        let opcodes = instr_re
            .captures_iter(opcodes)
            .map(|c| c.get(1).unwrap().as_str())
            .collect::<Vec<_>>();
        for opcode in opcodes {
            opcode_setup_str.extend(
                quote! {
                    instruction_map.insert(#opcode.to_owned(), #description.to_owned());
                }
            );
        }
    }
    //
    // let init_function = format!(
    //     "pub static OPCODE_DOCUMENTATION: std::sync::OnceLock<std::collections::HashMap<String, String>> = std::sync::OnceLock::new();\n fn documentation_init() {{\n let mut instruction_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();\n {}OPCODE_DOCUMENTATION.set(instruction_map).expect(\"Failed to set map\");\n }}\n",
    //     opcode_setup_str
    // );

    // init_function.parse().unwrap()

    quote!{
        pub static OPCODE_DOCUMENTATION: std::sync::OnceLock<std::collections::HashMap<String, String>> = std::sync::OnceLock::new();
        fn documentation_init() { 
            let mut instruction_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            #opcode_setup_str
            OPCODE_DOCUMENTATION.set(instruction_map).expect("Failed to set map");
        } 
    }.into()
}
