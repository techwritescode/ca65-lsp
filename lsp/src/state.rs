use std::collections::HashMap;
use std::str::FromStr;
use tower_lsp_server::Client;
use tower_lsp_server::lsp_types::{Diagnostic, DiagnosticSeverity, Range, TextDocumentContentChangeEvent, TextDocumentItem, Uri, VersionedTextDocumentIdentifier};
use analysis::{Scope, ScopeAnalyzer, Symbol, SymbolResolver};
use parser::ParseError;
use crate::codespan::{FileId, Files, IndexError};
use crate::symbol_cache::{symbol_cache_fetch, symbol_cache_insert, symbol_cache_reset, SymbolType};

pub struct State {
    pub sources: HashMap<Uri, FileId>,
    pub scopes: HashMap<Uri, Vec<Scope>>,
    pub files: Files,
    pub workspace_folder: Option<Uri>,
    pub client: Client,
}

impl State {
    pub fn get_or_insert_source(&mut self, uri: Uri, text: String) -> FileId {
        if let Some(id) = self.sources.get(&uri) {
            *id
        } else {
            let id = self.files.add(uri.clone(), text);
            self.sources.insert(uri.clone(), id);
            id
        }
    }

    pub fn reload_source(
        &mut self,
        document: &VersionedTextDocumentIdentifier,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) -> FileId {
        if let Some(id) = self.sources.get(&document.uri) {
            let mut source = self.files.source(*id).to_owned();
            for change in changes {
                if let (None, None) = (change.range, change.range_length) {
                    source = change.text;
                } else if let Some(range) = change.range {
                    let span = self
                        .files
                        .get(*id)
                        .range_to_byte_span(&range.into())
                        .unwrap_or_default();
                    source.replace_range(span, &change.text);
                }
            }
            self.files.update(*id, source);
            *id
        } else {
            // tracing::error!("attempted to reload source that does not exist");
            panic!();
        }
    }


    pub async fn parse_labels(&mut self, id: FileId) -> Vec<Diagnostic> {
        let uri = self.files.get_uri(id);

        let mut diagnostics = vec![];

        match self.files.index(id) {
            Ok(parse_errors) => {
                symbol_cache_reset(id);
                let mut analyzer = ScopeAnalyzer::new(self.files.ast(id).clone());
                let (scopes, symtab) = analyzer.analyze();
                self.scopes.insert(uri, scopes);
                // let symbols = analysis::DefAnalyzer::new(state.files.ast(id).clone()).parse();

                for (symbol, scope) in symtab.iter() {
                    symbol_cache_insert(
                        id,
                        scope.get_span(),
                        symbol.clone(),
                        scope.get_name(),
                        scope.get_description(),
                        match &scope {
                            Symbol::Macro { .. } => SymbolType::Macro,
                            Symbol::Label { .. } => SymbolType::Label,
                            Symbol::Constant { .. } => SymbolType::Constant,
                            Symbol::Parameter { .. } => SymbolType::Constant,
                            Symbol::Scope { .. } => SymbolType::Scope,
                        },
                    );
                }

                for err in parse_errors.iter() {
                    match err {
                        ParseError::UnexpectedToken(token) => {
                            diagnostics.push(Diagnostic::new_simple(
                                self
                                    .files
                                    .get(id)
                                    .byte_span_to_range(token.span)
                                    .unwrap()
                                    .into(),
                                format!("Unexpected Token {:?}", token.token_type),
                            ));
                        }
                        ParseError::Expected { expected, received } => {
                            diagnostics.push(Diagnostic::new_simple(
                                self
                                    .files
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
                            let pos = self
                                .files
                                .get(id)
                                .byte_index_to_position(self.files.get(id).source.len() - 1)
                                .unwrap();
                            diagnostics.push(Diagnostic::new_simple(
                                Range::new(pos.into(), pos.into()),
                                "Unexpected EOF".to_string(),
                            ));
                        }
                    }
                }
            }
            Err(err) => match err {
                IndexError::TokenizerError(err) => {
                    let pos = self
                        .files
                        .get(id)
                        .byte_index_to_position(err.offset)
                        .unwrap();
                    diagnostics.push(Diagnostic::new_simple(
                        Range::new(pos.into(), pos.into()),
                        "Unexpected character".to_string(),
                    ));
                }
                _ => {}
            },
        }
        // 
        // self.client
        //     .publish_diagnostics(
        //         Uri::from_str(self.files.get(id).name.as_str()).unwrap(),
        //         diagnostics,
        //         None,
        //     )
        //     .await;
        
        diagnostics
    }
    
    // TODO: store a diagnostics array for the different stages and concatenate them together
    pub async fn lint(&mut self, id: FileId) -> Vec<Diagnostic> {
        self.resolve_identifier_access(&self, id)
    }
    
    pub async fn publish_diagnostics(&mut self, id: FileId, diagnostics: Vec<Diagnostic>) {
        self.client
            .publish_diagnostics(
                Uri::from_str(self.files.get(id).name.as_str()).unwrap(),
                diagnostics,
                None,
            )
            .await;
    }

    pub fn resolve_identifier_access(&self, state: &State, id: FileId) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];
        let file = state.files.get(id);
        let identifiers = SymbolResolver::find_identifiers(state.files.ast(id).clone());

        for identifier_access in identifiers {
            let range = file
                .byte_span_to_range(identifier_access.span)
                .unwrap()
                .into();

            if identifier_access.name.starts_with("::") {
                if symbol_cache_fetch(identifier_access.name.clone()).is_empty() {
                    diagnostics.push(Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: format!("Unknown symbol: {}", identifier_access.name),
                        ..Default::default()
                    });
                }
                continue;
            }

            let mut resolved_fqn = None;

            for i in (0..=identifier_access.scope.len()).rev() {
                let scope = &identifier_access.scope[0..i];
                let fqn = [&["".to_owned()], scope, &[identifier_access.name.clone()]]
                    .concat()
                    .join("::")
                    .to_string();
                if !symbol_cache_fetch(fqn.clone()).is_empty() {
                    resolved_fqn = Some(fqn);
                    break;
                }
            }
            if resolved_fqn.is_none() {
                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("Unknown symbol: {}", identifier_access.name),
                    ..Default::default()
                });
            }
        }

        diagnostics
    }
}