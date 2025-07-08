use std::{collections::HashMap, fs, path::PathBuf, process::Command};

use anyhow::{Context, bail};
use owo_colors::OwoColorize;
use reqwest::Url;

use crate::{
    games::Game,
    sources::nexus::{self, api::NxmLink, client::NexusClient},
    ui::prompt,
};

pub async fn resolve_api_key() -> anyhow::Result<String> {
    println!(
        "\n{}\n{}\n{}\n{}\n",
        "The following steps will be performed when you continue:",
        "1. A browser window will open to the Nexus Mods API page.",
        "2. Scroll down to the 'Personal API Key' section, generate a key if needed, and copy it.",
        "3. Paste the key into the prompt below and submit."
    );

    let _ = prompt::input("Press Enter to begin", true)?;
    prompt::clear_previous_lines(1, 0)?;

    Command::new("xdg-open")
        .arg("https://www.nexusmods.com/users/myaccount?tab=api")
        .spawn()?;

    let api_key: String;
    loop {
        let input = prompt::password("Enter your Nexus API key")?;
        match NexusClient::validate_key(&input, Url::parse(nexus::client::DEFAULT_NEXUS_BASE_URL)?)
            .await
        {
            Ok(res) => {
                api_key = res.key;
                break;
            }
            Err(e) => {
                log::debug!("Error: {}", e);
                println!("{}", "Invalid API key, try again.".red().bold());
            }
        }
    }

    Ok(api_key)
}

pub fn configure_nxm_link_handler() -> anyhow::Result<()> {
    let desktop_file = include_str!("../../assets/moma-nxm.desktop");
    let local_share = dirs_next::data_dir().unwrap_or_else(|| PathBuf::from("~/.local/share"));
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

    Ok(())
}

pub fn parse_nxm_url(link: &str) -> anyhow::Result<NxmLink> {
    let url = Url::parse(link).context("Failed to parse NXM URL")?;
    let game = url.host_str().context("Missing game (host)")?.to_string();

    let segments: Vec<_> = url.path_segments().map(|c| c.collect()).unwrap_or_default();
    if segments.len() < 4 {
        bail!("Invalid NXM URL format");
    }

    if segments[0] != "mods" || segments[2] != "files" {
        bail!("Unexpected NXM URL path structure");
    }

    let mod_id = segments[1].to_string();
    let file_id = segments[3].to_string();

    let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
    let key = query
        .get("key")
        .context("Missing key parameter")?
        .to_string();
    let expires = query
        .get("expires")
        .context("Missing expires parameter")?
        .to_string();

    Ok(NxmLink {
        game,
        mod_id,
        file_id,
        key,
        expires,
    })
}

pub fn from_nexus_domain(domain: &str) -> anyhow::Result<Game> {
    match domain.to_lowercase().as_str() {
        "skyrimspecialedition" => {
            Game::from_id("skyrimse").ok_or_else(|| anyhow::anyhow!("Invalid internal game ID"))
        }
        _ => bail!("Unsupported game domain: {}", domain),
    }
}
