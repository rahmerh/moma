mod cli;
mod commands;
mod config;

use clap::Parser;
use cli::{Command, MomaCli};
use commands::init;
use config::Config;

fn main() {
    let config_path = dirs_next::config_dir().unwrap().join("moma/config.toml");

    if !config_path.exists() {
        let config = Config::default();
        config
            .save_to_file(&config_path)
            .expect("Failed to save config");
    }

    let args = MomaCli::parse();

    match args.command {
        Some(Command::Init) => {
            init::run();
        }
        None => {
            eprintln!("No command provided. Use --help.");
        }
    }
}
