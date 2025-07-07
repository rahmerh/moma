use clap::Args;

use crate::{config::Config, games::Game};

#[derive(Args)]
pub struct Uninstall;

impl Uninstall {
    pub fn run(&self, config: &Config, current_game: &Game) -> anyhow::Result<()> {
        Ok(())
    }
}
