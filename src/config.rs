use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub paths: Paths,
    pub data: Data,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Paths {
    pub skyrim: PathBuf,
    pub proton: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            paths: Paths {
                skyrim: PathBuf::from("~/.steam/steam/steamapps/common/Skyrim Special Edition"),
                proton: PathBuf::from("~/.steam/steam/steamapps/common/Proton Hotfix"),
            },
            data: Data {
                dir: PathBuf::from("~/.moma"),
            },
        }
    }
}

impl Config {
    pub fn save_to_file(&self, path: &PathBuf) -> std::io::Result<()> {
        let toml_string = toml::to_string_pretty(self).expect("Failed to serialize config");

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, toml_string)
    }
}
