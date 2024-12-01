use std::{collections::HashMap, error::Error, fs::File};

use codespan::{
    byte_span_to_range, get_line, get_line_source, get_word_at_position, range_to_byte_span,
    FileId, Files,
};
mod instructions;
mod parser;
// mod server;
mod symbol_cache;
use symbol_cache::{symbol_cache_fetch, symbol_cache_insert, symbol_cache_reset};
// mod text_store;
use tokio::sync::Mutex;
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        Diagnostic, DidChangeTextDocumentParams, DidOpenTextDocumentParams, GotoDefinitionParams,
        GotoDefinitionResponse, Hover, HoverParams, InitializeParams, InitializeResult, Location,
        MarkedString, Position, ServerCapabilities, TextDocumentContentChangeEvent,
        TextDocumentItem, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
        VersionedTextDocumentIdentifier,
    },
    Client, LanguageServer, LspService, Server,
};
use tracing::Level;
use tracing_subscriber::{filter, layer::SubscriberExt, Layer, Registry};

mod codespan;

use parser::{is_identifier, parse_ident};

struct State {
    sources: HashMap<Url, FileId>,
    files: Files,
}

struct Asm {
    client: Client,
    state: Mutex<State>,
}

impl Asm {
    fn new(client: Client) -> Self {
        Asm {
            client,
            state: Mutex::new(State {
                sources: HashMap::new(),
                files: Files::new(),
            }),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Asm {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::Incremental,
                )),
                definition_provider: Some(tower_lsp::lsp_types::OneOf::Left(true)),
                hover_provider: Some(tower_lsp::lsp_types::HoverProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let mut state = self.state.lock().await;
        tracing::info!("{:#?}", params);
        let id = get_or_insert_source(&mut state, &params.text_document);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut state = self.state.lock().await;
        let id = reload_source(&mut state, &params.text_document, params.content_changes);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let mut state = self.state.lock().await;

        if let Some(id) = state
            .sources
            .get(&params.text_document_position_params.text_document.uri)
        {
            let line_index = params.text_document_position_params.position.line;
            let span = get_line(&state.files, *id, line_index as usize).unwrap_or_else(|_| {
                tracing::error!("Failed here");
                panic!();
            });
            let line = byte_span_to_range(&state.files, *id, span).unwrap_or_else(|_| {
                tracing::error!("Failed here 2");
                panic!();
            });

            let word = get_word_at_position(
                &state.files,
                *id,
                params.text_document_position_params.position,
            )
            .unwrap();

            return Ok(Some(Hover {
                range: Some(line),
                contents: tower_lsp::lsp_types::HoverContents::Scalar(MarkedString::from_markdown(
                    word.to_string(),
                )),
            }));
        }

        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let mut state = self.state.lock().await;

        if let Some(id) = state
            .sources
            .get(&params.text_document_position_params.text_document.uri)
        {
            parse_labels(&state.files, *id);
            let word = get_word_at_position(
                &state.files,
                *id,
                params.text_document_position_params.position,
            )
            .unwrap_or_else(|_| {
                tracing::error!("Failed to get word");
                panic!();
            });

            if let Some(symbol) = symbol_cache_fetch(word.to_string()) {
                return Ok(Some(GotoDefinitionResponse::Scalar(Location::new(
                    params.text_document_position_params.text_document.uri,
                    tower_lsp::lsp_types::Range {
                        start: Position::new(symbol.line as u32, 0),
                        end: Position::new(symbol.line as u32, word.len() as u32),
                    },
                ))));
            }
        }

        Ok(None)
    }
}

fn get_or_insert_source(state: &mut State, document: &TextDocumentItem) -> FileId {
    if let Some(id) = state.sources.get(&document.uri) {
        *id
    } else {
        let id = state.files.add(document.uri.clone(), document.text.clone());
        state.sources.insert(document.uri.clone(), id);
        id
    }
}

fn reload_source(
    state: &mut State,
    document: &VersionedTextDocumentIdentifier,
    changes: Vec<TextDocumentContentChangeEvent>,
) -> FileId {
    if let Some(id) = state.sources.get(&document.uri) {
        let mut source = state.files.source(*id).to_owned();
        for change in changes {
            if let (None, None) = (change.range, change.range_length) {
                source = change.text;
            } else if let Some(range) = change.range {
                let span = range_to_byte_span(&state.files, *id, &range).unwrap_or_default();
                source.replace_range(span, &change.text);
            }
        }
        state.files.update(*id, source);
        *id
    } else {
        tracing::error!("attempted to reload source that does not exist");
        panic!();
    }
}

fn parse_labels(files: &Files, id: FileId) {
    symbol_cache_reset(id);

    for (i, line) in files.get(id).source.lines().enumerate() {
        if line.is_empty() {
            continue;
        }

        let split: Vec<&str> = line.split_whitespace().collect();

        if split.len() > 0 {
            let maybe_ident = split[0];
            if is_identifier(maybe_ident)
                && instructions::INSTRUCTION_MAP
                    .get()
                    .expect("Instructions not loaded")
                    .get(&maybe_ident.to_uppercase())
                    .is_none()
            {
                let parsed_ident = parse_ident(maybe_ident);
                symbol_cache_insert(id, i, parsed_ident.clone());
            }
        }
    }
}

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
