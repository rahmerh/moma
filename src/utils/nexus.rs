use reqwest::blocking::Client;

use crate::config;

pub struct NexusClient {
    api_key: String,
    client: Client,
}

const API_KEY_FILE_NAME: &str = "nexus-api-key";

// Documentation: https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0#/
impl NexusClient {
    pub fn try_from_disk() -> anyhow::Result<Self> {
        let key_path = config::resolve_config_file_path(API_KEY_FILE_NAME)
            .ok_or_else(|| anyhow::anyhow!("Could not resolve key path"))?;

        let api_key = std::fs::read_to_string(key_path)?.trim().to_owned();

        Ok(Self {
            api_key,
            client: reqwest::blocking::Client::new(),
        })
    }

    pub fn from_key(key: String) -> Self {
        Self {
            api_key: key,
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn save_key_to_disk(&self) -> anyhow::Result<()> {
        let path = config::resolve_config_file_path(API_KEY_FILE_NAME)
            .ok_or_else(|| anyhow::anyhow!("Could not resolve key path"))?;
        std::fs::write(path, &self.api_key)?;
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
