use anyhow::{Context, bail};
use reqwest::{
    Client, Url,
    header::{self, HeaderMap, HeaderValue},
};
use serde::Deserialize;

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

// Documentation: https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0#/
impl NexusClient {
    // pub fn new(config: &Config) -> anyhow::Result<Self> {
    //     let api_key = config
    //         .api_key
    //         .as_ref()
    //         .with_context(|| "Nexus API key not set")?;
    //     Self::with_key(api_key)
    // }

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
        log::debug!("{}", url);

        let res = self
            .client
            .get(url)
            .send()
            .await
            .context("Request failed")?;

        if !res.status().is_success() {
            bail!("Invalid API key or access denied");
        }

        let response: ValidateResponse = res
            .json()
            .await
            .context("Failed to deserialize validate response")?;
        Ok(response)
    }
}
