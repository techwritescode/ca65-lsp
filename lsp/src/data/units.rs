use codespan::FileId;
use std::collections::HashMap;

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
        let this = {
            if let Some(ids) = self.get(&file_id) {
                ids.clone()
            } else {
                vec![]
            }
        };

        let others: Vec<FileId> = self
            .0
            .iter()
            .filter_map(|(id, ids)| {
                if ids.contains(&file_id) {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();

        [this, others].concat()
    }
}
