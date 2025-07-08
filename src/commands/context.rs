use clap::Args;
use owo_colors::OwoColorize;

use crate::{config::Config, games::Game, utils::state::State};

#[derive(Args)]
pub struct Context {
    pub game: Option<String>,
}

impl Context {
    pub fn run(&self, config: &Config) -> anyhow::Result<()> {
        let state = State::new(config.state_file());

        let arg = match &self.game {
            Some(g) => g,
            None => {
                state.clear_context()?;
                return Ok(());
            }
        };

        let game = Game::from_id(arg).ok_or_else(|| anyhow::anyhow!("Unknown game '{}'", arg))?;

        state.set_context(&game)?;

        println!(
            "{} '{}'",
            "Current context set to:".bold().underline().cyan(),
            game.to_string().bold()
        );
        Ok(())
    }
}
