use clap::Args;

use crate::utils::state;

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

        state::set_context(self.game.as_str())?;

        Ok(())
    }
}
