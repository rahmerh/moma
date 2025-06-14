use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub games: HashMap<String, GameConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameConfig {
    pub game_path: PathBuf,
}

impl Config {
    pub fn default_path() -> PathBuf {
        dirs_next::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("moma/config.toml")
    }

    pub fn load_or_default() -> Self {
        let path = Self::default_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path).expect("Failed to read config");
            toml::from_str(&content).expect("Failed to parse config")
        } else {
            let config = Self::default();
            config.save().expect("Failed to generate default config");
            config
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::default_path();
        let parent = path.parent().unwrap();
        std::fs::create_dir_all(parent)?;
        let toml = toml::to_string_pretty(self).unwrap();
        std::fs::write(path, toml)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            games: HashMap::new(),
        }
    }
}
