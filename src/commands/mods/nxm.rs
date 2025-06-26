use anyhow::bail;
use clap::Args;

use crate::{
    config::Config,
    mods::manager::Manager,
    sources::nexus::{self, Nexus},
    types::{FileStatus, ModArchive},
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

        let mod_info = Nexus::get_mod_info(&game, &parsed.mod_id).await?;
        let file_info_result =
            Nexus::get_mod_file_info(&game, &parsed.mod_id, &parsed.file_id).await;
        let mut file_info = match file_info_result {
            Ok(i) => i,
            Err(i) => {
                notify::send_notification(&format!(
                    "A problem occured during download: '{}'",
                    i.to_string()
                ))?;
                bail!(i.to_string())
            }
        };
        file_info.archive_path = manager.get_cache_path();
        file_info.status = FileStatus::Downloading;

        if manager.is_archive_present(file_info.file_uid)? {
            notify::send_notification(&format!(
                "Archive '{}' is already downloaded.",
                file_info.file_name
            ))?;
            return Ok(());
        }

        manager.mark_archive_status(&mod_info, &file_info, FileStatus::Downloading)?;

        notify::send_notification(&format!("Starting '{}' download", file_info.file_name))?;

        let download_link = Nexus::get_download_link(&parsed).await?;
        Nexus::download_file(
            &download_link,
            &file_info.archive_path.join(&file_info.file_name),
        )
        .await?;

        notify::send_notification(&format!(
            "Downloaded complete for '{}'",
            file_info.file_name,
        ))?;

        manager.mark_archive_status(&mod_info, &file_info, FileStatus::Downloaded)?;
        manager.stage_archive(mod_info, file_info)?;

        Ok(())
    }
}
