use std::{
    fs::{self},
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::Context;

use crate::{
    config::{Config, GameConfig},
    utils::os::permissions,
};

const OVERLAY: &str = ".overlay";
const CACHE: &str = ".cache";
const MERGED: &str = "merged";
const WORK: &str = "work";
const ACTIVE: &str = "active";
const MODS: &str = "mods";
const STAGING: &str = "staging";
const SINK: &str = "sink";
const PROTON: &str = "proton";

pub const ARCHIVES: &str = "archives";

pub const MOD_LIST_FILE: &str = "mod-list.json";

/// Represents the game-specific working directory structure
pub struct Workspace {
    /// The user-defined configuration for this game (install path, Proton version, etc.)
    game: GameConfig,

    /// The game's working directory under Moma's root (e.g. `~/.moma/skyrim`)
    root: PathBuf,
}

impl Workspace {
    pub fn active_dir(&self) -> PathBuf {
        self.root.join(ACTIVE)
    }

    pub fn overlay_merged_dir(&self) -> PathBuf {
        self.root.join(OVERLAY).join(MERGED)
    }

    pub fn overlay_work_dir(&self) -> PathBuf {
        self.root.join(OVERLAY).join(WORK)
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.root.join(CACHE)
    }

    pub fn mods_dir(&self) -> PathBuf {
        self.root.join(MODS)
    }

    pub fn game_dir(&self) -> PathBuf {
        self.game.path.clone()
    }

    pub fn staging_dir(&self) -> PathBuf {
        self.root.join(STAGING)
    }

    pub fn sink_dir(&self) -> PathBuf {
        self.root.join(SINK)
    }

    pub fn proton_work_dir(&self) -> PathBuf {
        self.root.join(PROTON)
    }

    pub fn proton_binary(&self) -> PathBuf {
        self.game.proton_dir.join("proton")
    }

    pub fn work_dir(&self) -> PathBuf {
        self.root.clone()
    }

    pub fn new(config: &Config, game_config: &GameConfig) -> anyhow::Result<Self> {
        Ok(Self {
            game: game_config.clone(),
            root: config.work_dir.join(&game_config.name),
        })
    }

    pub fn prepare_file_system(&self) -> anyhow::Result<()> {
        let mut paths = vec![
            self.cache_dir(),
            self.proton_work_dir(),
            self.overlay_merged_dir(),
            self.overlay_work_dir(),
            self.active_dir(),
            self.sink_dir(),
        ];

        paths.sort_by_key(|p| p.components().count());

        for path in paths {
            Self::reset_dir(&path)?;
        }

        permissions::chown_dir(&self.work_dir(), true)
            .with_context(|| "Could not set working dir permissions.")?;

        Ok(())
    }

    pub fn validate_sink_is_empty(&self) -> anyhow::Result<bool> {
        let path = self.sink_dir();
        if !path.exists() {
            return Ok(true);
        }
        Ok(fs::read_dir(path)
            .with_context(|| "Sink dir could not be found.")?
            .next()
            .is_none())
    }

    fn reset_dir(path: &Path) -> anyhow::Result<()> {
        log::debug!("Resetting: {}", path.display());

        if path.exists() {
            fs::remove_dir_all(path)
                .with_context(|| format!("Failed to delete '{}'", path.display()))?;
        }

        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create '{}'", path.display()))?;

        Ok(())
    }
}
