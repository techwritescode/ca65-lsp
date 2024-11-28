use std::{error::Error, fs::File};

mod instructions;
mod text_store;
use lsp_server::{self, Connection, Message};
use lsp_types::{
    self, notification::Notification, request::Request, ClientCapabilities, Hover, HoverContents,
    InitializeParams, MarkedString, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind,
};
use text_store::TEXT_STORE;
use tracing::Level;
use tracing_subscriber::{filter, layer::SubscriberExt, Layer, Registry};

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let log = File::create("/Users/simonhochrein/Documents/asm6502-lsp/asm.log")
        .expect("failed to open log");

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_ansi(false)
        .with_writer(log)
        .with_filter(filter::LevelFilter::from_level(Level::DEBUG));

    tracing::subscriber::set_global_default(Registry::default().with(fmt_layer))?;

    text_store::init_text_store();
    instructions::init_instruction_map();

    tracing::warn!("{:#?}", instructions::INSTRUCTION_MAP);

    tracing::info!("Startup");
    let (connection, _io_threads) = Connection::stdio();

    let (id, params) = connection.initialize_start()?;

    let init_params: InitializeParams = serde_json::from_value(params).unwrap();
    // let client_capabilities: ClientCapabilities = init_params.capabilities;
    let server_capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
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

    for msg in &connection.receiver {
        tracing::info!("Message");
        match msg {
            Message::Notification(notification) => {
                tracing::error!("Notification {}", notification.method);
                if notification.method == lsp_types::notification::DidChangeTextDocument::METHOD {
                    let params: lsp_types::DidChangeTextDocumentParams =
                        serde_json::from_value(notification.params)?;
                    let uri = params.text_document.uri;
                    let text = params.content_changes[0].text.to_string();

                    TEXT_STORE
                        .get()
                        .expect("TEXT_STORE not initialized")
                        .lock()
                        .expect("text store mutex poisoned")
                        .insert(uri.to_string(), text);
                } else if notification.method
                    == lsp_types::notification::DidOpenTextDocument::METHOD
                {
                    let params: lsp_types::DidOpenTextDocumentParams =
                        serde_json::from_value(notification.params)?;
                    let uri = params.text_document.uri;
                    let text = params.text_document.text;

                    TEXT_STORE
                        .get()
                        .expect("TEXT_STORE not initialized")
                        .lock()
                        .expect("text store mutex poisoned")
                        .insert(uri.to_string(), text);
                }
            }
            Message::Request(request) => {
                tracing::error!("Request {:#?}", request);
                if request.method == lsp_types::request::HoverRequest::METHOD {
                    let params: lsp_types::HoverParams =
                        serde_json::from_value(request.params.clone())?;

                    let result = text_store::get_word_from_pos_params(
                        &params.text_document_position_params,
                    )?;

                    tracing::info!("{:#?} {}", params, result);

                    let is_6502_opcode = result.len() == 3;
                    if is_6502_opcode {
                        let candidate = instructions::INSTRUCTION_MAP
                            .get()
                            .expect("Instructions not loaded")
                            .get(&result.to_uppercase());
                        tracing::warn!("{:#?}", candidate);
                        if let Some(docs) = candidate {
                            let hover = Hover {
                                contents: HoverContents::Scalar(MarkedString::from_markdown(
                                    docs.clone(),
                                )),
                                range: None,
                            };

                            connection.sender.send(Message::Response(
                                lsp_server::Response::new_ok(request.id, &hover),
                            ))?
                        }
                    } else {
                        let hover = Hover {
                            contents: HoverContents::Scalar(MarkedString::from_markdown(
                                result.to_string(),
                            )),
                            range: None,
                        };

                        connection
                            .sender
                            .send(Message::Response(lsp_server::Response::new_ok(
                                request.id, &hover,
                            )))?
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}
