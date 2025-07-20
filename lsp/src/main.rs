mod analysis;
mod asm_server;
mod cache_file;
mod codespan;
mod completion;
mod data;
mod definition;
mod documentation;
mod error;
mod index_engine;
mod logger;
mod state;

use asm_server::Asm;
use data::instructions;
use tower_lsp_server::{LspService, Server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    instructions::init_instruction_map();
    documentation::init();

    let (service, socket) = LspService::new(Asm::new);
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
