use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
};

use anyhow::{Context, bail};

use crate::{
    games::workspace::{self, Workspace},
    types::{FileStatus, Mod, ModArchive, ModList},
    utils,
};

#[derive(Clone)]
pub struct ModListStore {
    workspace: Workspace,
}

impl ModListStore {
    pub fn new(workspace: Workspace) -> Self {
        Self { workspace }
    }

    pub fn archive_dest(&self, file_name: &str) -> PathBuf {
        self.workspace.cache_dir().join(file_name)
    }

    pub fn read(&self) -> anyhow::Result<ModList> {
        let mod_list_path = self.workspace.work_dir().join(workspace::MOD_LIST_FILE);

        let mod_list = match fs::read_to_string(mod_list_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => ModList::default(),
            Err(e) => return Err(e.into()),
        };

        Ok(mod_list)
    }

    pub fn stage_archive(&self, m: &Mod, archive: &ModArchive) -> anyhow::Result<()> {
        let target = self
            .workspace
            .staging_dir()
            .join(&m.uid.to_string())
            .join(workspace::ARCHIVES);

        if !target.exists() {
            fs::create_dir_all(&target)?;
        }

        let archive_path = archive
            .archive_path
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Archive has no path."))?;

        let target_path = target.join(archive.file_name.to_string());

        fs::rename(&archive_path, &target_path)
            .with_context(|| "Could not move archive to staging directory.")?;

        self.update_archive(&m.uid, &archive.file_uid, |a| {
            a.status = FileStatus::Downloaded;
            a.archive_path = Some(target_path);
        })?;

        Ok(())
    }

    pub fn install_archive(&self, m: &Mod, archive: &ModArchive) -> anyhow::Result<()> {
        let mods_dir = self.workspace.mods_dir();

        // TODO: Detect if folder needs to be flattened
        let archive_path = match &archive.archive_path {
            Some(a) => a,
            None => bail!("No archive path found for: '{}'", archive.file_name),
        };

        utils::fs::extract_archive(&archive_path, &mods_dir.join(m.uid.to_string()), false)?;
        fs::remove_file(&archive_path)?;

        self.update_archive(&m.uid, &archive.file_uid, |a| {
            a.status = FileStatus::Installed;
            a.archive_path = None;
        })?;

        Ok(())
    }

    pub fn add_archive(&self, mod_to_add_to: &Mod, archive: ModArchive) -> anyhow::Result<()> {
        let mut mod_list = self.read()?;

        if let Some(existing_mod) = mod_list
            .mods
            .iter_mut()
            .find(|m| m.uid == mod_to_add_to.uid)
        {
            let already_exists = existing_mod
                .archives
                .iter()
                .any(|a| a.file_uid == archive.file_uid);

            if !already_exists {
                existing_mod.archives.push(archive);
            }
        } else {
            let new_mod = Mod {
                uid: mod_to_add_to.uid,
                name: mod_to_add_to.name.to_string(),
                archives: vec![archive],
            };

            mod_list.mods.push(new_mod);
        }

        self.write(&mod_list)
    }

    pub fn update_archive<F>(&self, mod_uid: &u64, file_uid: &u64, updater: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut ModArchive),
    {
        let mut mod_list = self.read()?;

        let found = mod_list
            .mods
            .iter_mut()
            .find(|m| &m.uid == mod_uid)
            .and_then(|m| {
                m.archives
                    .iter_mut()
                    .find(|a| &a.file_uid == file_uid)
                    .map(|archive| {
                        updater(archive);
                    })
            });

        if found.is_none() {
            bail!(
                "Archive with file_uid {} under mod_uid {} not found",
                file_uid,
                mod_uid
            );
        }

        self.write(&mod_list)
    }

    pub fn get_archive_status(&self, mod_uid: u64, file_uid: u64) -> anyhow::Result<FileStatus> {
        let mod_list = self.read()?;

        let status = mod_list
            .mods
            .iter()
            .find(|m| m.uid == mod_uid)
            .and_then(|m| m.archives.iter().find(|a| a.file_uid == file_uid))
            .map(|a| a.status.clone());

        Ok(status.unwrap_or(FileStatus::Unknown))
    }

    fn write(&self, mod_list: &ModList) -> anyhow::Result<()> {
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
