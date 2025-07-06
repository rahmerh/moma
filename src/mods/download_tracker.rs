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

    pub fn reset_stuck_downloads(&self) -> anyhow::Result<usize> {
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
                            // Age in seconds, allow idle for 5 mins.
                            if age > 300 {
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

        Ok(cancelled_files.len())
    }

    pub fn ensure_tracking_file(&self, file_uid: u64) -> anyhow::Result<PathBuf> {
        let tracking_file = self.tracking_file(file_uid);

        File::create(&tracking_file)?;

        Ok(tracking_file)
    }

    pub fn tracking_file(&self, file_uid: u64) -> PathBuf {
        self.workspace
            .tracking_dir()
            .join(format!("{}.json", file_uid.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{Config, GameConfig},
        games::Game,
    };
    use tempfile::TempDir;

    fn setup_workspace() -> anyhow::Result<Workspace> {
        let tmp_dir = TempDir::new()?;

        let game = Game::SkyrimSE;
        let game_config = GameConfig {
            game: game.clone(),
            path: PathBuf::from("/fake/skyrimse"),
            proton_dir: PathBuf::from("/fake/proton"),
            env: None,
            sources: vec![],
        };

        let mut config = Config::test(tmp_dir.path().to_owned());
        config.add_game_config(game_config)?;

        Workspace::new(&game, &config)
    }

    #[test]
    fn tracking_file_should_return_correct_path() -> anyhow::Result<()> {
        // Arrange
        let workspace = setup_workspace()?;
        let mod_list_store = ModListStore::new(workspace.clone());
        let sut = DownloadTracker::new(workspace.clone(), mod_list_store);

        // Act
        let actual = sut.tracking_file(1);

        // Assert
        assert_eq!(actual, workspace.tracking_dir().join("1.json"));

        Ok(())
    }
}
