use anyhow::bail;
use clap::Args;
use owo_colors::OwoColorize;

use crate::{
    config::Config,
    mods::manager::Manager,
    ui::{print::Colorize, prompt},
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
        let staged_games = manager.get_staged_mod_infos()?;

        let games_to_install;
        if self.all {
            games_to_install = staged_games;
        } else {
            games_to_install =
                prompt::select_multiple("Select the mods you want to install", &staged_games)?;
        }

        manager.extract_and_move_mods(games_to_install);

        Ok(())
    }
}
