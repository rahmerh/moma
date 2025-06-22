use anyhow::{Context, bail};
use reqwest::{
    Url,
    blocking::Client,
    header::{self, HeaderMap, HeaderValue},
};
use serde::Deserialize;

use crate::sources::nexus::{Nexus, config::Config};

const NEXUS_BASE_URL: &str = "https://api.nexusmods.com/v1/";

pub struct NexusClient {
    client: Client,
}

#[derive(Deserialize)]
pub struct ValidateResponse {
    pub name: String,
    pub is_premium: bool,
    pub key: String,
}

#[derive(Debug, Deserialize)]
struct FilesResponse {
    files: Vec<File>,
}

#[derive(Debug, Deserialize)]
pub struct File {
    pub file_id: u64,
    pub category_name: Option<String>,
}

// Documentation: https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0#/
impl NexusClient {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let api_key = config
            .api_key
            .as_ref()
            .with_context(|| "Nexus API key not set")?;
        Self::with_key(api_key)
    }

    fn with_key(api_key: &str) -> anyhow::Result<Self> {
        let mut headers = HeaderMap::new();

        headers.insert("apikey", HeaderValue::from_str(api_key.trim())?);
        headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client })
    }

    pub fn validate_key(api_key: &str) -> anyhow::Result<ValidateResponse> {
        let client = Self::with_key(api_key)?;
        client.validate()
    }

    pub fn validate(&self) -> anyhow::Result<ValidateResponse> {
        let url = Url::parse(NEXUS_BASE_URL)?.join("users/validate.json")?;
        log::debug!("{}", url);

        let res = self.client.get(url).send().context("Request failed")?;

        if !res.status().is_success() {
            log::debug!("API response: {:?}", res.text().ok());
            bail!("Invalid API key or access denied");
        }

        let response: ValidateResponse = res
            .json()
            .context("Failed to deserialize validate response")?;
        Ok(response)
    }

    pub fn get_files(&self, game: &str, mod_id: &str) -> anyhow::Result<Vec<File>> {
        let nexus_domain = Nexus::resolve_nexus_domain(game).ok_or_else(|| {
            anyhow::anyhow!("Game '{}' could not be mapped to a nexus domain.", game)
        })?;

        let url_str = format!(
            "{}/games/{}/mods/{}/files.json",
            NEXUS_BASE_URL, nexus_domain, mod_id
        );

        let url = Url::parse(&url_str)?;
        let response = self.client.get(url).send().context("Request failed")?;

        let files: FilesResponse = response.json()?;

        Ok(files.files)
    }
}
