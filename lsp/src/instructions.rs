use std::{collections::HashMap, sync::OnceLock};
use regex;

pub static INSTRUCTION_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();
pub static FULL_INSTRUCTION_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();

pub fn init_instruction_map() {
    let instructions = include_str!("../instructions/6502.json");
    let map = serde_json::from_str::<HashMap<String, String>>(instructions).unwrap();
    _ = INSTRUCTION_MAP.set(map);
}

pub fn init_full_instruction_map() {
    let instructions_markdown = include_str!("../instructions/65816-opcodes.md");
    let instr_re = regex::Regex::new(r#"\{([a-z]{3})\}"#).unwrap();
    let section_re = regex::Regex::new(r#"(?s)\{:\}\n(.*?)\{\.\}"#).unwrap();

    let mut instruction_map: HashMap<String, String> = HashMap::new();

    let descriptions: Vec<&str> = section_re
        .captures_iter(instructions_markdown)
        .map(|caps| caps.get(1).unwrap().as_str())
        .collect();
    let mut betweens = section_re
        .split(instructions_markdown);

    descriptions.iter().for_each(|desc| {
        instr_re
            .captures_iter(betweens.next().unwrap())
            .map(|caps| caps.get(1).unwrap().as_str())
            .for_each(|instr| {
                instruction_map.insert(instr.to_string(), desc.to_string());
            });
    });

    _ = FULL_INSTRUCTION_MAP.set(instruction_map);
}