use clap::Args;
use owo_colors::OwoColorize;

use crate::games::get_supported_games;

#[derive(Args)]
pub struct Supported;

impl Supported {
    pub fn run(&self) -> anyhow::Result<()> {
        println!("{}", "Supported games:\n".bold().underline().cyan());

        for game in get_supported_games() {
            println!(" ➤ {}", game.name().to_lowercase().bold());
        }

        println!();

        Ok(())
    }
}
