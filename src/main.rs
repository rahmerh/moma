mod cli;
mod commands;
mod config;
mod games;
mod overlay;
mod theme;
mod utils;

use clap::Parser;
use owo_colors::OwoColorize;

use crate::{cli::Cli, config::Config};

fn main() {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("{} {}", "Encountered a panic:".red().bold(), info);
        std::process::exit(1);
    }));

    let cli = Cli::parse();

    if let Err(err) = run(&cli) {
        eprintln!("{} {}", "Encountered a problem:".red().bold(), err);

        if cli.debug {
            for cause in err.chain().skip(1) {
                eprintln!("  caused by: {}", cause);
            }
        }
        std::process::exit(1);
    }
}

fn run(cli: &Cli) -> anyhow::Result<()> {
    let mut config = Config::load_or_default();
    cli.run(&mut config)?;
    Ok(())
}
