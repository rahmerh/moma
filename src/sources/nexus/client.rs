use anyhow::{Context, bail};
use reqwest::{
    Client, Url,
    header::{self, HeaderMap, HeaderValue},
};

use crate::{
    games::Game,
    sources::nexus::{
        self,
        config::Config,
        types::{DownloadInfoRequest, DownloadInfoResponse, ModInfoResponse, ValidateResponse},
    },
};

const NEXUS_BASE_URL: &str = "https://api.nexusmods.com/v1/";

pub struct NexusClient {
    client: Client,
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

    pub async fn validate_key(api_key: &str) -> anyhow::Result<ValidateResponse> {
        let client = Self::with_key(api_key)?;
        client.validate().await
    }

    pub async fn validate(&self) -> anyhow::Result<ValidateResponse> {
        let url = Url::parse(NEXUS_BASE_URL)?.join("users/validate.json")?;

        let res = self.client.get(url).send().await?;

        if !res.status().is_success() {
            bail!("Invalid API key or access denied");
        }

        let response: ValidateResponse = res
            .json()
            .await
            .context("Failed to deserialize validate response")?;
        Ok(response)
    }

    pub async fn get_download_link(
        &self,
        request: DownloadInfoRequest,
    ) -> anyhow::Result<DownloadInfoResponse> {
        let url = Url::parse(NEXUS_BASE_URL)?
            .join("games/")?
            .join(&format!("{}/", request.game))?
            .join("mods/")?
            .join(&format!("{}/", request.mod_id))?
            .join("files/")?
            .join(&format!("{}/", request.file_id))?
            .join("download_link.json")?;

        let res = self
            .client
            .get(url)
            .query(&[("key", &request.key), ("expires", &request.expires)])
            .send()
            .await?;

        let text = res.text().await?;

        let response: Vec<DownloadInfoResponse> = serde_json::from_str(&text).map_err(|e| e)?;

        if let Some(first) = response.first() {
            Ok(first.clone())
        } else {
            anyhow::bail!("No download links returned from Nexus");
        }
    }

    pub async fn get_mod_info(&self, game: &Game, mod_id: &str) -> anyhow::Result<ModInfoResponse> {
        let url = Url::parse(NEXUS_BASE_URL)?
            .join("games/")?
            .join(&format!("{}/", nexus::to_nexus_domain(game)?))?
            .join("mods/")?
            .join(&format!("{}.json", mod_id))?;

        let res = self.client.get(url).send().await?;

        let mod_info: ModInfoResponse = res.json().await?;

        Ok(mod_info)
    }

    pub async fn download_file(
        &self,
        url: &Url,
        game: &str,
        mode_name: &str,
    ) -> anyhow::Result<()> {
        // self.client.get(url).send().await?;

        Ok(())
    }
}
