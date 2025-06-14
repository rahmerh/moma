use clap::{Parser, Subcommand};

use crate::{commands::Init, config::Config};

#[derive(Parser)]
#[command(name = "moma", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
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
