use crate::data::files::Files;
use crate::state::State;
use codespan::FileId;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp_server::Client;
use tower_lsp_server::lsp_types::request::WorkDoneProgressCreate;
use tower_lsp_server::lsp_types::{
    Diagnostic, InlayHintWorkspaceClientCapabilities, ProgressToken, Uri,
    WorkDoneProgressCreateParams, WorkspaceClientCapabilities,
};
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
            let uri = Uri::from_str(url::Url::from_file_path(file).unwrap().as_ref()).unwrap();
            let contents = std::fs::read_to_string(file).unwrap();
            let id = state.get_or_insert_source(uri, contents);
            diagnostics.insert(id, state.files.index(id).await.diagnostics);
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
                let (deps, dep_diagnostics) = IndexEngine::calculate_deps(&mut state, *id);
                diagnostics.insert(*id, dep_diagnostics);
                eprintln!(
                    "{:?}: {:#?}",
                    uri.to_owned().path().to_string(),
                    deps.iter()
                        .map(|d| state.files.get_uri(*d).to_owned().path().to_string())
                        .collect::<Vec<_>>()
                );

                state.units.insert(*id, deps);
            }
        }

        for id in parsed_files.iter() {
            // let diags = IndexEngine::invalidate(&mut state, *id).await;
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
            // eprintln!("Changed {:#?}", file.resolved_includes);
            file.resolved_includes = resolved_imports;
        } else {
            // eprintln!("No changes");
        }

        diagnostics
    }

    pub fn calculate_deps(state: &mut State, file: FileId) -> (Vec<FileId>, Vec<Diagnostic>) {
        let mut deps = HashSet::new();
        let mut diagnostics = vec![];
        IndexEngine::flatten_dependencies(&mut state.files, file, &mut deps, &mut diagnostics);
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
}
