use clap::Args;
use owo_colors::OwoColorize;
use strum::IntoEnumIterator;

use crate::{games::Game, sources::Source};

#[derive(Args)]
pub struct Supported;

impl Supported {
    pub fn run(&self) -> anyhow::Result<()> {
        println!("{}", "Supported games:\n".bold().underline().cyan());

        for game in Game::iter() {
            println!(" ➤ {}", game.to_string().to_lowercase().bold());
        }

        println!(
            "{}",
            "\nSupported mod platforms:\n".bold().underline().cyan()
        );

        for platform in Source::iter() {
            println!(" ➤ {}", platform.to_string().to_lowercase().bold());
        }

        println!();

        Ok(())
    }
}
