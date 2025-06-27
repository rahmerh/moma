use std::{
    fs::{self, File},
    io::Write,
};

use owo_colors::OwoColorize;

use crate::{
    config::{CACHE_DIR_NAME, Config, MODS_DIR_NAME},
    games::Game,
    sources::Source,
    ui::{print, prompt},
    utils::{self, fs::ExpandTilde},
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

    let game_work_dir = config.work_dir.join(Game::SkyrimSE.id()).expand();

    let cache_dir = game_work_dir.join(CACHE_DIR_NAME);
    let mods_dir = game_work_dir.join(MODS_DIR_NAME);

    let skse_output_dir = mods_dir.join("skse");
    let skse_archive_path = cache_dir.join("skse.7z");

    if skse_output_dir.exists() {
        if prompt::confirm("SKSE already downloaded, do you want to overwrite?")? {
            fs::remove_dir_all(&skse_output_dir)?;
        } else {
            return Ok(());
        }
    }

    println!("{}", "\nSetting up SKSE\n".bold().cyan());

    fs::create_dir_all(&cache_dir)?;
    fs::create_dir_all(&mods_dir)?;

    print::print_inline_status(
        format!("Downloading SKSE to \"{}\"", cache_dir.to_string_lossy()).as_ref(),
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
