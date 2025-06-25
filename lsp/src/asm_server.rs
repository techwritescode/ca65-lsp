use crate::cache_file::CacheFile;
use crate::codespan::Files;
use crate::completion::{
    Ca65DotOperatorCompletionProvider, Ca65KeywordCompletionProvider, CompletionProvider,
    FeatureCompletionProvider, InstructionCompletionProvider, MacpackCompletionProvider,
    SymbolCompletionProvider,
};
use crate::data::configuration::Configuration;
use crate::definition::Definition;
use crate::documentation::DOCUMENTATION_COLLECTION;
use crate::error::file_error_to_lsp;
use crate::index_engine::IndexEngine;
use crate::state::State;
use analysis::Scope;
use codespan::FileId;
use codespan::{File, Span};
use std::collections::HashMap;
use std::path::Path;
use std::process::Output;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp_server::lsp_types::{
    ClientCapabilities, CodeActionParams, CodeActionProviderCapability, CodeActionResponse,
    CompletionItem, CompletionOptions, CompletionParams, CompletionResponse, Diagnostic,
    DiagnosticSeverity, DidChangeWatchedFilesParams, DidChangeWorkspaceFoldersParams,
    DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, FileOperationRegistrationOptions,
    FoldingRange, FoldingRangeParams, FoldingRangeProviderCapability, HoverContents,
    HoverProviderCapability, InitializedParams, InlayHint, InlayHintLabel, InlayHintParams,
    LocationLink, MarkupContent, MarkupKind, MessageType, OneOf, Registration, SymbolKind,
    WorkspaceFileOperationsServerCapabilities, WorkspaceFoldersServerCapabilities,
    WorkspaceServerCapabilities,
};
use tower_lsp_server::{
    jsonrpc::Result,
    lsp_types::{
        DidChangeTextDocumentParams, DidOpenTextDocumentParams, GotoDefinitionParams,
        GotoDefinitionResponse, Hover, HoverParams, InitializeParams, InitializeResult,
        MarkedString, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
    },
    Client, LanguageServer,
};

#[allow(dead_code)]
pub struct Asm {
    client: Client,
    state: Arc<Mutex<State>>,
    configuration: Arc<Mutex<Configuration>>,
    completion_providers: Vec<Arc<dyn CompletionProvider + Send + Sync>>,
    definition: Definition,
    index_engine: Arc<Mutex<IndexEngine>>,
}

impl Asm {
    pub fn new(client: Client) -> Self {
        let state = Arc::new(Mutex::new(State {
            sources: HashMap::new(),
            files: Files::new(),
            workspace_folder: None,
            client: client.clone(),
            client_capabilities: ClientCapabilities::default(),
        }));
        Asm {
            client,
            state: state.clone(),
            configuration: Arc::new(Mutex::new(Configuration::default())),
            completion_providers: vec![
                Arc::from(SymbolCompletionProvider {}),
                Arc::from(InstructionCompletionProvider {}),
                Arc::from(Ca65KeywordCompletionProvider {}),
                Arc::from(Ca65DotOperatorCompletionProvider {}),
                Arc::from(MacpackCompletionProvider {}),
                Arc::from(FeatureCompletionProvider {}),
            ],
            definition: Definition {},
            index_engine: Arc::new(Mutex::new(IndexEngine::new(state.clone()))),
        }
    }

    async fn index(&self, file_id: &FileId) {
        let mut state = self.state.lock().await;
        let file = state.files.get_mut(*file_id);
        let diagnostics = [file.parse_labels().await, file.lint().await].concat();
        state.publish_diagnostics(*file_id, diagnostics).await;
    }

    async fn load_config(&self, path: &Path) -> Result<()> {
        let uri_str = "file://".to_owned() + path.to_str().unwrap();
        let uri = Uri::from_str(&uri_str).unwrap();

        match Configuration::load(path) {
            Ok(configuration) => {
                *self.configuration.lock().await = configuration;
                self.client.publish_diagnostics(uri, vec![], None).await;
            }
            Err(diagnostic) => {
                self.client
                    .publish_diagnostics(uri, vec![diagnostic], None)
                    .await;
            }
        }

        Ok(())
    }
}

