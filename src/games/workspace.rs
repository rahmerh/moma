use std::{
    fs::{self},
    io::ErrorKind,
    path::PathBuf,
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
        for path in [self.cache_dir(), self.proton_work_dir(), self.work_dir()] {
            fs::create_dir_all(path)?;
        }

        permissions::chown_dir(&self.proton_work_dir(), false)
            .with_context(|| "Could not set proton working dir permissions.")?;

        self.reset_overlay_dirs()?;

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

    fn reset_overlay_dirs(&self) -> anyhow::Result<()> {
        let merged = self.overlay_merged_dir();
        let work = self.overlay_work_dir();
        let active = self.active_dir();
        let overlay_root = self.root.join(OVERLAY);

        if overlay_root.exists() {
            for dir in [&merged, &work, &active] {
                match fs::remove_dir_all(dir) {
                    Ok(_) => {}
                    Err(err) if err.kind() == ErrorKind::NotFound => {}
                    Err(err) => {
                        return Err(err)
                            .with_context(|| format!("Failed to remove {}", dir.display()));
                    }
                }
            }
        }

        for dir in [&merged, &work, &active] {
            fs::create_dir_all(dir)
                .with_context(|| format!("Failed to create {}", dir.display()))?;
        }

        Ok(())
    }
}
