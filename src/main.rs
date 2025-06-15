mod cli;
mod commands;
mod config;
mod games;
mod theme;
mod utils;

use clap::Parser;
use owo_colors::OwoColorize;

use crate::{cli::Cli, config::Config};

fn main() -> anyhow::Result<()> {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("{} {}", "Fatal error:".red().bold(), info);
    }));

    let mut config = Config::load_or_default();

    Cli::parse().run(&mut config)?;

    Ok(())
}
