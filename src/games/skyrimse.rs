use std::{
    fs::{self, File},
    io::Write,
};

use owo_colors::OwoColorize;

use crate::{
    config::Config,
    games::{Game, workspace::Workspace},
    sources::Source,
    ui::{print, prompt},
    utils::{self},
};
use reqwest::get;

pub fn supported_sources() -> Vec<Source> {
    vec![Source::Nexus]
}

pub fn game_mod_executable() -> &'static str {
    "skse64_loader.exe"
}

pub async fn setup(config: &Config) -> anyhow::Result<()> {
    if !prompt::confirm("Do you want to setup SKSE?")? {
        println!("{}", "\nSkipping SKSE setup.".yellow());
        return Ok(());
    }

    let workspace = Workspace::new(&Game::SkyrimSE, config)?;

    let skse_output_dir = workspace.mods_dir().join("skse");
    let skse_archive_path = workspace.cache_dir().join("skse.7z");

    if skse_output_dir.exists() {
        if prompt::confirm("SKSE already downloaded, do you want to overwrite?")? {
            fs::remove_dir_all(&skse_output_dir)?;
        } else {
            return Ok(());
        }
    }

    println!("{}", "\nSetting up SKSE\n".bold().cyan());

    print::print_inline_status(
        format!(
            "Downloading SKSE to \"{}\"",
            workspace.cache_dir().display()
        )
        .as_ref(),
    )?;

    let bytes = get("https://skse.silverlock.org/beta/skse64_2_02_06.7z")
        .await?
        .bytes()
        .await?;
    let mut out = File::create(&skse_archive_path)?;
    out.write_all(&bytes)?;

    print::print_inline_status("Extracting archive...")?;

    utils::fs::extract_archive(&skse_archive_path, &skse_output_dir, true)?;

    print::print_inline_status("Done!")?;

    println!(
        "{}",
        "\n\nSKSE installed successfully\n"
            .bold()
            .underline()
            .cyan()
    );

    Ok(())
}
