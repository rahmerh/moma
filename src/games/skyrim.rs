use anyhow::Context;
use dialoguer::Confirm;
use once_cell::sync::Lazy;
use owo_colors::OwoColorize;
use pelite::{FileMap, PeFile, resources::version_info::VersionInfo};
use reqwest::blocking::get;
use std::{
    collections::HashMap,
    fs::{self, File, create_dir_all},
    io::copy,
    path::{Path, PathBuf},
};

use crate::{
    config::{CACHE_DIR_NAME, Config, GameConfig, MODS_DIR_NAME},
    mod_platforms::ModPlatformKind,
    utils::{
        fs::{ExpandTilde, extract_archive},
        print::print_inline_status,
        theme,
    },
};

use super::GameProfile;

pub struct SkyrimSe;

static SKSE_DOWNLOADS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("1.5", "https://skse.silverlock.org/beta/skse64_2_00_20.7z"),
        ("1.6", "https://skse.silverlock.org/beta/skse64_2_02_06.7z"),
    ])
});

impl SkyrimSe {
    fn determine_game_version(&self, game_path: &Path) -> anyhow::Result<String> {
        let expanded = shellexpand::tilde(&game_path.to_string_lossy()).to_string();

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

        Ok(game_version)
    }
}

impl GameProfile for SkyrimSe {
    fn name(&self) -> &'static str {
        "Skyrim"
    }

    fn default_game_path(&self, steam_dir: &Path) -> PathBuf {
        steam_dir
            .join("steamapps")
            .join("common")
            .join("Skyrim Special Edition")
    }

    fn game_executable(&self) -> &'static str {
        "SkyrimSE.exe"
    }

    fn game_mod_executable(&self) -> &'static str {
        "skse64_loader.exe"
    }

    fn setup_modding(&self, config: &Config, game_config: &GameConfig) -> anyhow::Result<()> {
        let theme = theme::default_theme();

        let confirmed = Confirm::with_theme(&theme)
            .with_prompt("Do you want to setup SKSE?")
            .interact()?;

        if !confirmed {
            println!("{}", "\nSkipping SKSE setup.".yellow());
            return Ok(());
        }

        let game_work_dir = config.work_dir.join(self.name().to_lowercase()).expand();

        let cache_dir = game_work_dir.join(CACHE_DIR_NAME);
        let mods_dir = game_work_dir.join(MODS_DIR_NAME);

        let skse_output_dir = mods_dir.join("skse");
        let skse_archive_path = cache_dir.join("skse.7z");

        if skse_output_dir.exists() {
            let confirmed = Confirm::with_theme(&theme)
                .with_prompt("SKSE already downloaded, do you want to overwrite?")
                .interact()?;

            if confirmed {
                fs::remove_dir_all(&skse_output_dir)?;
            } else {
                return Ok(());
            }
        }

        println!("{}", "\nSetting up SKSE\n".bold().cyan());

        create_dir_all(&cache_dir)?;
        create_dir_all(&mods_dir)?;

        print_inline_status(
            format!("Downloading SKSE to \"{}\"", cache_dir.to_string_lossy()).as_ref(),
        )?;

        let game_version = self.determine_game_version(&game_config.path)?;
        let url = SKSE_DOWNLOADS
            .get(&game_version[..3])
            .with_context(|| format!("Unsupported game version: {}", game_version))?;

        let mut response = get(url.to_string())?;
        let mut out = File::create(&skse_archive_path)?;
        copy(&mut response, &mut out)?;

        print_inline_status("Extracting archive...")?;

        extract_archive(&skse_archive_path, &skse_output_dir, true)?;

        print_inline_status("Done!")?;

        println!(
            "{}",
            "\n\nSKSE installed successfully\n"
                .bold()
                .underline()
                .cyan()
        );

        Ok(())
    }

    fn supported_mod_platforms(&self) -> Vec<ModPlatformKind> {
        vec![ModPlatformKind::Nexus]
    }
}
