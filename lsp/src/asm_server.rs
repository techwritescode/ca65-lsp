use crate::codespan::{FileId, Files, IndexError};
use crate::completion::{Ca65KeywordCompletionProvider, CompletionProvider, InstructionCompletionProvider, SymbolCompletionProvider, MacpackCompletionProvider};
use crate::configuration::{load_project_configuration, Configuration};
use crate::definition::Definition;
use crate::error::file_error_to_lsp;
use crate::symbol_cache::{
    symbol_cache_get, symbol_cache_insert, symbol_cache_reset, SymbolType,
};
use crate::documentation::{CA65_DOCUMENTATION, OPCODE_DOCUMENTATION};
use analysis::ScopeKind;
use parser::ParseError;
use std::collections::HashMap;
use std::process::Output;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp_server::lsp_types::{
    CodeActionParams, CodeActionProviderCapability, CodeActionResponse, CompletionItem,
    CompletionOptions, CompletionParams, CompletionResponse, Diagnostic, DiagnosticSeverity,
    DidChangeWorkspaceFoldersParams, DocumentSymbolParams, DocumentSymbolResponse,
    FileOperationRegistrationOptions, HoverContents, HoverProviderCapability, InitializedParams,
    MarkupContent, MarkupKind, MessageType, OneOf, Range, SymbolInformation,
    WorkspaceFileOperationsServerCapabilities, WorkspaceFoldersServerCapabilities,
    WorkspaceServerCapabilities,
};
use tower_lsp_server::{
    jsonrpc::Result,
    lsp_types::{
        DidChangeTextDocumentParams, DidOpenTextDocumentParams, GotoDefinitionParams,
        GotoDefinitionResponse, Hover, HoverParams, InitializeParams, InitializeResult, Location,
        MarkedString, ServerCapabilities, TextDocumentContentChangeEvent, TextDocumentItem,
        TextDocumentSyncCapability, TextDocumentSyncKind, Uri, VersionedTextDocumentIdentifier,
    },
    Client, LanguageServer,
};

pub struct State {
    pub sources: HashMap<Uri, FileId>,
    pub files: Files,
    pub workspace_folder: Option<Uri>
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Asm {
    client: Client,
    state: Arc<Mutex<State>>,
    configuration: Arc<Configuration>,
    completion_providers: Vec<Arc<dyn CompletionProvider + Send + Sync>>,
    definition: Definition,
}

impl Asm {
    pub fn new(client: Client) -> Self {
        let configuration = load_project_configuration();
        Asm {
            client,
            state: Arc::new(Mutex::new(State {
                sources: HashMap::new(),
                files: Files::new(),
                workspace_folder: None,
            })),
            configuration: Arc::new(configuration),
            completion_providers: vec![
                Arc::from(InstructionCompletionProvider {}),
                Arc::from(SymbolCompletionProvider {}),
                Arc::from(Ca65KeywordCompletionProvider {}),
                Arc::from(MacpackCompletionProvider {}),
            ],
            definition: Definition {},
        }
    }

