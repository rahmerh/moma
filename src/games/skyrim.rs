use anyhow::Context;
use once_cell::sync::Lazy;
use owo_colors::OwoColorize;
use pelite::{FileMap, PeFile, resources::version_info::VersionInfo};
use reqwest::blocking::get;
use std::{
    collections::HashMap,
    fs::{File, create_dir_all},
    io::copy,
    path::PathBuf,
};

use crate::config::{Config, GameConfig};

use super::GameProfile;

pub struct Skyrim;

static SKSE_DOWNLOADS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("1.5", "https://skse.silverlock.org/beta/skse64_2_00_20.7z"),
        ("1.6", "https://skse.silverlock.org/beta/skse64_2_02_06.7z"),
    ])
});

impl GameProfile for Skyrim {
    fn name(&self) -> &'static str {
        "Skyrim"
    }

    fn default_path(&self) -> PathBuf {
        PathBuf::from("~/.local/share/Steam/steamapps/common/Skyrim Special Edition")
    }

    fn game_executable(&self) -> &'static str {
        "SkyrimSE.exe"
    }

    fn setup_modding(&self, config: &Config, game_config: &GameConfig) -> anyhow::Result<()> {
        println!("{}", "\nSetting up SKSE".bold().cyan());

        let expanded = shellexpand::tilde(&game_config.path.to_string_lossy()).to_string();

        let map = FileMap::open(&format!("{}/{}", expanded, self.game_executable()))
            .with_context(|| format!("Failed to memory-map {:?}", expanded))?;
        let pe = PeFile::from_bytes(&map).context("Not a valid PE32+ executable")?;

        let resources = pe.resources().context("No resource section found")?;
        let ver_info: VersionInfo = resources
            .version_info()
            .context("No VERSIONINFO resource in executable")?;

        let game_version = ver_info
            .fixed()
            .context("Missing fixed file version info")?
            .dwFileVersion
            .to_string();

        let url = SKSE_DOWNLOADS
            .get(&game_version[..3])
            .with_context(|| format!("Unsupported game version: {}", game_version))?;

        create_dir_all(config.work_dir.join(".cache"))?;
        let mut response = get(url.to_string())?;
        let mut out = File::create(config.work_dir.join(".cache/skse.7z"))?;

        copy(&mut response, &mut out)?;

        Ok(())
    }
}
