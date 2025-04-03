use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::sync::OnceLock;
use tower_lsp_server::lsp_types::MessageType;
use crate::asm_server::Asm;

pub static CA65_KEYWORDS_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();

pub async fn parse_ca65_html(asm_server: &Asm) {
    let arc_config = asm_server.get_configuration();
    let cc65_path = arc_config.as_ref().toolchain.cc65.as_ref().unwrap();
    let ca65_html_path = format!("{cc65_path}\\html\\ca65.html");

    // get contents of ca65.html
    let mut f = std::fs::File::open(ca65_html_path).expect("could not open ca65.html");
    let mut reader = BufReader::new(f);
    let mut hm: HashMap<String, String> = HashMap::new();
    let mut s = String::new();
    let mut curr_keyword = String::new();
    let mut curr_description = String::new();
    for line in reader.lines().map_while(Result::ok) {
        if line.starts_with("<H2><A NAME=\".") {
            if !curr_keyword.is_empty() {
                hm.insert(curr_keyword.clone(), curr_description.clone());
                curr_keyword.clear();
                curr_description.clear();
            }
            curr_keyword = line.trim_start_matches("<H2><A NAME=\"").split_once("\"").unwrap().0.to_string();
        } else if !curr_keyword.is_empty() {
            curr_description.push_str(&line);
            curr_description.push('\n');
        }
    }

    CA65_KEYWORDS_MAP.set(hm).unwrap();

    // for (k, v) in hm {
    //     s.push_str(&k);
    //     s.push_str("\n--------");
    //     s.push_str(&v);
    //     s.push_str("\n\n\n\n\n\n");
    // }
    //
    // asm_server.get_client().log_message(MessageType::INFO, s).await;
}