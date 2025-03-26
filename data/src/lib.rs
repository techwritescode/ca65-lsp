use proc_macro2::TokenStream;
use quote::quote;

#[proc_macro]
pub fn include_documentation(_token_stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let source_str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/65816-opcodes.md"));

    let instr_re = regex::Regex::new(r#"\{([a-z]{3})}"#).unwrap();
    let description_re = regex::Regex::new(r#"(?s)((\{[a-z]{3}}\n)+)\{:}(.*?)\{\.}"#).unwrap();
    let descriptions = description_re
        .captures_iter(source_str)
        .map(|c| {
            let (_, [name, _, docs]) = c.extract();
            (name, docs.trim())
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

    quote! {
        pub static OPCODE_DOCUMENTATION: std::sync::OnceLock<std::collections::HashMap<String, String>> = std::sync::OnceLock::new();
        fn documentation_init() { 
            let mut instruction_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            #opcode_setup_str
            OPCODE_DOCUMENTATION.set(instruction_map).expect("Failed to set map");
        } 
    }.into()
}
