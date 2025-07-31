use crate::data::convert_uri::convert_uri;
use crate::data::files::Files;
use crate::data::symbol::Symbol;
use crate::state::State;
use codespan::FileId;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp_server::lsp_types::request::WorkDoneProgressCreate;
use tower_lsp_server::lsp_types::{
    Diagnostic, InlayHintWorkspaceClientCapabilities, ProgressToken, Uri,
    WorkDoneProgressCreateParams, WorkspaceClientCapabilities,
};
use tower_lsp_server::Client;
use uuid::Uuid;

pub struct IndexEngine {
    pub state: Arc<Mutex<State>>,
}

impl IndexEngine {
    pub fn new(state: Arc<Mutex<State>>) -> Self {
        IndexEngine { state }
    }

    pub async fn crawl_fs(slf: Arc<Mutex<IndexEngine>>, root_uri: Uri, client: Client) {
        let data = slf.lock().await;
        let token = ProgressToken::String(Uuid::new_v4().to_string());
        client
            .send_request::<WorkDoneProgressCreate>(WorkDoneProgressCreateParams {
                token: token.clone(),
            })
            .await
            .unwrap();

        let directory = url::Url::parse(root_uri.as_str())
            .unwrap()
            .to_file_path()
            .unwrap();
        let mut sources = vec![];
        let progress = client
            .progress(token, "Indexing".to_string())
            .with_percentage(0)
            .with_message("Looking for sources...".to_string())
            .begin()
            .await;

        for file in walkdir::WalkDir::new(directory).into_iter() {
            let file = file.unwrap();
            if !file.file_type().is_file() {
                continue;
            }

            if let Some("s" | "asm" | "inc" | "incs") =
                file.path().extension().and_then(OsStr::to_str)
            {
                sources.push(file);
            }
        }

        let mut state = data.state.lock().await;
        let mut diagnostics = HashMap::new();
        let mut parsed_files = vec![];

        for (idx, file) in sources.iter().enumerate() {
            let file = file.path();
            progress
                .report_with_message(
                    format!("{}/{}", idx, sources.len()),
                    ((idx as f32) / (sources.len() as f32) * 100.0) as u32,
                )
                .await;
            let uri = Uri::from_str(url::Url::from_file_path(file).unwrap().as_str()).unwrap();
            let contents = std::fs::read_to_string(file).unwrap();
            let id = state.get_or_insert_source(convert_uri(uri).unwrap(), contents);
            let file = state.files.index(id).await;
            diagnostics.insert(id, file.diagnostics);
            parsed_files.push(id);
        }

        if matches!(
            &state.client_capabilities.workspace,
            Some(WorkspaceClientCapabilities {
                inlay_hint: Some(InlayHintWorkspaceClientCapabilities {
                    refresh_support: Some(true),
                    ..
                }),
                ..
            })
        ) {
            state.client.inlay_hint_refresh().await.unwrap();
        }

        for id in parsed_files.iter() {
            let uri = state.files.get_uri(*id);
            let path = PathBuf::from_str(uri.path().as_str()).unwrap();
            if let Some(ext) = path.extension()
                && ext.to_str() == Some("s")
            {
                let (deps, dep_diagnostics) = IndexEngine::calculate_deps(&mut state.files, *id);
                diagnostics.insert(*id, dep_diagnostics);
                state.units.insert(*id, deps);
            }
        }

        let units = state.units.0.keys().cloned().collect::<Vec<_>>();
        for unit in units {
            let symbols = IndexEngine::get_symbol_tree(&mut state.files, unit);
            state.units[unit].symbols = symbols;
        }

        for id in parsed_files.iter() {
            state
                .publish_diagnostics(
                    *id,
                    diagnostics
                        .get(id)
                        .and_then(|d| Some(d.clone()))
                        .unwrap_or_default(),
                )
                .await;
        }

        progress.finish().await;
    }

    pub async fn invalidate(state: &mut State, file: FileId) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        let (resolved_imports, import_diagnostics) = state.files.resolve_import_paths(file);
        diagnostics.extend(import_diagnostics);

        diagnostics.extend(state.files.get_mut(file).lint().await);

        let file = state.files.get_mut(file);
        if resolved_imports.iter().ne(&file.resolved_includes) {
            file.resolved_includes = resolved_imports;
        }

        diagnostics
    }

    pub fn calculate_deps(files: &mut Files, file: FileId) -> (Vec<FileId>, Vec<Diagnostic>) {
        let mut deps = HashSet::new();
        let mut diagnostics = vec![];
        IndexEngine::flatten_dependencies(files, file, &mut deps, &mut diagnostics);
        if deps.contains(&file) {
            eprintln!("Circular dependency");
        }

        (deps.into_iter().collect(), diagnostics)
    }

    fn flatten_dependencies(
        files: &mut Files,
        file: FileId,
        dependencies: &mut HashSet<FileId>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let (resolved_imports, import_diagnostics) = files.resolve_import_paths(file);

        diagnostics.extend(import_diagnostics);

        for include in resolved_imports.iter() {
            if !dependencies.contains(&include.file) {
                dependencies.insert(include.file);
                Self::flatten_dependencies(files, include.file, dependencies, diagnostics);
            }
        }
    }

    pub fn get_symbol_tree(files: &mut Files, file_id: FileId) -> Vec<Symbol> {
        let mut stack = vec!["".to_owned()];
        let mut symbols = Vec::new();
        Self::get_symbols_for_file(files, file_id, &mut symbols, &mut stack);

        symbols
    }

    fn get_symbols_for_file(
        files: &mut Files,
        file_id: FileId,
        symbols: &mut Vec<Symbol>,
        stack: &mut Vec<String>,
    ) {
        let file = &files.get(file_id);
        let resolved_includes = file.resolved_includes.clone();
        let file_symbols = file.symbols.clone();

        for include in resolved_includes {
            let backup = stack.clone();
            stack.extend_from_slice(
                &include.scope[1..]
                    .iter()
                    .map(|s| s.name.to_owned())
                    .collect::<Vec<_>>(),
            );
            Self::get_symbols_for_file(files, include.file, symbols, stack);
            *stack = backup;
        }

        for symbol in file_symbols {
            let mut symbol = symbol.clone();
            let fqn = [stack.clone(), vec![symbol.fqn[2..].to_string()]]
                .concat()
                .join("::");
            symbol.fqn = fqn;
            symbols.push(symbol);
        }
    }
}
