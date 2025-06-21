use clap::Args;
use owo_colors::OwoColorize;

use crate::{games, mod_platforms};

#[derive(Args)]
pub struct Supported;

impl Supported {
    pub fn run(&self) -> anyhow::Result<()> {
        println!("{}", "Supported games:\n".bold().underline().cyan());

        for game in games::get_supported_games() {
            println!(" ➤ {}", game.name().to_lowercase().bold());
        }

        println!(
            "{}",
            "\nSupported mod platforms:\n".bold().underline().cyan()
        );

        for platform in mod_platforms::get_supported_mod_platforms() {
            println!(" ➤ {}", platform.name().to_lowercase().bold());
        }

        println!();

        Ok(())
    }
}
