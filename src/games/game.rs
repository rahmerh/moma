use std::{fmt::Display, path::PathBuf};

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::{config::Config, utils::fs::ExpandTilde};

#[derive(clap::ValueEnum, EnumIter, Clone, Serialize, Deserialize)]
pub enum Game {
    SkyrimSE,
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Game::SkyrimSE => write!(f, "Skyrim Special Edition"),
        }
    }
}

impl Game {
    pub fn id(&self) -> &'static str {
        match self {
            Game::SkyrimSE => "skyrimse",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "skyrimse" => Some(Game::SkyrimSE),
            _ => None,
        }
    }

    pub fn default_game_path(&self, config: &Config) -> anyhow::Result<PathBuf> {
        let path = match self {
            Game::SkyrimSE => config
                .steam_dir()
                .join("steamapps")
                .join("common")
                .join("Skyrim Special Edition"),
        };

        Ok(path.expand())
    }
}
