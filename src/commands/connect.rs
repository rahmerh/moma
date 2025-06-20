use clap::Args;
use dialoguer::{Confirm, Input, Password};
use owo_colors::OwoColorize;
use reqwest::{Url, blocking::Client};
use serde_json::Value;

use crate::utils::{
    nexus::{self},
    theme,
};

#[derive(Args)]
pub struct Connect {
    pub mod_platform: String,
}

const NEXUS_BASE_URL: &str = "https://api.nexusmods.com/v1/";

impl Connect {
    pub fn run(&self, debug: bool) -> anyhow::Result<()> {
        let theme = theme::default_theme();

        if nexus::api_key_exists() {
            let confirmation = Confirm::with_theme(&theme)
                .with_prompt("Nexus connection already set up, do you want to overwrite?")
                .interact()?;

            if !confirmation {
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

        let path = Password::with_theme(&theme)
            .with_prompt(format!("Enter your API key",))
            .validate_with(|input: &String| {
                let client = Client::new();

                let base = Url::parse(NEXUS_BASE_URL).unwrap();
                let url = base.join("users/validate.json").unwrap();

                log::debug!("{}", url.as_str());

                let res = match client
                    .get(url)
                    .header("apikey", input)
                    .header("Accept", "application/json")
                    .send()
                {
                    Ok(r) => r,
                    Err(_) => return Err("Could not reach Nexus Mods API"),
                };

                if !res.status().is_success() {
                    log::debug!("{}", res.text().unwrap());
                    return Err("Invalid API key or access denied");
                }

                let json: Value = match res.json() {
                    Ok(j) => j,
                    Err(_) => return Err("Failed to parse Nexus Mods response"),
                };

                let user = json
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");
                println!("Authenticated as Nexus user: {}", user);

                Ok(())
            })
            .interact()?;

        Ok(())
    }
}
