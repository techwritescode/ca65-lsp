use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use tower_lsp_server::lsp_types;
use tower_lsp_server::lsp_types::{Diagnostic, Position, Uri};

#[derive(serde::Deserialize, Default, Debug)]
pub struct Toolchain {
    pub cc65: Option<String>,
}

#[derive(serde::Deserialize, Debug, Default)]
pub struct Configuration {
    #[serde(default)]
    pub toolchain: Toolchain,
}

impl Configuration {
    pub fn new(path: PathBuf) -> Self {
        if let Ok(mut file) = File::open(path.clone()) {
            let mut buf: String = "".to_owned();
            file.read_to_string(&mut buf)
                .expect("Failed to read configuration");

            toml::from_str(buf.as_str()).unwrap()
        } else {
            Configuration {
                toolchain: Toolchain::default(),
            }
        }
    }

    pub fn get_ca65_path(&self) -> Option<PathBuf> {
        if let Some(toolchain_path) = self.toolchain.cc65.clone() {
            let compiler = Path::new(toolchain_path.as_str()).join("ca65");
            return Some(compiler);
        }

        None
    }

    pub fn load(path: &Path) -> Result<Configuration, Diagnostic> {
        match File::open(path) {
            Ok(mut file) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents).expect("failed to read config file");
                match toml::from_str::<Configuration>(&contents) {
                    Ok(config) => {
                        eprintln!("Loaded configuration {config:?}");
                        Ok(config)
                    }
                    Err(error) => {
                        let range = Self::toml_range_to_lsp_range(contents, error.span().unwrap()).unwrap();
                        eprintln!("Failed to parse config file: {error:?}");
                        Err(Diagnostic::new_simple(range, error.to_string()))
                    }
                }
            }
            Err(e) => {
                Ok(Configuration::default())
            }
        }
    }

    fn toml_range_to_lsp_range(config: String, range: std::ops::Range<usize>) -> Option<lsp_types::Range> {
        let mut start_pos = None;
        let mut end_pos = None;

        let mut line = 0;
        let mut line_start = 0;

        for (idx, c) in config.char_indices() {
            if idx == range.start {
                start_pos = Some(Position::new(line, (idx - line_start) as u32));
            }
            if idx == range.end {
                end_pos = Some(Position::new(line, (idx - line_start) as u32));
                break;
            }
            
            if c == '\n' {
                line += 1;
                line_start = idx + 1;
            }
        }

        if range.end == config.len() && end_pos.is_none() {
            end_pos = Some(Position::new(line, (range.end - line_start) as u32));
        }

        Some(lsp_types::Range {
            start: start_pos?,
            end: end_pos?,
        })
    }
}
