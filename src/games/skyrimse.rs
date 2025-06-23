use async_trait::async_trait;
use owo_colors::OwoColorize;
use reqwest::get;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    config::{CACHE_DIR_NAME, Config, MODS_DIR_NAME},
    sources::Source,
    ui::{print, prompt},
    utils::fs::{ExpandTilde, extract_archive},
};

use super::GameProfile;

pub struct SkyrimSe;

#[async_trait]
impl GameProfile for SkyrimSe {
    fn name(&self) -> &'static str {
        "SkyrimSE"
    }

    fn default_game_path(&self, steam_dir: &Path) -> PathBuf {
        steam_dir
            .join("steamapps")
            .join("common")
            .join("Skyrim Special Edition")
    }

    fn game_mod_executable(&self) -> &'static str {
        "skse64_loader.exe"
    }

    async fn setup_modding(&self, config: &Config) -> anyhow::Result<()> {
        if !prompt::confirm("Do you want to setup SKSE?")? {
            println!("{}", "\nSkipping SKSE setup.".yellow());
            return Ok(());
        }

        let game_work_dir = config.work_dir.join(self.name().to_lowercase()).expand();

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

        extract_archive(&skse_archive_path, &skse_output_dir, true)?;

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

    fn supported_sources(&self) -> Vec<Source> {
        vec![Source::Nexus]
    }
}
