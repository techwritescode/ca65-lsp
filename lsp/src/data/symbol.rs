use codespan::{
    FileId,
    Span
};

#[derive(Clone, Copy, Debug)]
pub enum SymbolType {
    Label,
    Constant,
    Macro,
    Scope,
}

#[derive(Clone, Debug)]
pub struct Symbol {
    pub file_id: FileId,
    pub fqn: String,
    pub label: String,
    pub span: Span,
    pub comment: String,
    pub sym_type: SymbolType,
}
