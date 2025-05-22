use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, OnceLock},
};
use codespan::Span;
use crate::codespan::{FileId};

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

type SymCache = Vec<Symbol>;

pub struct SymbolCache(SymCache);

impl Deref for SymbolCache {
    type Target = SymCache;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SymbolCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub static SYMBOL_CACHE: OnceLock<Arc<Mutex<SymbolCache>>> = OnceLock::new();

pub fn init_symbol_cache() {
    _ = SYMBOL_CACHE.set(Arc::new(Mutex::new(SymbolCache(Vec::new()))));
}

pub fn symbol_cache_reset(file_id: FileId) {
    let mut cache = SYMBOL_CACHE
        .get()
        .expect("Symbol cache not initialized")
        .lock()
        .expect("Symbol cache mutex poisoned");

    cache.retain(|symbol| symbol.file_id != file_id);
}

pub fn symbol_cache_insert(
    file_id: FileId,
    span: Span,
    fqn: String,
    label: String,
    comment: String,
    sym_type: SymbolType,
) {
    // tracing::debug!(
    //     "Inserting symbol {:?} {} {} {}",
    //     file_id,
    //     span,
    //     label,
    //     comment
    // );
    let mut cache = SYMBOL_CACHE
        .get()
        .expect("Symbol cache not initialized")
        .lock()
        .expect("Symbol cache mutex poisoned");
    cache.push(Symbol {
        fqn,
        label,
        span,
        file_id,
        comment,
        sym_type,
    });
}

pub fn symbol_cache_fetch(label: String) -> Vec<Symbol> {
    let cache = SYMBOL_CACHE
        .get()
        .expect("Symbol cache not initialized")
        .lock()
        .expect("Symbol cache mutex poisoned");

    cache
        .iter()
        .filter_map(|sym| {
            if sym.label == label {
                return Some(sym.clone());
            }
            None
        })
        .collect()
}

pub fn symbol_cache_get() -> Vec<Symbol> {
    SYMBOL_CACHE
        .get()
        .expect("Symbol cache not initialized")
        .lock()
        .expect("Symbol cache mutex poisoned")
        .to_owned()
}
