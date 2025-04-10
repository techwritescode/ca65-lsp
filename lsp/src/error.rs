use codespan::FileError;
use std::borrow::Cow;
use tower_lsp_server::jsonrpc::{Error, ErrorCode};

pub fn file_error_to_lsp(file_error: FileError) -> Error {
    Error {
        code: ErrorCode::InvalidParams,
        message: Cow::Owned(file_error.to_string()),
        data: None,
    }
}