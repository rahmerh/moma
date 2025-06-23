use std::{fs, path::PathBuf, process::Command};

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
        Config::load()
            .map(|cfg| cfg.api_key.is_some())
            .unwrap_or(false)
    }

    pub async fn setup() -> anyhow::Result<()> {
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

        let api_key: String;
        loop {
            let input = prompt::password("Enter your Nexus API key")?;
            match NexusClient::validate_key(&input).await {
                Ok(res) => {
                    api_key = res.key;
                    break;
                }
                Err(_) => {
                    println!("{}", "Invalid API key, try again.".red().bold());
                    continue;
                }
            };
        }

        let nexus_user = NexusClient::validate_key(&api_key)
            .await
            .with_context(|| "Could not validate the nexus API key")?;

        Config::save_api_key(&nexus_user.key)?;
        let mut config = Config::default();
        config.username = Some(nexus_user.name);
        config.is_premium = nexus_user.is_premium;
        config.save()?;

        println!(
            "{}",
            "\nSuccessfully connected your nexus mods account!\n"
                .cyan()
                .bold()
        );

        if prompt::confirm("Do you want to set up the nxm link handler?")? {
            println!(
                "{}\n{}",
                "Copying moma-nxm.desktop to ~/.local/share/applications".italic(),
                "Executing 'xdg-mime default moma-nxm.desktop x-scheme-handler/nxm'".italic()
            );

            let desktop_file = include_str!("../../assets/moma-nxm.desktop");
            let local_share =
                dirs_next::data_dir().unwrap_or_else(|| PathBuf::from("~/.local/share"));
            let desktop_path = local_share.join("applications/moma-nxm-handler.desktop");

            fs::write(&desktop_path, desktop_file).context("Failed to write .desktop file")?;

            Command::new("xdg-mime")
                .args([
                    "default",
                    "moma-nxm-handler.desktop",
                    "x-scheme-handler/nxm",
                ])
                .status()
                .context("Failed to run xdg-mime")?;
        }

        Ok(())
    }
}
