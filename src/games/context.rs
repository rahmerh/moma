use std::{
    fs::{self},
    io::ErrorKind,
    path::PathBuf,
};

use anyhow::Context;

use crate::{
    config::{Config, GameConfig},
    games::{GameProfile, get_game_profile_by_name},
    utils::os,
};

const OVERLAY: &str = ".overlay";
const MERGED: &str = "merged";
const WORK: &str = "work";
const ACTIVE: &str = "active";
const MODS: &str = "mods";
const SINK: &str = "sink";
const PROTON: &str = "proton";

pub struct GameContext<'a> {
    /// The user-defined configuration for this game (install path, Proton version, etc.)
    pub game: &'a GameConfig,

    /// Static game metadata and behavior (main executable, mod launcher, supported paths)
    pub profile: Box<dyn GameProfile>,

    /// The game's working directory under Moma's root (e.g. `~/.moma/skyrim`)
    pub root: PathBuf,
}

impl<'a> GameContext<'a> {
    pub fn active_dir(&self) -> PathBuf {
        self.root.join(ACTIVE)
    }

    pub fn overlay_merged_dir(&self) -> PathBuf {
        self.root.join(OVERLAY).join(MERGED)
    }

    pub fn overlay_work_dir(&self) -> PathBuf {
        self.root.join(OVERLAY).join(WORK)
    }

    pub fn mods_dir(&self) -> PathBuf {
        self.root.join(MODS)
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

    pub fn new(config: &'a Config, game_name: &str) -> anyhow::Result<Self> {
        let game = config
            .games
            .get(game_name)
            .ok_or_else(|| anyhow::anyhow!("No config found for game '{}'", game_name))?;

        let profile = get_game_profile_by_name(&game.name)
            .ok_or_else(|| anyhow::anyhow!("No game profile found for '{}'", game.name))?;

        Ok(Self {
            game,
            profile,
            root: config.work_dir.join(&game.name),
        })
    }

    pub fn prepare_file_system(&self) -> anyhow::Result<()> {
        fs::create_dir_all(self.sink_dir())?;
        fs::create_dir_all(self.proton_work_dir())?;

        os::chown_dir(&self.proton_work_dir())
            .with_context(|| "Could now set proton working dir permissions.")?;

        self.reset_overlay_dirs()?;

        Ok(())
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
