use clap::Args;

use crate::{games::Game, utils::state};

#[derive(Args)]
pub struct Context {
    pub game: String,
}

impl Context {
    pub fn run(&self) -> anyhow::Result<()> {
        if self.game.to_lowercase() == "clear" {
            state::clear_context()?;
            return Ok(());
        }

        let game = Game::from_id(&self.game)
            .ok_or_else(|| anyhow::anyhow!("Unknown game '{}'", self.game))?;

        state::set_context(game)?;
        Ok(())
    }
}
