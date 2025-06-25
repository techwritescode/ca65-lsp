use crate::codespan::Files;
use crate::data::symbol::Symbol;
use codespan::FileId;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use tower_lsp_server::lsp_types::Uri;

pub struct IncludeResolver {
    pub included: HashSet<FileId>,
    pub scope_stack: Vec<String>,
    pub symbols: Vec<Symbol>,
}

impl IncludeResolver {
    pub fn new() -> Self {
        IncludeResolver {
            included: HashSet::new(),
            scope_stack: Vec::new(),
            symbols: Vec::new(),
        }
    }

    pub fn resolve_include_tree(
        &mut self,
        files: &Files,
        sources: &HashMap<Uri, FileId>,
        file_id: FileId,
    ) {
        let uri = files.get_uri(file_id);
        let url = url::Url::parse(uri.as_str()).unwrap();
        self.included.insert(file_id);

        match url.to_file_path() {
            Ok(path) => {
                let parent = path.parent().unwrap();
                let file = files.get(file_id);

                for symbol in file.symbols.iter() {
                    self.symbols.push(Symbol {
                        file_id,
                        fqn: self.scope_stack.join("::") + symbol.fqn.as_str(),
                        label: symbol.label.clone(),
                        span: symbol.span,
                        comment: symbol.comment.clone(),
                        sym_type: symbol.sym_type,
                    })
                }

                for include in file.includes.iter() {
                    let name = &include.file[1..include.file.len() - 1];
                    if let Ok(path) = parent.join(name).canonicalize() {
                        let uri = Uri::from_str(url::Url::from_file_path(path).unwrap().as_ref())
                            .unwrap();
                        let source = sources.get(&uri);

                        match source {
                            Some(source) => {
                                if self.included.contains(source) {
                                    // TODO: Add error reporting
                                    eprintln!("Circular dependency: {:?}", include);
                                } else {
                                    self.included.insert(*source);
                                    self.scope_stack.push(
                                        include.scope[1..]
                                            .iter()
                                            .map(|scope| scope.name.clone())
                                            .collect::<Vec<String>>()
                                            .join("::"),
                                    );
                                    self.resolve_include_tree(files, sources, *source);
                                    self.scope_stack.pop();
                                }
                            }
                            None => {
                                // TODO: Same as above
                                eprintln!("Failed to find file {:?}", uri);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                panic!("{e:?}");
            }
        }
    }
}
