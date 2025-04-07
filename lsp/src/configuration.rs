use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(serde::Deserialize, Default, Debug)]
pub struct Toolchain {
    pub cc65: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
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
}

pub fn load_project_configuration() -> Configuration {
    let path = std::env::current_dir().expect("Failed to get current dir");
    let config_path = std::path::Path::new(&path).join("nes.toml");
    Configuration::new(config_path)
}
