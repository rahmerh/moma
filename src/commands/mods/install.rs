use std::path::PathBuf;

use anyhow::bail;
use clap::Args;

use crate::{
    config::Config,
    mods::manager::Manager,
    ui::{prompt, reorder},
    utils::state,
};

#[derive(Args)]
pub struct Install {
    #[arg(long)]
    pub all: bool,
}

impl Install {
    pub fn run(&self, config: &Config) -> anyhow::Result<()> {
        let current_game = match state::current_context()? {
            Some(game) => game,
            None => bail!("No game context set, please run 'moma context' first."),
        };

        let manager = Manager::new(&current_game, config)?;
        let staged_mods = manager.get_staged_mod_infos()?;

        let mods_to_install;
        if self.all {
            mods_to_install = staged_mods;
        } else {
            mods_to_install =
                prompt::select_multiple("Select the mods you want to install", &staged_mods)?;
        }

        for mod_to_install in mods_to_install {
            let mut files_to_install: Vec<PathBuf> = Vec::new();
            if mod_to_install.downloaded_archives.len() > 1 {
                let selected_files = prompt::select_multiple(
                    "Select the archives to install",
                    &mod_to_install.downloaded_archives,
                )?;

                let selected_files = reorder::reorder_items(selected_files)?;

                // files_to_install.extend(
                //     selected_files
                //         .iter()
                //         .map(|f| manager.get_staged_archive_path(&mod_to_install, &f.file_name)),
                // );
            } else {
                // files_to_install.extend(
                //     mod_to_install
                //         .downloaded_archives
                //         .iter()
                //         .map(|f| manager.get_staged_archive_path(&mod_to_install, &f.file_name)),
                // );
            }

            for file in files_to_install {
                manager.install_archive_for_mod(&mod_to_install.name, &file)?;
            }
        }

        Ok(())
    }
}
