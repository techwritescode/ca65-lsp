use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use tower_lsp_server::lsp_types::MessageType;
use crate::asm_server::Asm;

pub async fn parse_ca65_html(asm_server: &Asm) {
    let arc_config = asm_server.get_configuration();
    let config = arc_config.as_ref();
    let cc65_path = config.toolchain.cc65.as_ref().unwrap();
    let ca65_html_path = format!("{cc65_path}\\html\\ca65.html");

    // asm_server.get_client().log_message(MessageType::INFO, format!("here's the path, {ca65_html_path}")).await;

    // asm_server.get_client().log_message(MessageType::LOG, "k here we goooooo").await;
    // asm_server.get_client().log_message(MessageType::LOG, "k here we goooooo2").await;
    // asm_server.get_client().log_message(MessageType::LOG, "k here we goooooo3").await;
    // asm_server.get_client().log_message(MessageType::LOG, "k here we goooooo4").await;

    // let f = std::fs::File::open(&ca65_html_path).unwrap();
    // asm_server.get_client().log_message(MessageType::LOG, "ay uhhhh that worked").await;

    // get contents of ca65.html
    let mut f = std::fs::File::open(ca65_html_path).expect("could not open ca65.html");
    let mut reader = BufReader::new(f);
    let mut count = 0;
    let mut s = String::new();
    for line in reader.lines().map_while(Result::ok) {
        // asm_server.get_client().log_message(MessageType::LOG, line).await;
        s.push_str(&line);
        count += 1;
        if count == 10 { break; }
    }

    asm_server.get_client().log_message(MessageType::INFO, s).await;
}