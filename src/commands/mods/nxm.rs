use std::process::Command;

use anyhow::Context;
use clap::Args;

use crate::{
    config::Config,
    games::workspace::Workspace,
    sources::nexus::{self, Nexus},
    utils::fs::{self, ExpandTilde},
};

#[derive(Args)]
pub struct NxmHandler {
    pub url: String,
}

impl NxmHandler {
    // nxm://skyrimspecialedition/mods/152490/files/638592?key=0Or2IM4l-FXSJjvRogxbMw&expires=1750810470&user_id=191018313
    pub async fn run(&self, config: &Config) -> anyhow::Result<()> {
        let parsed = Nexus::parse_nxm_url(&self.url)?;

        let domain = &parsed.game;
        let game = nexus::from_nexus_domain(domain)?;
        let game_config = config.games.get(game.id()).ok_or_else(|| {
            anyhow::anyhow!("No configuration found for game {}", game.to_string())
        })?;
        let workspace = Workspace::new(config, game_config)?;

        let mod_info = Nexus::get_mod_info(&game, &parsed.mod_id).await?;
        let file_info = Nexus::get_mod_file_info(&game, &parsed.mod_id, &parsed.file_id).await?;

        let mod_cache_base_dir = &workspace.cache_dir().join(&mod_info.name);
        let output_path = mod_cache_base_dir.join(&file_info.file_name);

        if !output_path.exists() {
            std::fs::create_dir(mod_cache_base_dir).with_context(|| {
                format!("Folder '{}' already exists.", mod_cache_base_dir.display())
            })?;

            Command::new("notify-send")
                .arg(&format!("Starting mod download for {}", mod_info.name))
                .spawn()?;

            let download_link = Nexus::get_download_link(parsed).await?;

            Nexus::download_file(&download_link, &output_path).await?;

            Command::new("notify-send")
                .arg(format!(
                    "Downloaded {} to '{}'",
                    mod_info.name,
                    workspace.mods_dir().join(&file_info.file_name).display()
                ))
                .spawn()?;

            let extracted_path = &mod_cache_base_dir.join(&mod_info.name).expand();
            println!("{}", extracted_path.display());
            std::fs::create_dir(extracted_path)?;
            fs::extract_archive(&output_path, &extracted_path, false)?;
        }

        Ok(())
    }
}
