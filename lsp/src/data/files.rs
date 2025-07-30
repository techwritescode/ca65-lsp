use crate::analysis::scope_analyzer;
use crate::analysis::scope_analyzer::ScopeAnalyzer;
use crate::cache_file::{CacheFile, ResolvedInclude};
use crate::data::indexing_state::IndexingState;
use crate::data::path::diff_paths;
use crate::data::symbol::{Symbol, SymbolType};
use codespan::{File, FileId, Position};
use parser::{ParseError, Token, TokenizerError};
use path_clean::PathClean;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tower_lsp_server::lsp_types::{Diagnostic, Range, Uri};

pub enum IndexError {
    TokenizerError(TokenizerError),
    ParseError(ParseError),
}

pub struct Files {
    files: Vec<CacheFile>,
    pub sources: HashMap<Uri, FileId>,
}

impl Files {
    pub fn new() -> Self {
        Self {
            files: vec![],
            sources: HashMap::new(),
        }
    }

    pub fn get_uri_relative(&self, id: FileId, root: FileId) -> Option<String> {
        let target_uri = self.get_uri(id);
        let relative_uri = self.get_uri(root);

        let path = diff_paths(
            Path::new(&target_uri.path().to_string()),
            Path::new(&relative_uri.path().to_string()).parent()?,
        )?;

        Some(path.to_string_lossy().to_string())
    }

    pub fn add(&mut self, uri: Uri, contents: String) -> FileId {
        let file_id = FileId::new(self.files.len());
        self.files
            .push(CacheFile::new(File::new(uri.as_str(), contents), file_id));
        file_id
    }

    pub fn get(&self, id: FileId) -> &CacheFile {
        &self.files[id.get()]
    }

    pub fn get_mut(&mut self, id: FileId) -> &mut CacheFile {
        &mut self.files[id.get()]
    }

    pub fn get_uri(&self, id: FileId) -> Uri {
        Uri::from_str(self.get(id).file.name.as_str()).unwrap()
    }

    pub fn source(&self, id: FileId) -> &String {
        &self.get(id).file.source
    }

    pub fn line_tokens(&self, id: FileId, position: Position) -> Vec<Token> {
        let line_span = self.get(id).file.get_line(position.line).unwrap();
        let tokens = &self.files[id.get()].tokens;

        tokens
            .iter()
            .filter(|token| token.span.start >= line_span.start && token.span.end <= line_span.end)
            .cloned()
            .collect::<Vec<Token>>()
    }

    pub fn update(&mut self, id: FileId, source: String) {
        // tracing::info!("{}", source);
        self.get_mut(id).file.update(source)
    }

    pub fn show_instructions(&self, id: FileId, position: Position) -> bool {
        let tokens = self.line_tokens(id, position);
        let offset = self.get(id).file.position_to_byte_index(position).unwrap();
        tokens.is_empty() || tokens[0].span.end >= offset // Makes a naive guess at whether the current line contains an instruction. Doesn't work on lines with labels
    }

    pub fn resolve_import(&self, parent: FileId, path: &str) -> anyhow::Result<Option<FileId>> {
        let parent_uri = self.get_uri(parent);

        if !path.ends_with(".asm") && !path.ends_with(".s") && !path.ends_with(".inc") {
            return Ok(None);
        }

        let parent = PathBuf::from_str(parent_uri.path().as_str())?
            .parent()
            .ok_or_else(|| anyhow::anyhow!("parent folder not found"))?
            .join(path)
            .clean();
        let parent = Uri::from_str(url::Url::from_file_path(parent).unwrap().as_ref())?;

        let id = self
            .sources
            .iter()
            .find_map(|(uri, id)| if *uri == parent { Some(*id) } else { None });

        Ok(Some(id.ok_or_else(|| anyhow::anyhow!("file not found"))?))
    }

    pub fn resolve_import_paths(
        &mut self,
        parent: FileId,
    ) -> (Vec<ResolvedInclude>, Vec<Diagnostic>) {
        let mut results = vec![];
        let mut diagnostics = vec![];
        let parent_file = self.get(parent);

        for include in parent_file.includes.iter() {
            match self.resolve_import(
                parent,
                &include.path.lexeme[1..include.path.lexeme.len() - 1],
            ) {
                Ok(Some(resolved)) => results.push(ResolvedInclude {
                    file: resolved,
                    scope: include.scope.clone(),
                    token: include.path.clone(),
                }),
                Ok(None) => {}
                Err(e) => diagnostics.push(Diagnostic::new_simple(
                    parent_file
                        .file
                        .byte_span_to_range(include.path.span.into())
                        .unwrap()
                        .into(),
                    e.to_string(),
                )),
            }
        }

        (results, diagnostics)
    }

    pub fn resolve_imports_for_file(&self, parent: FileId) -> HashSet<FileId> {
        // eprintln!("Crawling {:?}", parent);
        let mut all_files = HashSet::new();
        for include in self.get(parent).resolved_includes.iter() {
            // eprintln!("Including {:?}", include);
            // eprintln!(
            //     "Including {:?} from {:?}",
            //     state.files.get_uri(include.file).as_str(),
            //     state.files.get_uri(parent).as_str()
            // );
            if !all_files.contains(&include.file) && include.file != parent {
                all_files.extend(self.resolve_imports_for_file(include.file));
            }
        }

        all_files
    }

    pub fn iter(&self) -> impl Iterator<Item = &CacheFile> {
        self.files.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut CacheFile> {
        self.files.iter_mut()
    }

    pub async fn index(&mut self, file_id: FileId) -> IndexingState {
        let mut diagnostics = vec![];
        let mut includes_changed = false;
        let parse_result = {
            let mut file = self.get_mut(file_id);
            file.parse()
        };

        let file = self.get_mut(file_id);

        if let Ok(parse_errors) = parse_result {
            diagnostics.extend_from_slice(&file.format_parse_errors(parse_errors));

            file.symbols.clear();
            let mut analyzer = ScopeAnalyzer::new(file.ast.clone());
            let (scopes, symtab, includes) = analyzer.analyze();
            file.scopes = scopes;

            for (symbol, scope) in symtab.iter() {
                file.symbols.push(Symbol {
                    fqn: symbol.clone(),
                    label: symbol.clone(),
                    span: scope.get_span(),
                    file_id: file.id,
                    comment: scope.get_description(),
                    sym_type: match &scope {
                        scope_analyzer::Symbol::Macro { .. } => SymbolType::Macro,
                        scope_analyzer::Symbol::Label { .. } => SymbolType::Label,
                        scope_analyzer::Symbol::Constant { .. } => SymbolType::Constant,
                        scope_analyzer::Symbol::Parameter { .. } => SymbolType::Constant,
                        scope_analyzer::Symbol::Scope { .. } => SymbolType::Scope,
                    },
                });
            }

            let file_includes = file.includes.clone();
            if !file_includes.iter().eq(includes.iter()) {
                file.includes = includes;

                let (resolved_imports, import_diagnostics) = self.resolve_import_paths(file_id);
                let file = self.get_mut(file_id);
                diagnostics.extend(import_diagnostics);
                file.resolved_includes = resolved_imports;
                includes_changed = true;
            }
        } else if let Err(IndexError::TokenizerError(err)) = parse_result {
            let pos = file.file.byte_index_to_position(err.offset).unwrap();
            diagnostics.push(Diagnostic::new_simple(
                Range::new(pos.into(), pos.into()),
                "Unexpected character".to_string(),
            ));
        }

        IndexingState {
            diagnostics,
            includes_changed,
        }
    }
}
