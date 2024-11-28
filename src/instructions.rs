use std::{collections::HashMap, sync::OnceLock};

pub static INSTRUCTION_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();

pub fn init_instruction_map() {
    let instructions = include_str!("../instructions/6502.json");
    let map = serde_json::from_str::<HashMap<String, String>>(instructions).unwrap();
    _ = INSTRUCTION_MAP.set(map);
}
