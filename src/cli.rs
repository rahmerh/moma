use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;

use crate::{
    commands::{
        connect::Connect,
        context::Context,
        init::Init,
        launch::Launch,
        mods::{install::Install, list_staged::ListStaged, nxm::NxmHandler},
        supported::Supported,
    },
    config::Config,
    ui::print::Colorize,
    utils::state,
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
    #[command(about = "Sets the current active game context")]
    Context(Context),
    #[command(
        name = "nxm",
        about = "Handles the nxm link. To be used in combination with the nxm handler set up by the connect command."
    )]
    NxmHandler(NxmHandler),
    #[command(subcommand, name = "mod", about = "All commands to manage mods with")]
    Mods(ModsCommand),
}

#[derive(Subcommand)]
pub enum ModsCommand {
    #[command(
        name = "list-staged",
        visible_alias = "ls",
        about = "Lists all currently staged mods. An active game context is required."
    )]
    ListStaged(ListStaged),
    #[command(about = "Installs mods from your staging directory.")]
    Install(Install),
}

impl Cli {
    pub async fn run(&self, config: &mut Config) -> anyhow::Result<()> {
        if let Some(game) = state::current_context()? {
            println!(
                "{}{}{}",
                "[".dark_cyan(),
                game.to_string().bold(),
                "]".dark_cyan()
            );
        }

        match &self.command {
            Some(Command::Init(cmd)) => cmd.run(config).await,
            Some(Command::Launch(cmd)) => cmd.run(config),
            Some(Command::Supported(cmd)) => cmd.run(),
            Some(Command::Connect(cmd)) => cmd.run().await,
            Some(Command::Context(cmd)) => cmd.run(),
            Some(Command::NxmHandler(cmd)) => cmd.run(config).await,
            Some(Command::Mods(cmd)) => match cmd {
                ModsCommand::ListStaged(cmd) => cmd.run(config),
                ModsCommand::Install(cmd) => cmd.run(config),
            },
            None => {
                use clap::CommandFactory;
                Cli::command().print_help()?;
                println!();
                Ok(())
            }
        }
    }
}
