use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Output;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use crate::codespan::{
    byte_span_to_range, get_line, get_word_at_position, position_to_byte_index, range_to_byte_span,
    FileId, Files,
};
use crate::configuration::{load_project_configuration, Configuration};
use crate::instructions;
use crate::parser::{is_identifier, parse_ident};
use crate::symbol_cache::{
    symbol_cache_fetch, symbol_cache_get, symbol_cache_insert, symbol_cache_reset, SymbolType,
};
use tempfile::NamedTempFile;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio::time;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionParams, CompletionResponse, Diagnostic, DiagnosticSeverity,
    DocumentSymbolParams, DocumentSymbolResponse, HoverContents, MessageType, OneOf,
    SymbolInformation,
};
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        DidChangeTextDocumentParams, DidOpenTextDocumentParams, GotoDefinitionParams,
        GotoDefinitionResponse, Hover, HoverParams, InitializeParams, InitializeResult, Location,
        MarkedString, Position, ServerCapabilities, TextDocumentContentChangeEvent,
        TextDocumentItem, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
        VersionedTextDocumentIdentifier,
    },
    Client, LanguageServer,
};

use streaming_iterator::StreamingIterator;

struct State {
    sources: HashMap<Url, FileId>,
    files: Files,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Asm {
    client: Client,
    state: Arc<Mutex<State>>,
    queue: Sender<FileId>,
    configuration: Arc<Configuration>,
}

impl Asm {
    pub fn new(client: Client) -> Self {
        let mut channel = tokio::sync::mpsc::channel(1);
        let configuration = load_project_configuration();
        let server = Asm {
            client,
            state: Arc::new(Mutex::new(State {
                sources: HashMap::new(),
                files: Files::new(),
            })),
            queue: channel.0,
            configuration: Arc::new(configuration),
        };
        let server2 = server.clone();
        tokio::spawn(async move {
            let duration = Duration::from_millis(800);

            let mut files_to_update: HashSet<FileId> = HashSet::new();
            let mut timed_out = false;
            loop {
                match time::timeout(duration, channel.1.recv()).await {
                    Ok(Some(file_id)) => {
                        files_to_update.insert(file_id);
                    }
                    Ok(None) => {
                        unreachable!("shouldn't happen");
                    }
                    Err(_) => {
                        timed_out = true;
                    }
                }

                if timed_out {
                    timed_out = false;
                    if files_to_update.is_empty() {
                        continue;
                    }

                    for file_id in files_to_update.iter() {
                        server2.index(file_id).await;
                    }
                    files_to_update.clear();
                }
            }
        });

        server
    }

