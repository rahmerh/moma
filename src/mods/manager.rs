use std::{
    fs::{self, File},
    path::PathBuf,
};

use anyhow::Context;

use crate::{
    config::Config,
    games::{
        Game,
        workspace::{self, Workspace},
    },
    types::{ModFileInfo, ModInfo},
    utils,
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

    pub fn prepare_staging_download(
        &self,
        mod_info: &ModInfo,
        file_info: &ModFileInfo,
    ) -> anyhow::Result<PathBuf> {
        let staging_mod_dir = self
            .get_staging_dir_for_mod(&mod_info.uid)
            .join(workspace::ARCHIVES);

        fs::create_dir_all(&staging_mod_dir)?;

        Ok(staging_mod_dir.join(&file_info.file_name))
    }

    pub fn register_archive(
        &self,
        mod_info: &ModInfo,
        mod_file: &ModFileInfo,
    ) -> anyhow::Result<()> {
        let mod_dir = self.get_staging_dir_for_mod(&mod_info.uid);
        let mod_info_file = mod_dir.join(MOD_INFO_FILE);

        let mut current_info = if mod_info_file.exists() {
            let content = fs::read_to_string(&mod_info_file)
                .with_context(|| format!("Failed to read {}", mod_info_file.display()))?;
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse {}", mod_info_file.display()))?
        } else {
            ModInfo {
                uid: mod_info.uid,
                name: mod_info.name.clone(),
                downloaded_archives: vec![],
            }
        };

        let already_exists = current_info
            .downloaded_archives
            .iter()
            .any(|f| f.file_name == mod_file.file_name);

        if !already_exists {
            current_info.downloaded_archives.push(mod_file.clone());
        }

        let file = File::create(&mod_info_file)?;
        serde_json::to_writer_pretty(file, &current_info)?;

        Ok(())
    }

    pub fn get_staged_mod_infos(&self) -> anyhow::Result<Vec<ModInfo>> {
        let mut mods = Vec::new();

        for entry in fs::read_dir(self.workspace.staging_dir())? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let info_path = path.join(MOD_INFO_FILE);
                if info_path.exists() {
                    let file = fs::File::open(info_path)?;
                    let mod_info: ModInfo = serde_json::from_reader(file)?;
                    mods.push(mod_info);
                }
            }
        }

        Ok(mods)
    }

    pub fn get_staged_archive_path(&self, mod_info: &ModInfo, archive_name: &str) -> PathBuf {
        self.workspace
            .staging_dir()
            .join(mod_info.uid.to_string())
            .join(workspace::ARCHIVES)
            .join(archive_name)
    }

    pub fn install_archive(&self, archives: Vec<PathBuf>) -> anyhow::Result<()> {
        let mods_dir = self.workspace.mods_dir();

        for archive in archives {
            utils::fs::extract_archive(
                &archive,
                &mods_dir.join(archive.file_name().unwrap()),
                false,
            )?;

            fs::remove_file(&archive)?;
        }

        Ok(())
    }

    fn get_staging_dir_for_mod(&self, uid: &u64) -> PathBuf {
        self.workspace.staging_dir().join(uid.to_string())
    }
}
