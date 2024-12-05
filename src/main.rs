mod asm_server;
mod instructions;
mod parser;
mod symbol_cache;

use asm_server::Asm;
use std::fs::File;
use tower_lsp::{LspService, Server};
use tracing::Level;
use tracing_subscriber::{filter, layer::SubscriberExt, Layer, Registry};

mod codespan;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let current_path = std::env::current_dir().expect("Failed to get working directory");
    let log_path = std::path::Path::new(&current_path).join("asm.log");
    let log = File::create(log_path).expect("failed to open log");

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_ansi(false)
        .with_writer(log)
        .with_filter(filter::LevelFilter::from_level(Level::DEBUG));

    tracing::subscriber::set_global_default(Registry::default().with(fmt_layer))?;

    std::panic::set_hook(Box::new(|err| {
        tracing::error!("{:#?}", err);
    }));

    tracing::error!("Starting up");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    symbol_cache::init_symbol_cache();
    instructions::init_instruction_map();

    let (service, socket) = LspService::new(|client| Asm::new(client));
    Server::new(stdin, stdout)
        .interleave(socket)
        .serve(service)
        .await;

    // text_store::init_text_store();
    //
    // tracing::warn!("{:#?}", instructions::INSTRUCTION_MAP);
    //
    // tracing::info!("Startup");
    // let (connection, _io_threads) = Connection::stdio();

    // let (id, _) = connection.initialize_start()?;
    //
    // let server_capabilities = ServerCapabilities {
    //     text_document_sync: Some(TextDocumentSyncCapability::Kind(
    //         TextDocumentSyncKind::INCREMENTAL,
    //     )),
    //     hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
    //     completion_provider: Some(lsp_types::CompletionOptions {
    //         ..Default::default()
    //     }),
    //     definition_provider: Some(lsp_types::OneOf::Left(true)),
    //     ..Default::default()
    // };
    //
    // let initialize_data = serde_json::json!({
    //     "capabilities": server_capabilities,
    //     "serverInfo": {
    //         "name": "asm6502",
    //         "version": "0.1",
    //     }
    // });
    //
    // connection.initialize_finish(id, initialize_data)?;
    //
    // let serve = server::Server::new(connection.sender);
    //
    // for msg in &connection.receiver {
    //     serve.dispatch(msg).expect("Unable to handle response");
    // }

    Ok(())
}
