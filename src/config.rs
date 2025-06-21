use anyhow::bail;
use dialoguer::Input;
use libc::{getpwuid, uid_t};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    ffi::CStr,
    path::{Path, PathBuf},
};

use crate::{
    mod_platforms::ModPlatformKind,
    utils::{fs::ExpandTilde, os::permissions, theme},
};

pub const CACHE_DIR_NAME: &str = ".cache";
pub const MODS_DIR_NAME: &str = "mods";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub games: HashMap<String, GameConfig>,
    pub work_dir: PathBuf,
    pub steam_dir: Option<PathBuf>,
    pub nexus_api_key: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GameConfig {
    /// Absolute path to the base game's installation directory
    pub path: PathBuf,

    /// Identifier used to match the game profile (e.g. "SkyrimSE")
    pub name: String,

    /// Path to the Proton version used to launch this game
    pub proton_dir: PathBuf,

    /// Environment variables to be set before game launch
    pub env: Option<HashMap<String, String>>,

    /// All mod platforms to be used to manage mods for this game
    pub mod_platforms: Vec<ModPlatformKind>,
}

impl GameConfig {
    pub fn get_env_vars(&self) -> HashMap<String, String> {
        let mut all_vars = HashMap::new();

        if let Some(env_map) = &self.env {
            all_vars.extend(env_map.clone());
        }

        for key in ["WAYLAND_DISPLAY", "DISPLAY", "XDG_RUNTIME_DIR"] {
            if let Some(val) = self.get_env_var_with_fallback(key) {
                all_vars.insert(key.to_string(), val);
            }
        }

        all_vars
    }

    fn get_env_var_with_fallback(&self, key: &str) -> Option<String> {
        if let Some(env_map) = &self.env {
            if let Some(val) = env_map.get(key) {
                return Some(val.clone());
            }
        }

        env::var(key).ok()
    }
}

impl Config {
    pub fn load_or_default() -> anyhow::Result<Config> {
        let path = match resolve_config_file_path("config.toml") {
            Some(path) => path,
            None => bail!("Failed to resolve config path"),
        };

        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read config at {}: {}", path.display(), e));
            Ok(toml::from_str(&content).expect("Failed to parse config"))
        } else {
            let config = Self::default();
            config.save().expect("Failed to generate default config");
            Ok(config)
        }
    }

    pub fn get_steam_dir(&mut self) -> anyhow::Result<PathBuf> {
        let steam_dir = match &self.steam_dir {
            Some(dir) => dir.clone(),
            None => self.resolve_and_store_steam_dir()?,
        };

        Ok(steam_dir)
    }

    fn resolve_and_store_steam_dir(&mut self) -> anyhow::Result<PathBuf> {
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
        let path = match resolve_config_file_path("config.toml") {
            Some(path) => path,
            None => bail!("Failed to resolve config path"),
        };
        let parent = path.parent().unwrap();
        std::fs::create_dir_all(parent)?;
        let toml = toml::to_string_pretty(self).unwrap();
        std::fs::write(&path, toml)?;
        permissions::chown_dir(&parent, true)
    }
}

fn is_valid_steam_dir(path: &Path) -> bool {
    path.join("steamapps").is_dir()
}

pub(crate) fn resolve_config_file_path(filename: &str) -> Option<PathBuf> {
    if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(config_home).join("moma").join(filename));
    }

    if let Ok(uid_str) = env::var("SUDO_UID") {
        if let Ok(uid) = uid_str.parse::<uid_t>() {
            unsafe {
                let pw = getpwuid(uid);
                if !pw.is_null() {
                    let dir = CStr::from_ptr((*pw).pw_dir).to_string_lossy().into_owned();
                    return Some(PathBuf::from(dir).join(".config/moma").join(filename));
                }
            }
        }
    }

    dirs_next::config_dir().map(|p| p.join("moma").join(filename))
}

impl Default for Config {
    fn default() -> Self {
        let path = PathBuf::from("~/.moma").expand().display().to_string();
        let work_dir = PathBuf::from(path);
        Self {
            games: HashMap::new(),
            work_dir,
            steam_dir: None,
            nexus_api_key: None,
        }
    }
}
