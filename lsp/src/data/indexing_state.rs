use tower_lsp_server::lsp_types::Diagnostic;

pub struct IndexingState {
    pub includes_changed: bool,
    pub diagnostics: Vec<Diagnostic>,
}
