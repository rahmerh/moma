mod cli;
mod commands;
mod config;
mod games;
mod sources;
mod types;
mod ui;
mod utils;

use clap::Parser;
use owo_colors::OwoColorize;

use crate::{cli::Cli, config::Config};

#[tokio::main]
async fn main() {
    env_logger::init();
    std::panic::set_hook(Box::new(|info| {
        eprintln!("{} {}", "Encountered a panic:".red().bold(), info);
        std::process::exit(1);
    }));

    let cli = Cli::parse();

    if let Err(err) = run(&cli).await {
        eprintln!("{} {}", "Encountered a problem:".red().bold(), err);

        for cause in err.chain().skip(1) {
            log::debug!(" -> caused by: {}", cause);
        }
        std::process::exit(1);
    }
}

async fn run(cli: &Cli) -> anyhow::Result<()> {
    let mut config = Config::load_or_default()?;
    cli.run(&mut config).await?;
    Ok(())
}
