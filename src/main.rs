mod cli;
mod commands;
mod config;

use std::process::ExitCode;

use clap::Parser;
use config::Config;

use crate::cli::Cli;

fn main() -> ExitCode {
    let config_path = dirs_next::config_dir().unwrap().join("moma/config.toml");

    if !config_path.exists() {
        let config = Config::default();
        config
            .save_to_file(&config_path)
            .expect("Failed to save config");
    }

    match Cli::parse().run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            println!("Error!: {e:?}");
            ExitCode::FAILURE
        }
    }
}
