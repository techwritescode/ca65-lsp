use std::{
    collections::HashMap,
    sync::OnceLock
};

pub static CA65_KEYWORDS_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();

pub fn parse_json_to_hashmap() {
    if let Ok(hm) = serde_json::from_str::<HashMap<String, String>>(include_str!("../../data/ca65-keyword-doc.json")) {
        if !CA65_KEYWORDS_MAP.set(hm).is_err() {
            eprintln!("CA65_KEYWORDS_MAP not able to be initialized");
        }
    }
}