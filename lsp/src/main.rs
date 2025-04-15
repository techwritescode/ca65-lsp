mod asm_server;
mod codespan;
mod configuration;
mod diagnostics;
mod instructions;
mod logger;
mod symbol_cache;
mod completion;
mod definition;
mod error;
mod path;
mod documentation;

use asm_server::Asm;
use tower_lsp_server::{LspService, Server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    symbol_cache::init_symbol_cache();
    instructions::init_instruction_map();
    documentation::init();

    let (service, socket) = LspService::new(|client| {
        Asm::new(client)
    });
    Server::new(stdin, stdout, socket)
        .serve(service)
        .await;

    Ok(())
}
