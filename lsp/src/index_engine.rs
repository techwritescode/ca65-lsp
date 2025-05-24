use crate::state::State;
use analysis::Symbol;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp_server::lsp_types::request::WorkDoneProgressCreate;
use tower_lsp_server::lsp_types::{ProgressToken, Uri, WorkDoneProgressCreateParams};
use tower_lsp_server::Client;
use uuid::Uuid;

pub struct CompilationUnit {
    root_uri: Uri,
    symbols: Vec<Symbol>,
    included_files: Vec<Uri>,
}

pub struct IndexEngine {
    pub state: Arc<Mutex<State>>,
}

impl IndexEngine {
    pub fn new(state: Arc<Mutex<State>>) -> Self {
        IndexEngine { state }
    }

    pub async fn crawl_fs(slf: Arc<Mutex<IndexEngine>>, root_uri: Uri, client: Client) {
        let slf = slf.lock().await;
        let token = ProgressToken::String(Uuid::new_v4().to_string());
        client
            .send_request::<WorkDoneProgressCreate>(WorkDoneProgressCreateParams {
                token: token.clone(),
            })
            .await
            .unwrap();

        let directory = Path::new(root_uri.path().as_str());
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

        let mut state = slf.state.lock().await;
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
            let uri = Uri::from_str(format!("file://{}", file.to_str().unwrap()).as_str()).unwrap();
            let contents = std::fs::read_to_string(file).unwrap();
            let id = state.get_or_insert_source(uri, contents);
            diagnostics.insert(id, state.parse_labels(id).await);
            parsed_files.push(id);
        }

        for id in parsed_files.iter() {
            let mut diags = diagnostics.get(id).unwrap().clone();
            diags.extend(state.lint(*id).await);
            state.publish_diagnostics(*id, diags).await;
        }

        progress.finish().await;
    }
}
