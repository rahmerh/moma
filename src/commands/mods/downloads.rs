use anyhow::bail;
use clap::Args;

use crate::{
    config::Config,
    games::workspace::Workspace,
    mods::{download_tracker::DownloadTracker, mod_list_store::ModListStore},
    ui::progress,
    usage_for,
    utils::state,
};

#[derive(Args)]
pub struct Downloads;

impl Downloads {
    pub fn run(&self, config: &Config) -> anyhow::Result<()> {
        let current_game = match state::current_context(&config.state_file)? {
            Some(game) => game,
            None => bail!("No game context set. (Try: {})", usage_for!("context")),
        };

        let workspace = Workspace::new(&current_game, config)?;
        let mod_list_store = ModListStore::new(workspace.clone());
        let tracker = DownloadTracker::new(workspace.clone(), mod_list_store);

        let reset = tracker.reset_stuck_downloads()?;

        if reset > 0 {
            println!(
                "Info: cancelled {} stuck downloads. If this happens frequently please open an issue in github.",
                reset
            );
        }

        progress::display_active_downloads(&workspace.tracking_dir())?;

        Ok(())
    }
}
