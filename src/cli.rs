use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;

use crate::{
    commands::{
        connect::Connect, context::Context, init::Init, launch::Launch, mods::nxm::NxmHandler,
        supported::Supported,
    },
    config::Config,
    utils::{state, string::StringUtils},
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
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Start Moma game setup wizard")]
    Init(Init),
    #[command(
        about = "Launch game with configuration. For a list of supported games, run `moma supported`."
    )]
    Launch(Launch),
    #[command(about = "Lists all currently supported games")]
    Supported(Supported),
    #[command(about = "Automatically connect to your desired mod platforms for downloads")]
    Connect(Connect),
    #[command(name = "mod", about = "Game mod context")]
    Context(Context),
    #[command(
        name = "nxm",
        about = "Handles the nxm link. To be used in combination with the nxm handler set up by the connect command."
    )]
    NxmHandler(NxmHandler),
}

impl Cli {
    pub async fn run(&self, config: &mut Config) -> anyhow::Result<()> {
        if let Some(game) = state::current_context()? {
            println!(
                "{}{}{}",
                "[".cyan(),
                game.to_string().capitalize().bold(),
                "]".cyan()
            );
        }

        match &self.command {
            Some(Command::Init(cmd)) => cmd.run(config).await,
            Some(Command::Launch(cmd)) => cmd.run(config),
            Some(Command::Supported(cmd)) => cmd.run(),
            Some(Command::Connect(cmd)) => cmd.run().await,
            Some(Command::Context(cmd)) => cmd.run(),
            Some(Command::NxmHandler(cmd)) => cmd.run(config).await,
            None => {
                use clap::CommandFactory;
                Cli::command().print_help()?;
                println!();
                Ok(())
            }
        }
    }
}
