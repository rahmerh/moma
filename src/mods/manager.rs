use std::{
    fs::{self, File, OpenOptions},
    path::PathBuf,
};

use anyhow::{Context, bail};

use crate::{
    config::Config,
    games::{
        Game,
        workspace::{self, Workspace},
    },
    types::{FileStatus, Mod, ModArchive, ModList},
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
        workspace.prepare_file_system()?;

        Ok(Self { workspace })
    }

    pub fn get_cache_path(&self) -> PathBuf {
        self.workspace.cache_dir()
    }

    pub fn get_staged_mod_infos(&self) -> anyhow::Result<Vec<Mod>> {
        let mut mods = Vec::new();

        for entry in fs::read_dir(self.workspace.staging_dir())? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let info_path = path.join(MOD_INFO_FILE);
                if info_path.exists() {
                    let file = fs::File::open(info_path)?;
                    let mod_info: Mod = serde_json::from_reader(file)?;
                    mods.push(mod_info);
                }
            }
        }

        Ok(mods)
    }

    pub fn is_archive_present(&self, file_uid: u64) -> anyhow::Result<bool> {
        let mod_list = self.read_mod_list()?;

        let present = mod_list.mods.iter().any(|m| {
            m.downloaded_archives
                .iter()
                .any(|a| a.file_uid == file_uid && a.status != FileStatus::Downloading)
        });

        Ok(present)
    }

    pub fn mark_archive_status(
        &self,
        mod_info: &Mod,
        archive: &ModArchive,
        status: FileStatus,
    ) -> anyhow::Result<()> {
        let mut mod_list = self.read_mod_list()?;

        if let Some(existing_mod) = mod_list.mods.iter_mut().find(|m| m.uid == mod_info.uid) {
            if let Some(existing_archive) = existing_mod
                .downloaded_archives
                .iter_mut()
                .find(|a| a.file_uid == archive.file_uid)
            {
                existing_archive.status = status;
            } else {
                let mut new_archive = archive.clone();
                new_archive.status = status;
                existing_mod.downloaded_archives.push(new_archive);
            }
        } else {
            let mut new_archive = archive.clone();
            new_archive.status = status;

            let new_mod = Mod {
                uid: mod_info.uid,
                name: mod_info.name.clone(),
                downloaded_archives: vec![new_archive],
            };

            mod_list.mods.push(new_mod);
        }

        self.write_mod_list(&mod_list)
    }

    pub fn stage_archive(&self, mod_info: Mod, archive: ModArchive) -> anyhow::Result<()> {
        let target = self
            .workspace
            .staging_dir()
            .join(&mod_info.uid.to_string())
            .join(workspace::ARCHIVES);

        if !target.exists() {
            fs::create_dir_all(&target)?;
        }

        println!(
            "{}",
            archive.archive_path.join(&archive.file_name).display()
        );

        fs::rename(
            archive.archive_path.join(&archive.file_name),
            target.join(&archive.file_name),
        )
        .with_context(|| "Could not move archive to staging directory.")
    }

    pub fn install_archive_for_mod(
        &self,
        mod_name: &str,
        archive_path: &PathBuf,
    ) -> anyhow::Result<()> {
        let mods_dir = self.workspace.mods_dir();

        // TODO: Detect if folder needs to be flattened
        utils::fs::extract_archive(&archive_path, &mods_dir.join(mod_name), false)?;
        fs::remove_file(&archive_path)?;

        // TODO: Update mod-list.json

        Ok(())
    }

    fn read_mod_list(&self) -> anyhow::Result<ModList> {
        let mod_list_path = self.workspace.work_dir().join(workspace::MOD_LIST_FILE);

        let mod_list = match fs::read_to_string(mod_list_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => ModList::default(),
            Err(e) => return Err(e.into()),
        };

        Ok(mod_list)
    }

    fn update_mod_list_entry(&self, mod_entry: &Mod) -> anyhow::Result<()> {
        let mut mod_list = self.read_mod_list()?;

        if let Some(found) = mod_list.mods.iter_mut().find(|m| m.uid == mod_entry.uid) {
            *found = mod_entry.clone();
        } else {
            mod_list.mods.push(mod_entry.clone());
        }

        self.write_mod_list(&mod_list)
    }

    fn write_mod_list(&self, mod_list: &ModList) -> anyhow::Result<()> {
        let mod_list_path = self.workspace.work_dir().join(workspace::MOD_LIST_FILE);

        let mod_list_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&mod_list_path)
            .with_context(|| "Failed to open mod list file for writing")?;

        serde_json::to_writer_pretty(mod_list_file, mod_list)?;

        Ok(())
    }
}
