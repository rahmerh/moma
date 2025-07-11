use clap::Args;

use crate::{config::Config, games::Game};

#[derive(Args)]
pub struct Uninstall;

impl Uninstall {
    pub fn run(&self, _config: &Config, _current_game: &Game) -> anyhow::Result<()> {
        Ok(())
    }
}
