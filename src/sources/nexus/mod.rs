use std::process::Command;

use anyhow::Context;
use dialoguer::{Confirm, Input, Password};
use owo_colors::OwoColorize;

use crate::sources::nexus::{client::NexusClient, config::Config};

mod client;
pub mod config;

pub struct NexusPlatform;

impl NexusPlatform {
    pub fn setup(&self) -> anyhow::Result<()> {
        if config::api_key_exists() {
            let confirmation = Confirm::with_theme(&theme)
                .with_prompt("Nexus connection already set up, do you want to overwrite?")
                .interact()?;

            if !confirmation {
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

        let _ = Input::<String>::new()
            .with_prompt("Press Enter to begin")
            .allow_empty(true)
            .interact_text();

        Command::new("xdg-open")
            .arg("https://www.nexusmods.com/users/myaccount?tab=api")
            .spawn()?;

        let response;
        loop {
            let input = Password::with_theme(&theme)
                .with_prompt("Enter your Nexus API key")
                .interact()
                .with_context(|| "Failed to read input")?;

            match NexusClient::validate_key(&input) {
                Ok(r) => {
                    println!("Authenticated as Nexus user: {}", r.name);
                    response = r;
                    break;
                }
                Err(_) => {
                    eprintln!("{}{}", "Invalid key".red(), ", please try again.");
                }
            }
        }

        Config::save_api_key(&response.key)?;
        let mut config = Config::load()?;
        config.username = Some(response.name);
        config.is_premium = response.is_premium;
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