    async fn index(&self, file_id: &FileId) {
        let state = self.state.lock().await;
        parse_labels(&state.files, *file_id);
        let orig_source = state.files.get(*file_id).name.trim_start_matches("file://");
        let orig_source = Path::new(orig_source).parent();
        let mut source = NamedTempFile::new().unwrap();
        source
            .write_all(state.files.source(*file_id).as_bytes())
            .unwrap();
        let source_path = source.path();
        let temp_path = NamedTempFile::new().unwrap();

        if let Some(compiler) = self.configuration.get_ca65_path() {
            let output = tokio::process::Command::new(compiler.to_str().unwrap())
                .args(vec![
                    source_path.to_str().unwrap(),
                    "-o",
                    temp_path.path().to_str().unwrap(),
                    "-I",
                    orig_source
                        .unwrap_or(Path::new(
                            &std::env::current_dir().expect("Failed to get current dir"),
                        ))
                        .to_str()
                        .unwrap(),
                ])
                .output()
                .await
                .unwrap();
            let mut errors = vec![];
            if !output.status.success() {
                errors.extend(
                    make_diagnostics_from_ca65_output(&state.files, *file_id, &output).await,
                );
            }
            self.client
                .publish_diagnostics(
                    Url::parse(state.files.get(*file_id).name.as_str()).unwrap(),
                    errors,
                    None,
                )
                .await;
        }
    }
}

async fn make_diagnostics_from_ca65_output(
    files: &Files,
    file_id: FileId,
    output: &Output,
) -> Vec<Diagnostic> {
    let mut diagnostics = vec![];

    for line in String::from_utf8(output.stderr.clone()).unwrap().lines() {
        let message: Vec<&str> = line.splitn(4, ":").map(|part| part.trim()).collect();

        if message.len() < 4 {
            tracing::error!("Failed to parse diagnostic {}", line);
            continue;
        }

        let line_span =
            get_line(&files, file_id, message[1].parse::<usize>().unwrap() - 1).unwrap();
        let range = byte_span_to_range(&files, file_id, line_span).unwrap();
        let severity = match message[2] {
            "Error" => Some(DiagnosticSeverity::Error),
            _ => None,
        };
        diagnostics.push(Diagnostic::new(
            range,
            severity,
            None,
            None,
            message[3].to_string(),
            None,
            None,
        ));
    }

    diagnostics
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
                completion_provider: Some(tower_lsp::lsp_types::CompletionOptions {
                    ..Default::default()
                }),
                document_symbol_provider: Some(OneOf::Left(true)),
                hover_provider: Some(tower_lsp::lsp_types::HoverProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        self.client
            .log_message(MessageType::Error, "Outline".to_string())
            .await;
        let state = self.state.lock().await;

        if let Some(id) = state.sources.get(&params.text_document.uri) {
            let mut symbols = vec![];
            for symbol in symbol_cache_get().iter() {
                if symbol.file_id == *id {
                    symbols.push(SymbolInformation {
                        name: symbol.label.clone(),
                        container_name: None,
                        kind: tower_lsp::lsp_types::SymbolKind::Function,
                        location: Location::new(
                            params.text_document.uri.clone(),
                            tower_lsp::lsp_types::Range {
                                start: Position::new(symbol.line as u32, 0),
                                end: Position::new(symbol.line as u32, symbol.label.len() as u32),
                            },
                        ),
                        tags: None,
                        deprecated: None,
                    });
                }
            }
            return Ok(Some(DocumentSymbolResponse::Flat(symbols)));
        }
        Ok(None)
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let mut state = self.state.lock().await;
        let id = get_or_insert_source(&mut state, &params.text_document);
        _ = self.queue.send(id).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut state = self.state.lock().await;
        let id = reload_source(&mut state, &params.text_document, params.content_changes);
        _ = self.queue.send(id).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let state = self.state.lock().await;

        if let Some(id) = state
            .sources
            .get(&params.text_document_position_params.text_document.uri)
        {
            let position = params.text_document_position_params.position;
            let word =
                get_word_at_position(&state.files, *id, position).expect("Word out of bounds");

            let mut symbols = symbol_cache_fetch(word.to_string());
            symbols.sort_by(|sym, _| {
                if sym.file_id == *id {
                    return Ordering::Less;
                }
                Ordering::Equal
            });
            let documentation = symbols
                .first()
                .map_or(None, |symbol| {
                    Some(format!("```ca65\n{}\n```", symbol.comment.clone()))
                })
                .map(MarkedString::from_markdown);
            return Ok(documentation.map_or(None, |doc| {
                Some(Hover {
                    range: None,
                    contents: HoverContents::Scalar(doc),
                })
            }));
        }

        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let state = self.state.lock().await;

        if let Some(id) = state
            .sources
            .get(&params.text_document_position_params.text_document.uri)
        {
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&tree_sitter_ca65::LANGUAGE.into());
            let tree = parser
                .parse(state.files.get(*id).source.as_bytes(), None)
                .unwrap();

            let byte = position_to_byte_index(
                &state.files,
                *id,
                params.text_document_position_params.position,
            )
            .unwrap();

            let mut node = tree.root_node();
            loop {
                let child = node.first_named_child_for_byte(byte);
                if let Some(child) = child {
                    node = child;
                } else {
                    tracing::error!("{}", node.to_sexp());
                    break;
                }
            }

            let word = get_word_at_position(
                &state.files,
                *id,
                params.text_document_position_params.position,
            )
            .unwrap_or_else(|_| {
                tracing::error!("Failed to get word");
                panic!();
            });

            let mut definitions = symbol_cache_fetch(word.to_string());

            tracing::error!("{} {:#?}", word, definitions);

            definitions.sort_by(|sym, _| {
                if sym.file_id == *id {
                    return Ordering::Less;
                }
                Ordering::Equal
            });

            return Ok(Some(GotoDefinitionResponse::Array(
                definitions
                    .iter()
                    .map(|definition| {
                        let source_file =
                            Url::parse(state.files.get(definition.file_id).name.as_str()).unwrap();
                        Location::new(
                            source_file,
                            tower_lsp::lsp_types::Range {
                                start: Position::new(definition.line as u32, 0),
                                end: Position::new(definition.line as u32, word.len() as u32),
                            },
                        )
                    })
                    .collect(),
            )));
        }

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let mut completion_items: Vec<CompletionItem> = vec![];
        for (opcode, description) in instructions::INSTRUCTION_MAP
            .get()
            .expect("Instructions not loaded")
            .iter()
        {
            completion_items.push(CompletionItem::new_simple(
                opcode.to_lowercase().to_owned(),
                description.to_owned(),
            ));
        }
        for symbol in symbol_cache_get().iter() {
            completion_items.push(CompletionItem::new_simple(
                symbol.label.to_owned(),
                "".to_owned(),
            ));
        }
        Ok(Some(CompletionResponse::Array(completion_items)))
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

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_ca65::LANGUAGE.into())
        .expect("Failed to load grammar");

    let source = files.get(id).source.as_bytes();
    let mut tree = parser.parse(source, None).unwrap();

    let query = tree_sitter::Query::new(
        &tree_sitter_ca65::LANGUAGE.into(),
        "(label (identifier) @name) (constant (identifier) @name) @constant",
    )
    .expect("Failed to build query");

    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source);

    while let Some(m) = matches.next() {
        tracing::error!("{:#?}", m);
        match m.pattern_index {
            0 => {
                let name = m.captures[0].node.utf8_text(source).unwrap().to_string();
                symbol_cache_insert(
                    id,
                    m.captures[0].node.range().start_point.row,
                    name.clone(),
                    name + ":",
                    SymbolType::Label,
                );
            }
            1 => {
                let name = m.nodes_for_capture_index(0).next().unwrap();
                let name_str = name.utf8_text(source).unwrap().to_string();
                let constant = m.nodes_for_capture_index(1).next().unwrap();
                symbol_cache_insert(
                    id,
                    name.range().start_point.row,
                    name_str,
                    constant.utf8_text(source).unwrap().to_string(),
                    SymbolType::Constant,
                );
            }
            _ => {}
        }
    }
}
