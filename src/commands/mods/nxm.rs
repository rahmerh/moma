use anyhow::bail;
use clap::Args;

use crate::{
    config::Config,
    mods::manager::Manager,
    sources::nexus::{self, Nexus},
    types::FileStatus,
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

        let current_archive_status =
            manager.get_archive_status(mod_info.uid, file_info.file_uid)?;

        if current_archive_status != FileStatus::Unknown {
            let message = match current_archive_status {
                FileStatus::Unknown => "",
                FileStatus::Downloading => &format!(
                    "There is already a download in progress for '{}'.",
                    file_info.file_name
                ),
                FileStatus::Downloaded => {
                    &format!("'{}' is already downloaded.", file_info.file_name)
                }
                FileStatus::Installed => {
                    &format!("'{}' is already installed.", file_info.file_name)
                }
                FileStatus::Error => "",
            };
            notify::send_notification(&message)?;
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
