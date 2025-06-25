use crate::codespan::Files;
use codespan::FileId;
use std::collections::HashMap;
use std::str::FromStr;
use tower_lsp_server::lsp_types::{
    ClientCapabilities, Diagnostic, TextDocumentContentChangeEvent, Uri,
    VersionedTextDocumentIdentifier,
};
use tower_lsp_server::Client;

pub struct State {
    pub sources: HashMap<Uri, FileId>,
    pub files: Files,
    pub workspace_folder: Option<Uri>,
    pub client: Client,
    pub client_capabilities: ClientCapabilities,
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
            let file = &self.files.get(*id);
            let mut source = file.file.source.to_owned();
            for change in changes {
                if let (None, None) = (change.range, change.range_length) {
                    source = change.text;
                } else if let Some(range) = change.range {
                    let span = file
                        .file
                        .range_to_byte_span(&range.into())
                        .unwrap_or_default();
                    source.replace_range(span, &change.text);
                }
            }
            self.files.update(*id, source);
            *id
        } else {
            panic!();
        }
    }

    pub async fn publish_diagnostics(&mut self, id: FileId, diagnostics: Vec<Diagnostic>) {
        self.client
            .publish_diagnostics(
                Uri::from_str(self.files.get(id).file.name.as_str()).unwrap(),
                diagnostics,
                None,
            )
            .await;
    }
}
