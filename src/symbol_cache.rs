use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, OnceLock},
    usize,
};

use crate::codespan::FileId;

#[derive(Clone, Debug)]
pub struct Symbol {
    pub file_id: FileId,
    pub label: String,
    pub line: usize,
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

pub fn symbol_cache_insert(file_id: FileId, line: usize, label: String) {
    let mut cache = SYMBOL_CACHE
        .get()
        .expect("Symbol cache not initialized")
        .lock()
        .expect("Symbol cache mutex poisoned");
    cache.push(Symbol {
        label,
        line,
        file_id,
    });
}

pub fn symbol_cache_fetch(label: String) -> Option<Symbol> {
    let cache = SYMBOL_CACHE
        .get()
        .expect("Symbol cache not initialized")
        .lock()
        .expect("Symbol cache mutex poisoned");

    match cache.iter().find(|sym| sym.label == label) {
        Some(sym) => Some(sym.clone()),
        None => None,
    }
}
