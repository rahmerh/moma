use dialoguer::Input;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::utils::{fs::ExpandTilde, os::chown_dir, theme};

pub const CACHE_DIR_NAME: &str = ".cache";
pub const MODS_DIR_NAME: &str = "mods";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub games: HashMap<String, GameConfig>,
    pub work_dir: PathBuf,
    pub steam_dir: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameConfig {
    /// Absolute path to the base game's installation directory
    pub path: PathBuf,

    /// Identifier used to match the game profile (e.g. "SkyrimSE")
    pub name: String,

    /// Path to the Proton version used to launch this game
    pub proton_dir: PathBuf,
}

impl Config {
    pub fn default_path() -> PathBuf {
        dirs_next::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("moma/config.toml")
    }

    pub fn load_or_default() -> Self {
        let path = Self::default_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read config at {}: {}", path.display(), e));
            toml::from_str(&content).expect("Failed to parse config")
        } else {
            let config = Self::default();
            config.save().expect("Failed to generate default config");
            config
        }
    }

    pub fn get_steam_dir(&mut self) -> anyhow::Result<PathBuf> {
        let steam_dir = match &self.steam_dir {
            Some(dir) => dir.clone(),
            None => self.resolve_and_store_steam_dir()?,
        };

        Ok(steam_dir)
    }

    pub fn resolve_and_store_steam_dir(&mut self) -> anyhow::Result<PathBuf> {
        let dir = Self::determine_steam_dir(self)?;
        self.steam_dir = Some(dir.clone());
        self.save()?;
        Ok(dir)
    }

    fn determine_steam_dir(&mut self) -> anyhow::Result<PathBuf> {
        if let Some(ref dir) = self.steam_dir {
            return Ok(dir.expand());
        }

        let candidates = [
            "~/.steam/steam",
            "~/.var/app/com.valvesoftware.Steam/.steam/steam",
            "~/.local/share/Steam",
        ];

        for candidate in &candidates {
            let expanded = PathBuf::from(candidate).expand();
            if is_valid_steam_dir(&expanded) {
                return Ok(expanded);
            }
        }

        let manual_path = Input::with_theme(&theme::default_theme())
            .with_prompt("Enter your Steam installation directory")
            .validate_with(|input: &String| {
                let path = Path::new(input.trim()).expand();
                if is_valid_steam_dir(&path) {
                    Ok(())
                } else {
                    Err("Not a valid Steam installation directory.")
                }
            })
            .interact_text()?;

        let steam_dir = PathBuf::from(manual_path.trim()).expand();
        self.steam_dir = Some(steam_dir.clone());

        Ok(steam_dir)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::default_path();
        let parent = path.parent().unwrap();
        std::fs::create_dir_all(parent)?;
        let toml = toml::to_string_pretty(self).unwrap();
        std::fs::write(&path, toml)?;
        chown_dir(&parent, true)
    }
}

fn is_valid_steam_dir(path: &Path) -> bool {
    path.join("steamapps").is_dir()
}

impl Default for Config {
    fn default() -> Self {
        let path = PathBuf::from("~/.moma").expand().display().to_string();
        let work_dir = PathBuf::from(path);
        Self {
            games: HashMap::new(),
            work_dir,
            steam_dir: None,
        }
    }
}
