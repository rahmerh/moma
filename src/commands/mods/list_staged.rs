use anyhow::bail;
use clap::Args;
use owo_colors::OwoColorize;

use crate::{config::Config, mods::manager::Manager, ui::print::Colorize, utils::state};

#[derive(Args)]
pub struct ListStaged {}

impl ListStaged {
    pub fn run(&self, config: &Config) -> anyhow::Result<()> {
        let current_game = match state::current_context()? {
            Some(game) => game,
            None => bail!("No game context set, please run 'moma context' first."),
        };

        let manager = Manager::new(&current_game, config)?;

        println!(
            "\n{}: {}\n",
            "Mods in your staged directory for".dark_cyan(),
            current_game.to_string().bold()
        );
        for staged in manager.get_staged_mod_infos()? {
            println!(" ➤ {}", staged.name);
        }
        println!(
            "\nRun 'moma mod install all' to install all staged mods.\nOr run 'moma mod install' to pick mods to install."
        );

        Ok(())
    }
}
