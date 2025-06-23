use std::{fs, path::PathBuf};

use crate::{
    config::Config,
    games::{Game, workspace::Workspace},
    types::ModFileInfo,
};

pub struct Manager {
    workspace: Workspace,
}

impl Manager {
    pub fn new(game: &Game, config: &Config) -> anyhow::Result<Self> {
        let game_config = config.games.get(game.id()).ok_or_else(|| {
            anyhow::anyhow!("No configuration found for game {}", game.to_string())
        })?;
        let workspace = Workspace::new(config, game_config)?;

        Ok(Self { workspace })
    }

    pub fn prepare_staging_download(&self, file_info: &ModFileInfo) -> anyhow::Result<PathBuf> {
        let staging_mod_dir = self
            .workspace
            .staging_dir()
            .join(&file_info.uid.to_string())
            .join("archive");

        fs::create_dir_all(&staging_mod_dir)?;

        Ok(staging_mod_dir.join(&file_info.file_name))
    }
}
