use std::{env::home_dir, fs};

use clap::Args;

#[derive(Args)]
pub struct Init;

impl Init {
    pub fn run(&self) -> anyhow::Result<()> {
        let base = home_dir().unwrap().join(".moma");

        let paths = vec![
            base.join("mods"),
            base.join(".overlay/mods-merged"),
            base.join(".overlay/work"),
            base.join(".overlay/game"),
            base.join(".cache"),
        ];

        for path in paths {
            fs::create_dir_all(&path).expect("Help!");
        }

        eprintln!("Initialized default directories.");

        Ok(())
    }
}
