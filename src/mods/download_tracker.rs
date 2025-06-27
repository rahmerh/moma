use std::{
    fs::{self, File},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    games::workspace::Workspace,
    mods::mod_list_store::ModListStore,
    types::{DownloadProgress, FileStatus},
};

pub struct DownloadTracker {
    workspace: Workspace,
    mod_list_store: ModListStore,
}

impl DownloadTracker {
    pub fn new(workspace: Workspace, mod_list_store: ModListStore) -> Self {
        Self {
            workspace,
            mod_list_store,
        }
    }

    pub fn reset_stuck_downloads(&self) -> anyhow::Result<Vec<String>> {
        let mut mod_list = self.mod_list_store.read()?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let mut cancelled_files = Vec::new();

        for mod_entry in &mut mod_list.mods {
            for archive in &mut mod_entry.archives {
                if archive.status != FileStatus::Downloading {
                    continue;
                }

                let tracking_file = self.tracking_file(archive.file_uid);
                let mut is_failed = false;
                let mut reason = "";

                let contents = match fs::read_to_string(&tracking_file) {
                    Ok(c) => c,
                    Err(_) => {
                        is_failed = true;
                        reason = "missing tracking file";
                        String::new()
                    }
                };

                if !is_failed {
                    match serde_json::from_str::<DownloadProgress>(&contents) {
                        Ok(progress) => {
                            let age = now.saturating_sub(progress.updated_at);
                            if age > 60 {
                                is_failed = true;
                                reason = "interrupted";
                            }
                        }
                        Err(_) => {
                            is_failed = true;
                            reason = "invalid tracking file";
                        }
                    }
                }

                if is_failed {
                    archive.status = FileStatus::Failed(reason.into());
                    cancelled_files.push(archive.file_name.to_string());
                    let _ = fs::remove_file(&tracking_file);
                    self.mod_list_store
                        .update_archive(&mod_entry.uid, &archive.file_uid, |a| {
                            a.status = archive.status.clone()
                        })?;
                    log::warn!("Marked archive {} as failed ({})", archive.file_uid, reason);
                }
            }
        }

        Ok(cancelled_files)
    }

    pub fn ensure_tracking_file(&self, file_uid: u64) -> anyhow::Result<PathBuf> {
        let tracking_file = self.tracking_file(file_uid);

        File::create(&tracking_file)?;

        Ok(tracking_file)
    }

    fn tracking_file(&self, file_uid: u64) -> PathBuf {
        self.workspace
            .tracking_dir()
            .join(format!("{}.json", file_uid.to_string()))
    }
}
