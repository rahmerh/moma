use anyhow::bail;
use clap::Args;

use crate::{
    config::Config,
    mods::manager::Manager,
    sources::nexus::{self, Nexus},
    ui::notify,
};

#[derive(Args)]
pub struct NxmHandler {
    pub url: String,
}

impl NxmHandler {
    pub async fn run(&self, config: &Config) -> anyhow::Result<()> {
        let parsed = Nexus::parse_nxm_url(&self.url)?;

        let domain = &parsed.game;
        let game = nexus::from_nexus_domain(domain)?;
        let manager = Manager::new(&game, &config)?;

        let file_info_result =
            Nexus::get_mod_file_info(&game, &parsed.mod_id, &parsed.file_id).await;
        let file_info = match file_info_result {
            Ok(i) => i,
            Err(i) => {
                notify::send_notification(&format!(
                    "A problem occured during download: '{}'",
                    i.to_string()
                ))?;
                bail!(i.to_string())
            }
        };

        let mod_info = Nexus::get_mod_info(&game, &parsed.mod_id).await?;

        notify::send_notification(&format!("Starting '{}' download", file_info.file_name))?;

        let staged_archive_path = manager.prepare_staging_download(&mod_info, &file_info)?;

        if staged_archive_path.exists() {
            notify::send_notification(&format!(
                "'{}' already downloaded, skipping.",
                file_info.file_name
            ))?;

            return Ok(());
        }

        let download_link = Nexus::get_download_link(parsed).await?;
        Nexus::download_file(&download_link, &staged_archive_path).await?;

        manager.register_archive(&mod_info, &file_info)?;

        notify::send_notification(&format!(
            "Downloaded {} to '{}'",
            file_info.file_name,
            staged_archive_path.display()
        ))?;

        Ok(())
    }
}
