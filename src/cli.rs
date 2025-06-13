use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "packsy", version)]
pub struct MomaCli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Init,
}
