mod asm_server;
mod codespan;
mod configuration;
mod diagnostics;
mod instructions;
mod logger;
mod symbol_cache;
mod ca65_doc;
mod completion;
mod definition;
mod error;
mod path;

use asm_server::Asm;
use tower_lsp_server::{LspService, Server};

use data::include_documentation;

include_documentation!();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    symbol_cache::init_symbol_cache();
    instructions::init_instruction_map();
    ca65_doc::parse_json_to_hashmap();

    documentation_init();

    let (service, socket) = LspService::new(|client| {
        Asm::new(client)
    });
    Server::new(stdin, stdout, socket)
        .serve(service)
        .await;

    Ok(())
}
