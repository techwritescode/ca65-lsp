use codespan::FileId;
use std::collections::{HashMap, HashSet};
use std::ops::Index;

#[derive(Debug, Default)]
pub struct Units(HashMap<FileId, Vec<FileId>>);

impl Units {
    pub fn insert(&mut self, file: FileId, ids: Vec<FileId>) {
        self.0.insert(file, ids);
    }
    pub fn get(&self, file_id: &FileId) -> Option<&Vec<FileId>> {
        self.0.get(file_id)
    }
    pub fn find_related(&self, file_id: FileId) -> Vec<FileId> {
        // TODO: Make sure this is a hashset, and don't include self
        self.0
            .iter()
            .filter_map(|(k, v)| {
                if *k == file_id || v.contains(&file_id) {
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
    type Output = Vec<FileId>;
    fn index(&self, file_id: FileId) -> &Self::Output {
        &self.0[&file_id]
    }
}