#[allow(dead_code)]
async fn make_diagnostics_from_ca65_output(
    files: &Files,
    file_id: FileId,
    output: &Output,
) -> Vec<Diagnostic> {
    let mut diagnostics = vec![];

    for line in String::from_utf8(output.stderr.clone()).unwrap().lines() {
        let message: Vec<&str> = line.splitn(4, ":").map(|part| part.trim()).collect();

        if message.len() < 4 {
            // tracing::error!("Failed to parse diagnostic {}", line);
            continue;
        }

        let file = &files.get(file_id).file;

        let line_span = file
            .get_line(message[1].parse::<usize>().unwrap() - 1)
            .unwrap();
        let range = file.byte_span_to_range(line_span).unwrap();
        let severity = match message[2] {
            "Error" => Some(DiagnosticSeverity::ERROR),
            _ => None,
        };
        diagnostics.push(Diagnostic::new(
            range.into(),
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

impl LanguageServer for Asm {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let mut state = self.state.lock().await;
        if let Some(workspace_folders) = params.workspace_folders {
            if !workspace_folders.is_empty() {
                state.workspace_folder = Some(workspace_folders.first().unwrap().clone().uri)
            }
        }

        state.client_capabilities = params.capabilities.clone();

        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                definition_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string()]),
                    ..Default::default()
                }),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                workspace: Some(WorkspaceServerCapabilities {
                    file_operations: Some(WorkspaceFileOperationsServerCapabilities {
                        did_create: Some(FileOperationRegistrationOptions::default()),
                        ..Default::default()
                    }),
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                }),
                inlay_hint_provider: Some(OneOf::Left(true)),
                folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        let registration = Registration {
            id: "config-watcher".to_string(),
            method: "workspace/didChangeWatchedFiles".to_string(),
            register_options: Some(serde_json::json!({
                "watchers": [
                    {
                        "globPattern": "**/ca65.toml",
                        "kind": 7, // 0b00000111 for Create, Write, and Delete
                    }
                ]
            })),
        };

        self.client
            .register_capability(vec![registration])
            .await
            .unwrap();

        let folder = self.state.lock().await.workspace_folder.clone();
        if let Some(workspace_folder) = folder {
            let config_path = Path::new(workspace_folder.path().as_str()).join("ca65.toml");
            if config_path.exists() {
                self.load_config(&config_path)
                    .await
                    .expect("Failed to read config");
            }

            tokio::spawn(IndexEngine::crawl_fs(
                self.index_engine.clone(),
                workspace_folder,
                self.client.clone(),
            ))
            .await
            .unwrap();
        }
        // _ = self
        //     .client
        //     .progress(ProgressToken::String("load".to_string()), "Loading")
        //     .with_message("Indexing")
        //     .with_percentage(50)
        //     .begin()
        //     .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let mut state = self.state.lock().await;
        let id = state.get_or_insert_source(params.text_document.uri, params.text_document.text);
        drop(state);

        self.index(&id).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut state = self.state.lock().await;
        let id = state.reload_source(&params.text_document, params.content_changes);
        drop(state);

        self.index(&id).await;
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
            let (definitions, span) = self
                .definition
                .get_definition_position(
                    &state,
                    *id,
                    params.text_document_position_params.position.into(),
                )
                .map_err(file_error_to_lsp)?
                .unwrap_or((Vec::new(), Span::new(0, 0)));

            return Ok(Some(GotoDefinitionResponse::Link(
                definitions
                    .iter()
                    .map(|definition| {
                        let range = state
                            .files
                            .get(definition.file_id)
                            .file
                            .byte_span_to_range(definition.span)
                            .unwrap()
                            .into();
                        let source_range = state
                            .files
                            .get(*id)
                            .file
                            .byte_span_to_range(span)
                            .unwrap()
                            .into();

                        LocationLink {
                            origin_selection_range: Some(source_range),
                            target_uri: state.files.get_uri(definition.file_id),
                            target_range: range,
                            target_selection_range: range,
                        }
                    })
                    .collect(),
            )));
        }

        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let state = self.state.lock().await;

        if let Some(id) = state
            .sources
            .get(&params.text_document_position_params.text_document.uri)
        {
            let word = state
                .files
                .get(*id)
                .file
                .get_word_at_position(params.text_document_position_params.position.into())
                .map_err(file_error_to_lsp)?;

            // TODO: take context into account when choosing to show hover doc
            for (_doc_kind, doc) in DOCUMENTATION_COLLECTION.get().unwrap() {
                if let Some(doc) = doc.get_doc_for_word(&word.to_lowercase()) {
                    return Ok(Some(Hover {
                        range: None,
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: doc,
                        }),
                    }));
                }
            }

            let definitions = self
                .definition
                .get_definition_position(
                    &state,
                    *id,
                    params.text_document_position_params.position.into(),
                )
                .map_err(file_error_to_lsp)?;

            return if let Some((definitions, _span)) = definitions {
                let documentation = definitions
                    .first()
                    .map(|symbol| format!("```ca65\n{}\n```", symbol.comment.clone()))
                    .map(MarkedString::from_markdown);
                Ok(documentation.map(|doc| Hover {
                    range: None,
                    contents: HoverContents::Scalar(doc),
                }))
            } else {
                Ok(None)
            };
        }

        Ok(None)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let state = self.state.lock().await;

        if let Some(id) = state.sources.get(&params.text_document.uri) {
            let mut symbols = vec![];
            let file = state.files.get(*id);

            fn scope_to_symbol(scope: &Scope, file: &CacheFile) -> DocumentSymbol {
                let range = file.file.byte_span_to_range(scope.span).unwrap().into();
                DocumentSymbol {
                    name: scope.name.clone(),
                    detail: None,
                    kind: SymbolKind::NAMESPACE,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: {
                        let children: Vec<DocumentSymbol> = scope
                            .children
                            .iter()
                            .map(|child| scope_to_symbol(child, file))
                            .collect();
                        if children.is_empty() {
                            None
                        } else {
                            Some(children)
                        }
                    },
                }
            }

            for symbol in file.scopes.iter() {
                symbols.push(scope_to_symbol(symbol, file));
            }
            return Ok(Some(DocumentSymbolResponse::Nested(symbols)));
        }
        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let state = self.state.lock().await;

        if let Some(id) = state
            .sources
            .get(&params.text_document_position.text_document.uri)
        {
            let mut completion_items: Vec<CompletionItem> = vec![];
            for provider in self.completion_providers.iter() {
                completion_items.extend(provider.completions_for(
                    &state,
                    *id,
                    params.text_document_position.position.into(),
                ));
            }
            Ok(Some(CompletionResponse::Array(completion_items)))
        } else {
            Ok(None)
        }
    }

    async fn code_action(&self, _params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        // self.client
        //     .log_message(
        //         MessageType::INFO,
        //         format!("Uri: {}", params.text_document.uri.as_str()),
        //     )
        //     .await;
        // let config_uri = Uri::from_str(
        //     "file:///home/simonhochrein/Documents/ca65-lsp/project.toml",
        // )
        //     .unwrap();
        //
        // Ok(Some(vec![CodeActionOrCommand::CodeAction(CodeAction {
        //     title: "Create workspace file".to_string(),
        //     edit: Some(WorkspaceEdit {
        //         document_changes: Some(DocumentChanges::Operations(vec![
        //             DocumentChangeOperation::Op(ResourceOp::Create(CreateFile {
        //                 uri: config_uri.clone(),
        //                 annotation_id: None,
        //                 options: Some(CreateFileOptions {
        //                     overwrite: Some(false),
        //                     ignore_if_exists: Some(true),
        //                 }),
        //             })),
        //             DocumentChangeOperation::Edit(TextDocumentEdit {
        //                 text_document: OptionalVersionedTextDocumentIdentifier {
        //                     uri: config_uri.clone(),
        //                     version: None,
        //                 },
        //                 edits: vec![OneOf::Left(TextEdit::new(
        //                     Range::new(Position::new(0, 0), Position::new(0, 0)),
        //                     "[compiler]\n".to_owned(),
        //                 ))],
        //             }),
        //         ])),
        //         ..Default::default()
        //     }),
        //     // diagnostics: Some(vec![
        //     //     Diagnostic::new(
        //     //         Range::new(Position::new(0, 0), Position::new(0, 1)),
        //     //         Some(DiagnosticSeverity::WARNING),
        //     //         None,
        //     //         None,
        //     //         "Requires folder based project".to_owned(),
        //     //         None,
        //     //         None,
        //     //     )
        //     // ]),
        //     // command: Some(Command::new("Open file".to_string(), "vscode.open".to_string(), Some(vec![
        //     //     serde_json::json!(config_uri.to_string())
        //     // ]))),
        //     kind: Some(CodeActionKind::QUICKFIX),
        //     ..Default::default()
        // })]))
        Ok(None)
    }
    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        self.client
            .log_message(MessageType::INFO, format!("Config: {:#?}", params))
            .await;
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        if let Some(event) = params.changes.first() {
            self.load_config(Path::new(event.uri.path().as_str()))
                .await
                .expect("load_config failed");
        }
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        let state = self.state.lock().await;

        if let Some(id) = state.sources.get(&params.text_document.uri) {
            let file = &state.files.get(*id);
            Ok(Some(
                file.scopes
                    .iter()
                    .flat_map(|scope| scope_to_folding_range(&file.file, scope))
                    .collect(),
            ))
        } else {
            Ok(None)
        }
    }
    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let state = self.state.lock().await;

        if let Some(id) = state.sources.get(&params.text_document.uri) {
            let file = &state.files.get(*id);
            Ok(Some(
                file.scopes
                    .iter()
                    .flat_map(|scope| scope_to_inlay_hint(&file.file, scope))
                    .collect(),
            ))
        } else {
            Ok(None)
        }
    }
}

fn scope_to_folding_range(file: &File, scope: &Scope) -> Vec<FoldingRange> {
    let range = file.byte_span_to_range(scope.span).unwrap();
    let mut results = vec![FoldingRange {
        start_line: range.start.line as u32,
        start_character: None,
        end_line: (range.end.line - 1) as u32,
        end_character: None,
        kind: None,
        collapsed_text: None,
    }];

    results.extend(
        scope
            .children
            .iter()
            .flat_map(|scope| scope_to_folding_range(file, scope)),
    );

    results
}

fn scope_to_inlay_hint(file: &File, scope: &Scope) -> Vec<InlayHint> {
    let range = file.byte_span_to_range(scope.span).unwrap();
    let mut results = vec![InlayHint {
        position: range.end.into(),
        label: InlayHintLabel::String(scope.name.clone()),
        kind: None,
        text_edits: None,
        tooltip: None,
        padding_left: Some(true),
        padding_right: None,
        data: None,
    }];

    results.extend(
        scope
            .children
            .iter()
            .flat_map(|scope| scope_to_inlay_hint(file, scope)),
    );

    results
}
