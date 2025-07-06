use std::fs::{self, File};

use anyhow::{Context, anyhow, bail};
use serde::{Deserialize, Serialize};

use crate::{cli::Cli, usage_for, utils::os::permissions};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

const API_KEY_FILE_NAME: &str = "nexus/api-key";
const CONFIG_FILE_NAME: &str = "nexus/config.toml";

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_file_path = crate::config::Config::resolve_config_file_path(CONFIG_FILE_NAME)
            .ok_or_else(|| anyhow!("Failed to resolve config path"))?;

        if !config_file_path.exists() {
            bail!(
                "Nexus config could not be found, did you run '{}'?",
                usage_for!(Cli::CONNECT)
            );
        }

        let contents = fs::read_to_string(&config_file_path)
            .with_context(|| format!("Failed to read config at {}", config_file_path.display()))?;

        let mut config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config at {}", config_file_path.display()))?;

        let api_key_path = crate::config::Config::resolve_config_file_path(API_KEY_FILE_NAME)
            .ok_or_else(|| anyhow!("Failed to resolve api key path"))?;
        if let Ok(key) = fs::read_to_string(&api_key_path) {
            config.api_key = Some(key.trim().to_string());
        }

        Ok(config)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = match crate::config::Config::resolve_config_file_path(CONFIG_FILE_NAME) {
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
        let path = crate::config::Config::resolve_config_file_path(API_KEY_FILE_NAME)
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

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: Some("https://api.nexusmods.com/v1/".to_string()),
        }
    }
}
