use crate::{data::files::Files, data::units::Units};
use codespan::FileId;
use std::str::FromStr;
use std::sync::Mutex;
use lazy_static::lazy_static;
use tower_lsp_server::Client;
use tower_lsp_server::lsp_types::{
    ClientCapabilities, Diagnostic, TextDocumentContentChangeEvent, Uri,
    VersionedTextDocumentIdentifier,
};

#[derive(Debug)]
pub enum UriMode {
    VSCode,
    Helix,
}

pub struct State {
    pub files: Files,
    pub workspace_folder: Option<Uri>,
    pub client: Client,
    pub client_capabilities: ClientCapabilities,
    pub units: Units,
}

lazy_static! {
    pub static ref URI_MODE: Mutex<UriMode> = Mutex::new(UriMode::VSCode);
}

impl State {
    pub fn new(client: Client) -> Self {
        Self {
            files: Files::new(),
            workspace_folder: None,
            client,
            client_capabilities: ClientCapabilities::default(),
            units: Units::default(),
        }
    }
    pub fn get_or_insert_source(&mut self, uri: Uri, text: String) -> FileId {
        if let Some(id) = self.files.sources.get(&uri) {
            *id
        } else {
            let id = self.files.add(uri.clone(), text);
            self.files.sources.insert(uri.clone(), id);
            id
        }
    }

    pub fn reload_source(
        &mut self,
        document: &VersionedTextDocumentIdentifier,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) -> FileId {
        let id = *self.files.sources.get(&document.uri).unwrap();
        let file = &self.files.get(id);
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
        self.files.update(id, source);
        id
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

    pub fn set_workspace_folder(&mut self, workspace_folder: Uri) {
        self.workspace_folder = Some(workspace_folder);
        self.detect_uri_mode();
    }

    #[cfg(target_os = "windows")]
    fn detect_uri_mode(&mut self) {
        let root = self.workspace_folder.clone().unwrap();
        let segments = root.path().segments();
        let mut segments = segments
            .map(|segments| segments.to_string())
            .collect::<Vec<_>>();
        
        let drive_letter = segments[0].to_owned();

        if matches!(drive_letter.chars().nth(1), Some(':')) {
            // Helix mode: drive letters are C:
            *URI_MODE.lock().unwrap() = UriMode::Helix;
        } else {
            // VS Code mode: drive letters are c%3A
            *URI_MODE.lock().unwrap() = UriMode::VSCode;
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn detect_uri_mode(&mut self) {
        self.uri_mode = UriMode::VSCode; // Should work for helix on mac & linux
    }
}
