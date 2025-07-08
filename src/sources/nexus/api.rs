use std::path::PathBuf;

use reqwest::Url;

use crate::{
    games::Game,
    mods::download_tracker::DownloadTracker,
    sources::nexus::{client::NexusClient, config::Config, types::DownloadInfoRequest},
    types::{Mod, ModArchive},
};

#[derive(Clone)]
pub struct NxmLink {
    pub game: String,
    pub mod_id: String,
    pub file_id: String,
    pub key: String,
    pub expires: String,
}

impl From<NxmLink> for DownloadInfoRequest {
    fn from(link: NxmLink) -> Self {
        Self {
            game: link.game,
            mod_id: link.mod_id,
            file_id: link.file_id,
            key: link.key,
            expires: link.expires,
        }
    }
}

pub struct Nexus {
    client: NexusClient,
}

impl Nexus {
    pub fn new(config: &Config, download_tracker: DownloadTracker) -> anyhow::Result<Self> {
        Ok(Self {
            client: NexusClient::new(config, download_tracker)?,
        })
    }

    pub async fn get_mod_info(&self, game: &Game, mod_id: &str) -> anyhow::Result<Mod> {
        let response = self.client.get_mod_info(game, mod_id).await?;
        Ok(response.into())
    }

    pub async fn get_mod_file_info(
        &self,
        game: &Game,
        mod_id: &str,
        file_id: &str,
    ) -> anyhow::Result<ModArchive> {
        let response = self.client.get_mod_file_info(game, mod_id, file_id).await?;
        Ok(response.into())
    }

    pub async fn get_download_link(&self, nxmlink: &NxmLink) -> anyhow::Result<Url> {
        let request: DownloadInfoRequest = nxmlink.clone().into();
        let response = self.client.get_download_link(request).await?;
        Ok(Url::parse(&response.uri)?)
    }

    pub async fn download_file(
        &self,
        url: &Url,
        output_file: &PathBuf,
        tracking_file: &PathBuf,
    ) -> anyhow::Result<()> {
        self.client
            .download_file(url, output_file, tracking_file)
            .await
    }
}
