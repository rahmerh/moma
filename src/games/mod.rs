use std::{fmt::Display, path::PathBuf};

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::{config::Config, sources::Source, utils::fs::ExpandTilde};

pub mod skyrimse;
pub mod workspace;

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

    pub fn default_mod_sources(&self) -> Vec<Source> {
        match self {
            Game::SkyrimSE => skyrimse::supported_sources(),
        }
    }

    pub fn game_mod_executable(&self) -> &'static str {
        match self {
            Game::SkyrimSE => skyrimse::game_mod_executable(),
        }
    }

    pub async fn setup(&self, config: &Config) -> anyhow::Result<()> {
        match self {
            Game::SkyrimSE => skyrimse::setup(config).await,
        }
    }
}
