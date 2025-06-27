use std::{
    fs::{self},
    path::PathBuf,
};

use anyhow::Context;

use crate::{
    config::{Config, GameConfig},
    games::Game,
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
const TRACKING: &str = "tracking";

pub const ARCHIVES: &str = "archives";

pub const MOD_LIST_FILE: &str = "mod-list.json";

/// Represents the game-specific working directory structure
#[derive(Clone)]
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

    pub fn tracking_dir(&self) -> PathBuf {
        self.cache_dir().join(TRACKING)
    }

    pub fn overlay_dir(&self) -> PathBuf {
        self.root.join(OVERLAY)
    }

    pub fn overlay_merged_dir(&self) -> PathBuf {
        self.overlay_dir().join(MERGED)
    }

    pub fn overlay_work_dir(&self) -> PathBuf {
        self.overlay_dir().join(WORK)
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

    pub fn new(game: &Game, config: &Config) -> anyhow::Result<Self> {
        let game_config = config.games.get(game.id()).ok_or_else(|| {
            anyhow::anyhow!("No configuration found for game {}", game.to_string())
        })?;

        Ok(Self {
            game: game_config.clone(),
            root: config.work_dir.join(&game_config.name),
        })
    }

    pub fn prepare_file_system(&self) -> anyhow::Result<()> {
        // These folders can't be made by a root process, which could happen if someone ran moma launch before init.
        if permissions::is_process_root() {
            return Ok(());
        }

        let mut paths = vec![
            self.proton_work_dir(),
            self.overlay_merged_dir(),
            self.overlay_work_dir(),
            self.active_dir(),
            self.tracking_dir(),
            self.sink_dir(),
            self.mods_dir(),
        ];

        paths.sort_by_key(|p| p.components().count());

        for path in paths {
            if !path.exists() {
                println!("Creating: '{}'", path.display());
            }

            fs::create_dir_all(&path)
                .with_context(|| format!("Failed to create '{}'", path.display()))?;
        }

        Ok(())
    }
}
