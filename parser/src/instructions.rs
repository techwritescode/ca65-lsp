use std::collections::HashMap;

pub struct Instructions {
    instructions: HashMap<String, String>
}

impl Instructions {
    pub fn load() -> Instructions {
        let instructions_raw = include_str!("../../lsp/instructions/6502.txt");
        let lines: Vec<_> = instructions_raw.lines().collect();
        let mut instructions = HashMap::new();
        for group in lines.chunks(2) {
            instructions.insert(group[0].to_string().to_lowercase(), group[1].to_string());
        }
        Instructions { instructions }
    }

    pub fn is_instruction(&self, mnemonic: &String) -> bool {
        self.instructions.contains_key(mnemonic.to_lowercase().as_str())
    }
}