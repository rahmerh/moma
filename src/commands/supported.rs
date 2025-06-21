use clap::Args;
use owo_colors::OwoColorize;
use strum::IntoEnumIterator;

use crate::{games, mod_platforms::ModPlatformKind};

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

        for platform in ModPlatformKind::iter() {
            println!(" ➤ {}", platform.to_string().to_lowercase().bold());
        }

        println!();

        Ok(())
    }
}
