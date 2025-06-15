use clap::{Parser, Subcommand};

use crate::{commands::Init, config::Config};

#[derive(Parser)]
#[command(
    name = "moma",
    version,
    about = "Declarative mod manager that automates game- and mod setup"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Start Moma game setup wizard")]
    Init(Init),
}

impl Cli {
    pub fn run(&self, config: &mut Config) -> anyhow::Result<()> {
        match &self.command {
            Some(Command::Init(cmd)) => cmd.run(config),
            None => {
                use clap::CommandFactory;
                Cli::command().print_help()?;
                println!();
                Ok(())
            }
        }
    }
}
