use clap::{Parser, Subcommand};

use crate::{
    commands::{Init, Launch, Supported},
    config::Config,
};

#[derive(Parser)]
#[command(
    name = "moma",
    version,
    about = "Declarative mod manager that automates game- and mod setup"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Prints extra error information to be used in debugging.
    #[arg(short, long, global = true)]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Start Moma game setup wizard")]
    Init(Init),
    Launch(Launch),
    #[command(about = "Lists all currently supported games")]
    Supported(Supported),
}

impl Cli {
    pub fn run(&self, config: &mut Config) -> anyhow::Result<()> {
        match &self.command {
            Some(Command::Init(cmd)) => cmd.run(config),
            Some(Command::Launch(cmd)) => cmd.run(config),
            Some(Command::Supported(cmd)) => cmd.run(),
            None => {
                use clap::CommandFactory;
                Cli::command().print_help()?;
                println!();
                Ok(())
            }
        }
    }
}
