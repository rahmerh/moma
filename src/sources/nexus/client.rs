use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, bail};
use futures::TryStreamExt;
use reqwest::{
    Client, Url,
    header::{self, HeaderMap, HeaderValue},
};
use std::collections::VecDeque;
use tokio::io::AsyncReadExt;
use tokio_util::io::StreamReader;

use crate::{
    games::Game,
    sources::nexus::{
        self,
        config::Config,
        types::{
            DownloadInfoRequest, DownloadInfoResponse, ModFileInfoResponse, ModInfoResponse,
            ValidateResponse,
        },
    },
    types::DownloadProgress,
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

        let file_info: ModInfoResponse = res.json().await?;

        Ok(file_info)
    }

    pub async fn get_mod_file_info(
        &self,
        game: &Game,
        mod_id: &str,
        file_id: &str,
    ) -> anyhow::Result<ModFileInfoResponse> {
        let url = Url::parse(NEXUS_BASE_URL)?
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

    pub async fn download_file(&self, url: &Url, output_file: &PathBuf) -> anyhow::Result<()> {
        let res = self.client.get(url.clone()).send().await?;
        let total_size = res.content_length().unwrap_or(0);

        let stream = StreamReader::new(
            res.bytes_stream()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)),
        );

        self.stream_to_file_with_tracking(
            stream,
            &output_file,
            total_size,
            PathBuf::from("/tmp/hoi.json").as_path(),
            output_file.file_name().unwrap().display().to_string(),
        )
        .await
    }

    async fn stream_to_file_with_tracking<R: tokio::io::AsyncRead + Unpin>(
        &self,
        mut stream: R,
        dest_path: &Path,
        total_size: u64,
        progress_file: &Path,
        file_name: String,
    ) -> anyhow::Result<()> {
        let mut file = File::create(dest_path)?;
        let mut buffer = [0u8; 8192];

        let start_unix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let mut downloaded: u64 = 0;
        let mut last_written = Instant::now();

        loop {
            let read_bytes = stream.read(&mut buffer).await?;
            if read_bytes == 0 {
                break;
            }

            file.write_all(&buffer[..read_bytes])?;
            downloaded += read_bytes as u64;

            if last_written.elapsed().as_millis() > 500 {
                let updated_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

                let progress = DownloadProgress {
                    file_name: file_name.clone(),
                    progress_bytes: downloaded,
                    total_bytes: total_size,
                    started_at: start_unix,
                    updated_at,
                };

                let tmp_path = progress_file.with_extension("tmp");
                fs::write(&tmp_path, serde_json::to_string_pretty(&progress)?)?;
                fs::rename(tmp_path, progress_file)?;
                last_written = Instant::now();
            }
        }

        fs::remove_file(progress_file).with_context(|| {
            format!(
                "Failed to delete progress file '{}'",
                progress_file.display()
            )
        })?;

        Ok(())
    }
}