    async fn index(&self, file_id: &FileId) {
        let mut state = self.state.lock().await;
        self.parse_labels(&mut state.files, *file_id).await;
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

        let line_span = files
            .get(file_id)
            .get_line(message[1].parse::<usize>().unwrap() - 1)
            .unwrap();
        let range = files.get(file_id).byte_span_to_range(line_span).unwrap();
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
        
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                definition_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions {
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
                document_symbol_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
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
        let id = get_or_insert_source(&mut state, &params.text_document);
        drop(state);

        self.index(&id).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut state = self.state.lock().await;
        let id = reload_source(&mut state, &params.text_document, params.content_changes);
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
            let definitions = self
                .definition
                .get_definition_position(
                    &state.files,
                    *id,
                    params.text_document_position_params.position.into(),
                )
                .map_err(file_error_to_lsp)?
                .unwrap_or(Vec::new());

            return Ok(Some(GotoDefinitionResponse::Array(
                definitions
                    .iter()
                    .map(|definition| {
                        let range = state
                            .files
                            .get(definition.file_id)
                            .byte_span_to_range(definition.span)
                            .unwrap()
                            .into();
                        Location::new(state.files.get_uri(definition.file_id), range)
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
                .get_word_at_position(params.text_document_position_params.position.into())
                .map_err(file_error_to_lsp)?;

            if let Some(documentation) = OPCODE_DOCUMENTATION
                .get()
                .unwrap()
                .get_doc_for_word(&word.to_lowercase())
            {
                return Ok(Some(Hover {
                    range: None,
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: documentation,
                    }),
                }));
            }

            if let Some(documentation) = CA65_DOCUMENTATION
                .get()
                .unwrap()
                .get_doc_for_word(&word.to_lowercase())
            {
                return Ok(Some(Hover {
                    range: None,
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: documentation,
                    }),
                }));
            }

            let definitions = self
                .definition
                .get_definition_position(
                    &state.files,
                    *id,
                    params.text_document_position_params.position.into(),
                )
                .map_err(file_error_to_lsp)?;

            return if let Some(definitions) = definitions {
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
            for symbol in symbol_cache_get().iter() {
                if symbol.file_id == *id {
                    let range = state
                        .files
                        .get(*id)
                        .byte_span_to_range(symbol.span)
                        .unwrap()
                        .into();

                    symbols.push(SymbolInformation {
                        name: symbol.label.clone(),
                        container_name: None,
                        kind: tower_lsp_server::lsp_types::SymbolKind::FUNCTION,
                        location: Location::new(params.text_document.uri.clone(), range),
                        tags: None,
                        deprecated: None,
                    });
                }
            }
            return Ok(Some(DocumentSymbolResponse::Flat(symbols)));
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
                let span = state
                    .files
                    .get(*id)
                    .range_to_byte_span(&range.into())
                    .unwrap_or_default();
                source.replace_range(span, &change.text);
            }
        }
        state.files.update(*id, source);
        *id
    } else {
        // tracing::error!("attempted to reload source that does not exist");
        panic!();
    }
}

impl Asm {
    async fn parse_labels(&self, files: &mut Files, id: FileId) {
        symbol_cache_reset(id);
        let mut diagnostics = vec![];

        match files.index(id) {
            Ok(()) => {
                let symbols = analysis::ScopeAnalyzer::new(files.ast(id).clone()).parse();

                for (symbol, scope) in symbols.iter() {
                    symbol_cache_insert(
                        id,
                        scope.span,
                        symbol.clone(),
                        scope.description.clone(),
                        match &scope.kind {
                            ScopeKind::Macro => SymbolType::Macro,
                            ScopeKind::Label => SymbolType::Label,
                            ScopeKind::Constant => SymbolType::Constant,
                            ScopeKind::Parameter => SymbolType::Constant,
                        },
                    );
                }
            }
            Err(err) => match err {
                IndexError::TokenizerError(err) => {
                    let pos = files.get(id).byte_index_to_position(err.offset).unwrap();
                    diagnostics.push(Diagnostic::new_simple(
                        Range::new(pos.into(), pos.into()),
                        "Unexpected character".to_string(),
                    ));
                }
                IndexError::ParseError(err) => match err {
                    ParseError::UnexpectedToken(token) => {
                        diagnostics.push(Diagnostic::new_simple(
                            files.get(id).byte_span_to_range(token.span).unwrap().into(),
                            format!("Unexpected Token {:?}", token.token_type),
                        ));
                    }
                    ParseError::Expected { expected, received } => {
                        diagnostics.push(Diagnostic::new_simple(
                            files
                                .get(id)
                                .byte_span_to_range(received.span)
                                .unwrap()
                                .into(),
                            format!(
                                "Expected {:?} but received {:?}",
                                expected, received.token_type
                            ),
                        ));
                    }
                    ParseError::EOF => {
                        let pos = files
                            .get(id)
                            .byte_index_to_position(files.get(id).source.len() - 1)
                            .unwrap();
                        diagnostics.push(Diagnostic::new_simple(
                            Range::new(pos.into(), pos.into()),
                            "Unexpected EOF".to_string(),
                        ));
                    }
                },
            },
        }

        self.client
            .publish_diagnostics(
                Uri::from_str(files.get(id).name.as_str()).unwrap(),
                diagnostics,
                None,
            )
            .await;
    }
}
