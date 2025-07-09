use clap::{Parser, Subcommand};

use crate::{app_context::AppContext, commands::init::Init, config::Config};

#[derive(Parser)]
#[command(
    name = Cli::MOMA,
    version,
    about = "The layered mod manager that automates game and mod setup"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(name = Cli::INIT, about = "Start Moma game setup wizard")]
    Init(Init),
    // #[command(name = Cli::LAUNCH, about = "Launch game with configuration.")]
    // Launch(Launch),
    // #[command(name = Cli::CONNECT, about = "Automatically connect to your desired mod platforms for downloads")]
    // Connect(Connect),
    // #[command(name= Cli::CONTEXT, about = "Sets the current active game context")]
    // Context(Context),
    // #[command(name = Cli::NXM, hide = true)]
    // NxmHandler(NxmHandler),
    // #[command(subcommand, name = Cli::MOD, about = "All commands to manage mods with")]
    // Mods(ModsCommand),
}

// #[derive(Subcommand)]
// pub enum ModsCommand {
//     #[command(name = Cli::MOD_DOWNLOADS, about = "Displays status of all downloads")]
//     Downloads(Downloads),
//     #[command(name = Cli::MOD_INSTALL, about = "Installs mods from your staging directory.")]
//     Install(Install),
// }

impl Cli {
    pub const MOMA: &str = "moma";

    pub const INIT: &str = "init";

    pub async fn run(&self, config: &mut Config) -> anyhow::Result<()> {
        let ctx = AppContext::new();

        match &self.command {
            Some(Command::Init(cmd)) => cmd.run(ctx, config).await,
            // Some(Command::Launch(cmd)) => cmd.run(config),
            // Some(Command::Connect(cmd)) => cmd.run().await,
            // Some(Command::Context(cmd)) => cmd.run(config),
            // Some(Command::NxmHandler(cmd)) => cmd.run(config).await,
            // Some(Command::Mods(cmd)) => {
            //     let state = State::new(config.state_file());
            //     let current_context = match state.current_context()? {
            //         Some(context) => context,
            //         None => bail!(
            //             "Game context required for mod commands (Try: '{}')",
            //             usage_for!(Cli::CONTEXT)
            //         ),
            //     };
            //
            //     match cmd {
            //         ModsCommand::Install(cmd) => cmd.run(config, &current_context),
            //         ModsCommand::Downloads(cmd) => cmd.run(config, &current_context),
            //     }
            // }
            None => {
                use clap::CommandFactory;
                Cli::command().print_help()?;
                println!();
                Ok(())
            }
        }
    }
}
