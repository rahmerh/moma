use anyhow::{Context, bail};
use clap::Args;

use crate::{
    config::Config,
    games::workspace::Workspace,
    mods::{download_tracker::DownloadTracker, mod_list_store::ModListStore},
    sources::nexus::{self},
    types::FileStatus,
    ui::notify,
    usage_for,
};

#[derive(Args)]
pub struct NxmHandler {
    pub url: String,
}

impl NxmHandler {
    pub async fn run(&self, config: &Config) -> anyhow::Result<()> {
        let nexus_config = match nexus::Config::load() {
            Ok(config) => config,
            Err(_) => {
                let message = &format!(
                    "Nexus connection is not set up. Run '{}'",
                    usage_for!("connect")
                );
                notify::send_notification(message)?;
                bail!("{message}");
            }
        };

        let parsed = nexus::parse_nxm_url(&self.url)?;

        let domain = &parsed.game;
        let game = nexus::from_nexus_domain(domain)?;

        let workspace = Workspace::new(&game, config)?;
        let mod_list_store = ModListStore::new(workspace.clone());
        let download_tracker = DownloadTracker::new(workspace, mod_list_store.clone());

        let api = nexus::Nexus::new(&nexus_config)?;

        let mod_info = api.get_mod_info(&game, &parsed.mod_id).await?;
        let file_info_result = api
            .get_mod_file_info(&game, &parsed.mod_id, &parsed.file_id)
            .await;
        let mut file_info = match file_info_result {
            Ok(i) => i,
            Err(err) => {
                notify::send_notification(&format!(
                    "A problem occured during download: '{}'",
                    err
                ))?;
                bail!(err)
            }
        };

        let current_archive_status =
            mod_list_store.get_archive_status(mod_info.uid, file_info.file_uid)?;

        if current_archive_status != FileStatus::Unknown {
            match &current_archive_status {
                FileStatus::Downloading => {
                    notify::send_notification(&format!(
                        "There is already a download in progress for '{}'.",
                        file_info.file_name
                    ))?;
                    return Ok(());
                }
                FileStatus::Downloaded => {
                    notify::send_notification(&format!(
                        "'{}' is already downloaded.",
                        file_info.file_name
                    ))?;
                    return Ok(());
                }
                FileStatus::Installed => {
                    notify::send_notification(&format!(
                        "'{}' is already installed.",
                        file_info.file_name
                    ))?;
                    return Ok(());
                }
                FileStatus::Unknown | FileStatus::Failed(_) => {
                    // TODO: Allow redownload for now, add prompt or something later
                }
            };
        }
        let archive_path = mod_list_store.archive_download_dest(&file_info.file_name);

        file_info.status = FileStatus::Downloading;
        file_info.archive_path = Some(archive_path.clone());
        mod_list_store.add_archive(&mod_info, file_info.clone())?;

        notify::send_notification(&format!("Starting download for '{}'", file_info.file_name))?;

        let tracking_file = download_tracker
            .ensure_tracking_file(file_info.file_uid)
            .with_context(|| {
                format!(
                    "Could not resolve tracking file path for {}",
                    file_info.file_name
                )
            })?;

        let download_link = api.get_download_link(&parsed).await?;
        let download_result = api
            .download_file(&download_link, &archive_path, &tracking_file)
            .await;

        match download_result {
            Ok(()) => {
                notify::send_notification(&format!(
                    "Download complete for '{}'",
                    file_info.file_name
                ))?;
                mod_list_store.stage_archive(&mod_info, &file_info)?;
            }
            Err(err) => {
                notify::send_notification(&format!(
                    "Download failed for '{}': {}",
                    file_info.file_name, err
                ))?;
                mod_list_store.update_archive(&mod_info.uid, &file_info.file_uid, |a| {
                    a.status = FileStatus::Failed(format!("{err:#}"))
                })?;
                return Err(err);
            }
        }

        Ok(())
    }
}
