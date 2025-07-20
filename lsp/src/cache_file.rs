use crate::analysis::scope_analyzer;
use crate::analysis::scope_analyzer::{Scope, ScopeAnalyzer};
use crate::analysis::symbol_resolver::SymbolResolver;
use crate::{
    codespan::IndexError,
    data::symbol::{Symbol, SymbolType},
};
use codespan::{File, FileId};
use parser::{Ast, ParseError, Token};
use tower_lsp_server::lsp_types::{Diagnostic, DiagnosticSeverity, Range};

type IndexResult<T> = Result<T, IndexError>;

pub struct CacheFile {
    pub id: FileId,
    pub file: File,
    pub tokens: Vec<Token>,
    pub ast: Ast,
    pub scopes: Vec<Scope>,
    pub includes: Vec<Include>,
    pub resolved_includes: Vec<ResolvedInclude>,
    pub symbols: Vec<Symbol>,
}

#[derive(Clone, Debug)]
pub struct Include {
    pub path: Token,
    pub scope: Vec<Scope>,
}

#[derive(Clone, Debug)]
pub struct ResolvedInclude {
    pub token: Token,
    pub file: FileId,
    pub scope: Vec<Scope>,
}

impl PartialEq for ResolvedInclude {
    fn eq(&self, other: &Self) -> bool {
        if self.file != other.file {
            false
        } else {
            self.scope.iter().eq(other.scope.iter())
        }
    }
}

impl CacheFile {
    pub fn new(file: File, id: FileId) -> CacheFile {
        CacheFile {
            id,
            file,
            tokens: Vec::new(),
            ast: Ast::new(),
            scopes: vec![],
            includes: vec![],
            resolved_includes: vec![],
            symbols: vec![],
        }
    }

    pub async fn parse_labels(&mut self) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        match self.index() {
            Ok(parse_errors) => {
                self.symbols.clear();
                let mut analyzer = ScopeAnalyzer::new(self.ast.clone());
                let (scopes, symtab, includes) = analyzer.analyze();
                self.scopes = scopes;
                self.includes = includes;
                // let symbols = analysis::DefAnalyzer::new(state.files.ast(id).clone()).parse();

                for (symbol, scope) in symtab.iter() {
                    self.symbols.push(Symbol {
                        fqn: symbol.clone(),
                        label: symbol.clone(),
                        span: scope.get_span(),
                        file_id: self.id,
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

                for err in parse_errors.iter() {
                    match err {
                        ParseError::UnexpectedToken(token) => {
                            diagnostics.push(Diagnostic::new_simple(
                                self.file.byte_span_to_range(token.span).unwrap().into(),
                                format!("Unexpected Token {:?}", token.token_type),
                            ));
                        }
                        ParseError::Expected { expected, received } => {
                            diagnostics.push(Diagnostic::new_simple(
                                self.file.byte_span_to_range(received.span).unwrap().into(),
                                format!(
                                    "Expected {:?} but received {:?}",
                                    expected, received.token_type
                                ),
                            ));
                        }
                        ParseError::EOF => {
                            let pos = self
                                .file
                                .byte_index_to_position(self.file.source.len() - 1)
                                .unwrap();
                            diagnostics.push(Diagnostic::new_simple(
                                Range::new(pos.into(), pos.into()),
                                "Unexpected EOF".to_string(),
                            ));
                        }
                    }
                }
            }
            Err(IndexError::TokenizerError(err)) => {
                let pos = self.file.byte_index_to_position(err.offset).unwrap();
                diagnostics.push(Diagnostic::new_simple(
                    Range::new(pos.into(), pos.into()),
                    "Unexpected character".to_string(),
                ));
            }
            _ => unreachable!(),
        }

        diagnostics
    }

    pub fn index(&mut self) -> IndexResult<Vec<ParseError>> {
        match parser::Tokenizer::new(&self.file.source, &crate::codespan::INSTRUCTIONS).parse() {
            Ok(tokens) => {
                self.tokens = tokens;

                let (ast, errors) = parser::Parser::new(&self.tokens).parse();
                self.ast = ast;

                Ok(errors)
            }
            Err(err) => Err(IndexError::TokenizerError(err)),
        }
    }

    // TODO: store a diagnostics array for the different stages and concatenate them together
    pub async fn lint(&mut self) -> Vec<Diagnostic> {
        self.resolve_identifier_access()
    }

    pub fn resolve_identifier_access(&self) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];
        let identifiers = SymbolResolver::find_identifiers(self.ast.clone());

        for identifier_access in identifiers {
            let range = self
                .file
                .byte_span_to_range(identifier_access.span)
                .unwrap()
                .into();

            if identifier_access.name.starts_with("::") {
                let m = self
                    .symbols
                    .iter()
                    .find(|Symbol { fqn, .. }| fqn == &identifier_access.name);

                if m.is_none() {
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
                let target_fqn = [&["".to_owned()], scope, &[identifier_access.name.clone()]]
                    .concat()
                    .join("::")
                    .to_string();
                let m = self
                    .symbols
                    .iter()
                    .find(|Symbol { fqn, .. }| fqn == &target_fqn);

                if m.is_some() {
                    resolved_fqn = Some(target_fqn);
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
