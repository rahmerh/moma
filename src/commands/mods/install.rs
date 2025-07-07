use clap::Args;
use crossterm::style::Stylize;

use crate::{
    config::Config,
    games::{Game, workspace::Workspace},
    mods::mod_list_store::ModListStore,
    types::{FileStatus, Mod},
    ui::prompt,
};

#[derive(Args)]
pub struct Install {
    #[arg(long)]
    pub all: bool,
}

impl Install {
    pub fn run(&self, config: &Config, current_game: &Game) -> anyhow::Result<()> {
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

        if mods_with_downloaded_archives.is_empty() {
            println!("{}", "No mods available for installation.".italic());
            return Ok(());
        }

        for mod_entry in mods_with_downloaded_archives {
            let name = mod_entry.name.to_string();
            println!(
                "\n{}: '{}'",
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

            println!(
                "About to install the following archives: \n\n{}",
                archives_to_install
                    .iter()
                    .enumerate()
                    .map(|(i, a)| format!("{}. {}", i + 1, a.file_name))
                    .collect::<Vec<_>>()
                    .join("\n")
            );

            println!();
            let confirmed = prompt::confirm("Do you want to install?")?;

            if !confirmed {
                continue;
            }

            for archive in archives_to_install {
                mod_list_store.install_archive(&mod_entry, &archive)?;

                println!("Successfully installed '{}'", archive.file_name);
            }
        }

        Ok(())
    }
}
