use std::{
    fs::{self, File},
    path::PathBuf,
};

use crate::{
    config::Config,
    games::{Game, workspace::Workspace},
    types::{ModFileInfo, ModInfo},
};

pub struct Manager {
    workspace: Workspace,
}

const MOD_INFO_FILE: &str = "info.json";

impl Manager {
    pub fn new(game: &Game, config: &Config) -> anyhow::Result<Self> {
        let game_config = config.games.get(game.id()).ok_or_else(|| {
            anyhow::anyhow!("No configuration found for game {}", game.to_string())
        })?;
        let workspace = Workspace::new(config, game_config)?;

        Ok(Self { workspace })
    }

    pub fn prepare_staging_download(&self, file_info: &ModFileInfo) -> anyhow::Result<PathBuf> {
        let staging_mod_dir = self.get_staging_dir_for_mod(&file_info.uid).join("archive");

        fs::create_dir_all(&staging_mod_dir)?;

        Ok(staging_mod_dir.join(&file_info.file_name))
    }

    pub fn write_mod_info(&self, uid: u64, mod_info: &ModInfo) -> anyhow::Result<()> {
        let mod_info_file = self.get_staging_dir_for_mod(&uid).join(MOD_INFO_FILE);

        let file = File::create(mod_info_file)?;
        serde_json::to_writer_pretty(file, mod_info)?;
        Ok(())
    }

    pub fn get_staged_mod_infos(&self) -> anyhow::Result<Vec<ModInfo>> {
        let mut mods = Vec::new();

        for entry in fs::read_dir(self.workspace.staging_dir())? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let info_path = path.join("info.json");
                if info_path.exists() {
                    let file = fs::File::open(info_path)?;
                    let mod_info: ModInfo = serde_json::from_reader(file)?;
                    mods.push(mod_info);
                }
            }
        }

        Ok(mods)
    }

    fn get_staging_dir_for_mod(&self, uid: &u64) -> PathBuf {
        self.workspace.staging_dir().join(uid.to_string())
    }
}
