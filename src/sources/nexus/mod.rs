use std::process::Command;

use anyhow::Context;
use owo_colors::OwoColorize;

use crate::{
    sources::nexus::{client::NexusClient, config::Config},
    ui::prompt,
};

mod client;
pub mod config;

pub struct Nexus;

impl Nexus {
    pub fn is_setup() -> bool {
        match Config::load() {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn setup() -> anyhow::Result<()> {
        if Self::is_setup() {
            if !prompt::confirm("Nexus connection already set up, do you want to overwrite?")? {
                println!("{}", "Exiting setup.".yellow());
                return Ok(());
            }
        }

        println!("\n{}", "Connecting to Nexus Mods".bold().underline().cyan());

        println!(
            "\n{}\n{}\n{}\n{}\n",
            "The following steps will be performed when you continue:",
            "1. A browser window will open to the Nexus Mods API page.",
            "2. Scroll down to the 'Personal API Key' section, generate a key if needed, and copy it.",
            "3. Paste the key into the prompt below and submit."
        );

        let _ = prompt::input("Press Enter to begin", true)?;

        Command::new("xdg-open")
            .arg("https://www.nexusmods.com/users/myaccount?tab=api")
            .spawn()?;

        let api_key = prompt::password_with_retry("Enter your Nexus API key", |key| {
            NexusClient::validate_key(key)
                .map(|_| key.to_string())
                .map_err(|_| anyhow::anyhow!("Invalid API key"))
        })?;

        let nexus_user = NexusClient::validate_key(&api_key)
            .with_context(|| "Could not validate the nexus API key")?;

        Config::save_api_key(&nexus_user.key)?;
        let mut config = Config::load()?;
        config.username = Some(nexus_user.name);
        config.is_premium = nexus_user.is_premium;
        config.save()?;

        println!(
            "{}",
            "\nSuccessfully connected your nexus mods account!\n"
                .cyan()
                .bold()
        );

        Ok(())
    }

    pub fn download_mod(&self, config: &Config) -> anyhow::Result<()> {
        let client = NexusClient::new(config);

        Ok(())
    }
}
