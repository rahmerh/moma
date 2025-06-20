use clap::{Parser, Subcommand};

use crate::{
    commands::{
        connect::Connect, init::Init, launch::Launch, modctl::add::Add, supported::Supported,
    },
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
    #[command(
        about = "Launch game with configuration. For a list of supported games, run `moma supported`."
    )]
    Launch(Launch),
    #[command(about = "Lists all currently supported games")]
    Supported(Supported),
    #[command(about = "Automatically connect to your desired mod platforms for downloads")]
    Connect(Connect),
    #[command(about = "Mod control commands")]
    Mod {
        game: String,

        #[command(subcommand)]
        action: ModAction,
    },
}

#[derive(Subcommand)]
pub enum ModAction {
    #[command(about = "Adds the mod with corresponding mod id to the game's mod folder.")]
    Add {
        /// The mod's nexus id.
        mod_id: String,
    },
}

impl ModAction {
    pub fn run(&self, game: &str, config: &mut Config) -> anyhow::Result<()> {
        match self {
            ModAction::Add { mod_id } => Add::run(game, mod_id, config),
        }
    }
}

impl Cli {
    pub fn run(&self, config: &mut Config) -> anyhow::Result<()> {
        match &self.command {
            Some(Command::Init(cmd)) => cmd.run(config),
            Some(Command::Launch(cmd)) => cmd.run(config),
            Some(Command::Supported(cmd)) => cmd.run(),
            Some(Command::Connect(cmd)) => cmd.run(self.debug),
            Some(Command::Mod { game, action }) => action.run(game, config),
            None => {
                use clap::CommandFactory;
                Cli::command().print_help()?;
                println!();
                Ok(())
            }
        }
    }
}
