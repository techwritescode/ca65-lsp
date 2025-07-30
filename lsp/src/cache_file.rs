use crate::analysis::scope_analyzer::Scope;
use crate::analysis::symbol_resolver::SymbolResolver;
use crate::data::files::IndexError;
use crate::data::symbol::Symbol;
use codespan::{File, FileId};
use lazy_static::lazy_static;
use parser::{Ast, Instructions, ParseError, Token};
use tower_lsp_server::lsp_types::{Diagnostic, DiagnosticSeverity, Range};

lazy_static! {
    pub static ref INSTRUCTIONS: Instructions = Instructions::load();
}

type IndexResult<T> = Result<T, IndexError>;

#[derive(Debug, Clone)]
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

impl PartialEq for Include {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.scope.iter().eq(other.scope.iter())
    }
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

    pub fn parse(&mut self) -> IndexResult<Vec<ParseError>> {
        match parser::Tokenizer::new(&self.file.source, &INSTRUCTIONS).parse() {
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

    pub fn format_parse_errors(&self, errors: Vec<ParseError>) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        for err in errors.iter() {
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

        diagnostics
    }
}
