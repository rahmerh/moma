use anyhow::bail;
use clap::Args;
use crossterm::style::Stylize;

use crate::{
    config::Config,
    games::workspace::Workspace,
    mods::mod_list_store::ModListStore,
    types::{FileStatus, Mod},
    ui::prompt,
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

        let workspace = Workspace::new(&current_game, config)?;
        let mod_list_store = ModListStore::new(workspace);
        let mod_list = mod_list_store.read()?;

        let mods_with_downloaded_archives: Vec<Mod> = mod_list
            .mods
            .into_iter()
            .filter_map(|mut m| {
                m.archives.retain(|a| a.status == FileStatus::Downloaded);
                if m.archives.is_empty() { None } else { Some(m) }
            })
            .collect();

        for mod_entry in mods_with_downloaded_archives {
            let name = mod_entry.name.to_string();
            println!(
                "\n{}: '{}'\n",
                "Installing archives for".cyan().bold(),
                name.bold()
            );

            let mut archives_to_install = Vec::new();
            if mod_entry.archives.len() > 1 {
                let selection = prompt::select_multiple(
                    "Select one or more archives to install.",
                    &mod_entry.archives,
                )?;

                archives_to_install.extend(selection);
            } else {
                archives_to_install.extend(mod_entry.archives.clone());
            }

            for archive in archives_to_install {
                mod_list_store.install_archive(&mod_entry, &archive)?;
            }
        }

        Ok(())
    }
}
