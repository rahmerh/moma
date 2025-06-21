use std::fs::{self, File};

use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};

use crate::{config, utils::os::permissions};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub username: Option<String>,
    pub is_premium: bool,
    pub api_key: Option<String>,
}

const API_KEY_FILE_NAME: &str = "nexus/api-key";
const CONFIG_FILE_NAME: &str = "nexus/config.toml";

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = match config::resolve_config_file_path(CONFIG_FILE_NAME) {
            Some(path) => path,
            None => bail!("Failed to resolve config path"),
        };

        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read config at {}: {}", path.display(), e));
            Ok(toml::from_str(&content).expect("Failed to parse config"))
        } else {
            let config = Self::default();
            config.save().expect("Failed to generate default config");
            Ok(config)
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = match config::resolve_config_file_path(CONFIG_FILE_NAME) {
            Some(path) => path,
            None => bail!("Failed to resolve config path"),
        };

        let parent = path.parent().unwrap();

        std::fs::create_dir_all(parent)?;

        let toml = toml::to_string_pretty(self).unwrap();

        fs::write(&path, toml)
            .with_context(|| format!("Could not write to '{}'", path.display()))?;
        permissions::chown_dir(&parent, true)
    }

    pub fn save_api_key(api_key: &String) -> anyhow::Result<()> {
        let path = config::resolve_config_file_path(API_KEY_FILE_NAME)
            .ok_or_else(|| anyhow::anyhow!("Could not resolve key path"))?;

        if !path.exists() {
            fs::create_dir_all(&path.parent().unwrap())?;
            File::create(&path)?;
        }

        fs::write(&path, api_key)
            .with_context(|| format!("Could not write to '{}'", path.display()))?;

        Ok(())
    }
}

pub fn api_key_exists() -> bool {
    if let Some(key_path) = config::resolve_config_file_path(API_KEY_FILE_NAME) {
        key_path.exists()
    } else {
        false
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            username: None,
            is_premium: false,
            api_key: None,
        }
    }
}
