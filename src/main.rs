use std::{error::Error, fs::File};

mod instructions;
mod server;
mod symbol_cache;
mod text_store;
use lsp_server::Connection;
use lsp_types::{self, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind};
use tracing::Level;
use tracing_subscriber::{filter, layer::SubscriberExt, Layer, Registry};

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let current_path = std::env::current_dir().expect("Failed to get working directory");
    let log_path = std::path::Path::new(&current_path).join("asm.log");
    let log = File::create(log_path).expect("failed to open log");

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_ansi(false)
        .with_writer(log)
        .with_filter(filter::LevelFilter::from_level(Level::DEBUG));

    tracing::subscriber::set_global_default(Registry::default().with(fmt_layer))?;

    text_store::init_text_store();
    symbol_cache::init_symbol_cache();
    instructions::init_instruction_map();

    tracing::warn!("{:#?}", instructions::INSTRUCTION_MAP);

    tracing::info!("Startup");
    let (connection, _io_threads) = Connection::stdio();

    let (id, _) = connection.initialize_start()?;

    let server_capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
        completion_provider: Some(lsp_types::CompletionOptions {
            ..Default::default()
        }),
        definition_provider: Some(lsp_types::OneOf::Left(true)),
        ..Default::default()
    };

    let initialize_data = serde_json::json!({
        "capabilities": server_capabilities,
        "serverInfo": {
            "name": "asm6502",
            "version": "0.1",
        }
    });

    connection.initialize_finish(id, initialize_data)?;

    let serve = server::Server::new(connection.sender);

    for msg in &connection.receiver {
        serve.dispatch(msg).expect("Unable to handle response");
    }

    Ok(())
}
