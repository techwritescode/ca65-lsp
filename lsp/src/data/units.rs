use crate::data::symbol::Symbol;
use codespan::FileId;
use std::collections::{HashMap, HashSet};
use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct Unit {
    pub deps: Vec<FileId>,
    pub symbols: Vec<Symbol>,
}

#[derive(Debug, Default)]
pub struct Units(pub HashMap<FileId, Unit>);

impl Units {
    pub fn insert(&mut self, file: FileId, ids: Vec<FileId>) {
        self.0.insert(
            file,
            Unit {
                deps: ids,
                symbols: vec![],
            },
        );
    }
    pub fn get(&self, file_id: &FileId) -> Option<&Unit> {
        self.0.get(file_id)
    }
    pub fn find_related(&self, file_id: FileId) -> Vec<FileId> {
        // TODO: Make sure this is a hashset, and don't include self
        self.0
            .iter()
            .filter_map(|(k, v)| {
                if *k == file_id || v.deps.contains(&file_id) {
                    Some(*k)
                } else {
                    None
                }
            })
            .collect::<HashSet<FileId>>()
            .into_iter()
            .collect()
    }
}

impl Index<FileId> for Units {
    type Output = Unit;
    fn index(&self, file_id: FileId) -> &Self::Output {
        &self.0[&file_id]
    }
}

impl IndexMut<FileId> for Units {
    fn index_mut(&mut self, file_id: FileId) -> &mut Unit {
        self.0.get_mut(&file_id).unwrap()
    }
}
