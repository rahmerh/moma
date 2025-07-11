use anyhow::{Context, bail};
use futures::TryStreamExt;
use reqwest::{
    Client, Url,
    header::{self, HeaderMap, HeaderValue},
};
use std::path::PathBuf;
use tokio_util::io::StreamReader;

use crate::{
    games::Game,
    mods::download_tracker::DownloadTracker,
    sources::nexus::{
        self,
        config::Config,
        types::{
            DownloadInfoRequest, DownloadInfoResponse, ModFileInfoResponse, ModInfoResponse,
            ValidateResponse,
        },
    },
};

pub struct NexusClient {
    client: Client,
    download_tracker: DownloadTracker,
    base_url: Url,
}

pub const DEFAULT_NEXUS_BASE_URL: &str = "https://api.nexusmods.com/v1/";

// Documentation: https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0#/
impl NexusClient {
    pub fn new(config: &Config, download_tracker: DownloadTracker) -> anyhow::Result<Self> {
        let api_key = config
            .api_key
            .as_ref()
            .with_context(|| "Nexus API key not set")?;

        let client = Self::create_client_with_api_key(api_key)?;
        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| DEFAULT_NEXUS_BASE_URL.to_string());

        Ok(Self {
            client,
            base_url: Url::parse(&base_url)?,
            download_tracker,
        })
    }

    fn create_client_with_api_key(api_key: &str) -> anyhow::Result<Client> {
        let mut headers = HeaderMap::new();

        headers.insert("apikey", HeaderValue::from_str(api_key.trim())?);
        headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));

        Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to build HTTP client")
    }

    pub async fn validate_key(api_key: &str, base_url: Url) -> anyhow::Result<ValidateResponse> {
        let client = Self::create_client_with_api_key(api_key)?;

        let res = client
            .get(base_url.join("users/validate.json")?)
            .send()
            .await?;

        let status = res.status();
        let bytes = res.bytes().await?;

        log::debug!("Validate response: {}", String::from_utf8_lossy(&bytes));

        if !status.is_success() {
            bail!("Invalid API key or access denied");
        }

        let response: ValidateResponse =
            serde_json::from_slice(&bytes).context("Failed to deserialize validate response")?;

        Ok(response)
    }

    pub async fn get_download_link(
        &self,
        request: DownloadInfoRequest,
    ) -> anyhow::Result<DownloadInfoResponse> {
        let url = self
            .base_url
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
        let url = self
            .base_url
            .join("games/")?
            .join(&format!("{}/", nexus::to_nexus_domain(game)?))?
            .join("mods/")?
            .join(&format!("{}.json", mod_id))?;

        let res = self.client.get(url).send().await?;

        let file_info: ModInfoResponse = res.json().await?;

        Ok(file_info)
    }

    pub async fn get_mod_file_info(
        &self,
        game: &Game,
        mod_id: &str,
        file_id: &str,
    ) -> anyhow::Result<ModFileInfoResponse> {
        let url = self
            .base_url
            .join("games/")?
            .join(&format!("{}/", nexus::to_nexus_domain(game)?))?
            .join("mods/")?
            .join(&format!("{}/", mod_id))?
            .join("files/")?
            .join(&format!("{}.json", file_id))?;

        let res = self.client.get(url).send().await?;

        let file_info: ModFileInfoResponse = res.json().await?;

        Ok(file_info)
    }

    pub async fn download_file(
        &self,
        url: &Url,
        output_file: &PathBuf,
        tracking_file: &PathBuf,
    ) -> anyhow::Result<()> {
        let res = self.client.get(url.clone()).send().await?;
        let total_size = res.content_length().unwrap_or(0);

        let stream = StreamReader::new(
            res.bytes_stream()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)),
        );

        self.download_tracker
            .stream_to_file_with_tracking(
                stream,
                &output_file,
                total_size,
                tracking_file.as_path(),
                output_file.file_name().unwrap().display().to_string(),
            )
            .await
    }
}
