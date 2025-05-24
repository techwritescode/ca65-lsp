mod asm_server;
mod codespan;
mod completion;
mod configuration;
mod definition;
mod diagnostics;
mod documentation;
mod error;
mod index_engine;
mod instructions;
mod logger;
mod path;
mod state;
mod symbol_cache;

use asm_server::Asm;
use tower_lsp_server::{LspService, Server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    symbol_cache::init_symbol_cache();
    instructions::init_instruction_map();
    documentation::init();

    let (service, socket) = LspService::new(|client| Asm::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
