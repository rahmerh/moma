use anyhow::bail;
use clap::Args;

use crate::{
    config::Config,
    games::workspace::Workspace,
    mods::{download_tracker::DownloadTracker, mod_list_store::ModListStore},
    usage_for,
    utils::state,
};

#[derive(Args)]
pub struct Downloads;

impl Downloads {
    pub fn run(&self, config: &Config) -> anyhow::Result<()> {
        let current_game = match state::current_context()? {
            Some(game) => game,
            None => bail!("No game context set. (Try: {})", usage_for!("context")),
        };

        let workspace = Workspace::new(&current_game, config)?;
        let mod_list_store = ModListStore::new(workspace.clone());
        let tracker = DownloadTracker::new(workspace, mod_list_store);

        let reset = tracker.reset_stuck_downloads()?;

        if reset > 0 {
            println!(
                "Info: cancelled {} stuck downloads. If this happens frequently please open an issue in github.",
                reset
            );
        }

        // let mod_list = manager.mod_list()?;
        //
        // for mod_entry in mod_list.mods {
        //     let downloads_in_progress: Vec<ModArchive> = mod_entry
        //         .archives
        //         .into_iter()
        //         .filter(|a| a.status == FileStatus::Downloading)
        //         .collect();
        //
        //     if downloads_in_progress.is_empty() {
        //         continue;
        //     }
        //
        //     for download in downloads_in_progress {
        //         println!("{}", download.file_name);
        //     }
        // }

        Ok(())
    }
}
