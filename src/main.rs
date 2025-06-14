mod cli;
mod commands;
mod config;
mod games;

use std::process::ExitCode;

use clap::Parser;

use crate::{cli::Cli, config::Config};

fn main() -> ExitCode {
    let mut config = Config::load_or_default();

    match Cli::parse().run(&mut config) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            println!("Error!: {e:?}");
            ExitCode::FAILURE
        }
    }
}
