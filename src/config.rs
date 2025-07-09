use anyhow::bail;
use libc::{getpwuid, uid_t};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    ffi::CStr,
    path::{Path, PathBuf},
};

use crate::{
    games::game::Game,
    ui::prompt,
    utils::{fs::ExpandTilde, state},
};

pub const CACHE_DIR_NAME: &str = ".cache";
pub const MODS_DIR_NAME: &str = "mods";

#[derive(Serialize, Deserialize)]
pub struct Config {
    games: HashMap<String, GameConfig>,
    work_dir: PathBuf,
    steam_dir: PathBuf,
    state_file: PathBuf,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameConfig {
    /// Game for which config this is.
    pub game: Game,

    /// Absolute path to the base game's installation directory
    pub path: PathBuf,

    /// Path to the Proton version used to launch this game
    pub proton_dir: PathBuf,

    /// Environment variables to be set before game launch
    pub env: Option<HashMap<String, String>>,
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
        let path = match Self::resolve_config_file_path("config.toml") {
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

    pub fn game_config_for(&self, game: &Game) -> anyhow::Result<&GameConfig> {
        self.games.get(game.id()).ok_or_else(|| anyhow::anyhow!(""))
    }

    pub fn add_game(&mut self, game_config: GameConfig) -> anyhow::Result<()> {
        self.games
            .insert(game_config.game.id().to_string(), game_config);
        self.save()
    }

    pub fn base_working_dir(&self) -> PathBuf {
        self.work_dir.clone()
    }

    pub fn steam_dir(&self) -> PathBuf {
        self.steam_dir.clone()
    }

    pub fn state_file(&self) -> PathBuf {
        self.state_file.clone()
    }

    fn save(&self) -> anyhow::Result<()> {
        let path = match Self::resolve_config_file_path("config.toml") {
            Some(path) => path,
            None => bail!("Failed to resolve config path"),
        };
        let parent = path.parent().unwrap();
        std::fs::create_dir_all(parent)?;
        let toml = toml::to_string_pretty(self)?;
        std::fs::write(&path, toml)?;

        Ok(())
    }

    fn determine_steam_dir() -> anyhow::Result<PathBuf> {
        let candidates = [
            "~/.steam/steam",
            "~/.var/app/com.valvesoftware.Steam/.steam/steam",
            "~/.local/share/Steam",
        ];

        for candidate in &candidates {
            let expanded = PathBuf::from(candidate).expand();
            if Self::is_valid_steam_dir(&expanded) {
                return Ok(expanded);
            }
        }

        loop {
            let path = prompt::path("Enter your Steam installation directory", None)?;
            if Self::is_valid_steam_dir(&path) {
                return Ok(path.clone());
            } else {
                println!("{}", "Not a valid Steam directory, please try again.".red());
            }
        }
    }

    fn is_valid_steam_dir(path: &Path) -> bool {
        path.join("steamapps").is_dir()
    }

    pub fn resolve_config_file_path(filename: &str) -> Option<PathBuf> {
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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            games: HashMap::new(),
            work_dir: PathBuf::from("~/.moma").expand(),
            steam_dir: Self::determine_steam_dir().unwrap(),
            state_file: PathBuf::from(state::DEFAULT_STATE_FILE_PATH),
        }
    }
}

#[cfg(test)]
impl Config {
    pub fn test_with_config(work_dir: PathBuf, game_config: GameConfig) -> Self {
        let mut games = HashMap::new();
        games.insert(game_config.game.id().to_string(), game_config);

        Self {
            games,
            steam_dir: work_dir.join("steam"),
            state_file: work_dir.join("state.json"),
            work_dir: work_dir.join("working"),
        }
    }

    pub fn test(work_dir: PathBuf) -> Self {
        Self {
            games: HashMap::new(),
            steam_dir: work_dir.join("steam"),
            state_file: work_dir.join("state.json"),
            work_dir: work_dir.join("working"),
        }
    }
}
