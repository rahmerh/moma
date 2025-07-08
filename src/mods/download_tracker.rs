use std::{
    fs::{self, File},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::{
    games::workspace::Workspace,
    mods::mod_list_store::ModListStore,
    types::{DownloadProgress, FileStatus},
};

#[derive(Clone)]
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
                            // Age in seconds, allow idle for 1 min.
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

                    if let Err(e) = fs::remove_file(&tracking_file) {
                        if e.kind() != ErrorKind::NotFound {
                            return Err(e.into());
                        }
                    }

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

    pub async fn stream_to_file_with_tracking<R: AsyncRead + Unpin>(
        &self,
        mut stream: R,
        dest_path: &Path,
        total_size: u64,
        progress_file: &Path,
        file_name: String,
    ) -> anyhow::Result<()> {
        let mut file = File::create(dest_path)?;
        let mut buffer = [0u8; 8192];

        let start_unix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let mut downloaded: u64 = 0;
        let mut last_written = Instant::now();

        loop {
            let read_bytes = stream.read(&mut buffer).await?;
            if read_bytes == 0 {
                break;
            }

            file.write_all(&buffer[..read_bytes])?;
            downloaded += read_bytes as u64;

            if last_written.elapsed().as_millis() > 500 {
                let updated_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

                let progress = DownloadProgress {
                    file_name: file_name.clone(),
                    progress_bytes: downloaded,
                    total_bytes: total_size,
                    started_at: start_unix,
                    updated_at,
                };

                fs::write(progress_file, serde_json::to_string_pretty(&progress)?)?;
                last_written = Instant::now();
            }
        }

        if let Err(e) = fs::remove_file(progress_file) {
            if e.kind() != ErrorKind::NotFound {
                return Err(e.into());
            }
        }

        Ok(())
    }

    pub fn ensure_tracking_file(&self, file_uid: u64) -> anyhow::Result<PathBuf> {
        let tracking_file = self.tracking_file(file_uid);

        if !tracking_file.exists() {
            File::create(&tracking_file)?;
        }

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
        types::{Mod, ModArchive},
    };
    use std::{fs::OpenOptions, io::Cursor};
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

        let config = Config::test_with_config(tmp_dir.path().to_owned(), game_config);

        Workspace::new(&game, &config)
    }

    #[test]
    fn reset_stuck_downloads_should_return_zero_when_mod_list_empty() -> anyhow::Result<()> {
        // Arrange
        let workspace = setup_workspace()?;
        let mod_list_store = ModListStore::new(workspace.clone());
        let sut = DownloadTracker::new(workspace.clone(), mod_list_store);

        // Act
        let result = sut.reset_stuck_downloads();

        // Assert
        assert!(result.is_ok());

        let actual = result.unwrap();

        assert_eq!(actual, 0);

        Ok(())
    }

    #[test]
    fn reset_stuck_downloads_should_return_zero_when_one_archive_already_downloaded()
    -> anyhow::Result<()> {
        // Arrange
        let workspace = setup_workspace()?;
        workspace.prepare_file_system()?;

        let mod_list_store = ModListStore::new(workspace.clone());
        let sut = DownloadTracker::new(workspace.clone(), mod_list_store.clone());

        let m = Mod {
            uid: 1,
            name: "Test mod".to_string(),
            archives: vec![],
        };

        let archive = ModArchive {
            file_uid: 2,
            file_name: "test.7z".to_string(),
            status: FileStatus::Downloaded,
            archive_path: None,
        };

        mod_list_store.add_archive(&m, archive)?;

        // Act
        let result = sut.reset_stuck_downloads();

        // Assert
        assert!(result.is_ok());

        let actual = result.unwrap();

        assert_eq!(actual, 0);

        Ok(())
    }

    #[test]
    fn reset_stuck_downloads_should_mark_archive_as_failed_when_tracking_file_is_missing()
    -> anyhow::Result<()> {
        // Arrange
        let workspace = setup_workspace()?;
        workspace.prepare_file_system()?;

        let mod_list_store = ModListStore::new(workspace.clone());
        let sut = DownloadTracker::new(workspace.clone(), mod_list_store.clone());

        let m = Mod {
            uid: 1,
            name: "Test mod".to_string(),
            archives: vec![],
        };

        let archive = ModArchive {
            file_uid: 2,
            file_name: "test.7z".to_string(),
            status: FileStatus::Downloading,
            archive_path: None,
        };

        mod_list_store.add_archive(&m, archive.clone())?;

        // Act
        let result = sut.reset_stuck_downloads();

        // Assert
        assert!(result.is_ok());

        let actual = result.unwrap();
        assert_eq!(actual, 1);

        let actual_mod_list = mod_list_store.read()?;
        let actual_archive = actual_mod_list
            .mods
            .first()
            .unwrap()
            .archives
            .first()
            .unwrap();

        assert_eq!(actual_archive.file_uid, archive.file_uid);
        assert_eq!(actual_archive.file_name, archive.file_name);
        assert_eq!(actual_archive.archive_path, archive.archive_path);
        assert_eq!(
            actual_archive.status,
            FileStatus::Failed("missing tracking file".to_string())
        );

        Ok(())
    }

    #[test]
    fn reset_stuck_downloads_should_mark_archive_as_failed_when_tracking_file_is_invalid()
    -> anyhow::Result<()> {
        // Arrange
        let workspace = setup_workspace()?;
        workspace.prepare_file_system()?;

        let mod_list_store = ModListStore::new(workspace.clone());
        let sut = DownloadTracker::new(workspace.clone(), mod_list_store.clone());

        let m = Mod {
            uid: 1,
            name: "Test mod".to_string(),
            archives: vec![],
        };

        let archive = ModArchive {
            file_uid: 2,
            file_name: "test.7z".to_string(),
            status: FileStatus::Downloading,
            archive_path: None,
        };

        mod_list_store.add_archive(&m, archive.clone())?;
        let tracking_file = sut.tracking_file(archive.file_uid);
        fs::write(tracking_file, "Invalid!")?;

        // Act
        let result = sut.reset_stuck_downloads();

        // Assert
        assert!(result.is_ok());

        let actual = result.unwrap();
        assert_eq!(actual, 1);

        let actual_mod_list = mod_list_store.read()?;
        let actual_archive = actual_mod_list
            .mods
            .first()
            .unwrap()
            .archives
            .first()
            .unwrap();

        assert_eq!(actual_archive.file_uid, archive.file_uid);
        assert_eq!(actual_archive.file_name, archive.file_name);
        assert_eq!(actual_archive.archive_path, archive.archive_path);
        assert_eq!(
            actual_archive.status,
            FileStatus::Failed("invalid tracking file".to_string())
        );

        Ok(())
    }

    #[test]
    fn reset_stuck_downloads_should_mark_archive_as_failed_when_last_update_was_more_than_60_seconds_ago()
    -> anyhow::Result<()> {
        // Arrange
        let workspace = setup_workspace()?;
        workspace.prepare_file_system()?;

        let mod_list_store = ModListStore::new(workspace.clone());
        let sut = DownloadTracker::new(workspace.clone(), mod_list_store.clone());

        let m = Mod {
            uid: 1,
            name: "Test mod".to_string(),
            archives: vec![],
        };

        let archive = ModArchive {
            file_uid: 2,
            file_name: "test.7z".to_string(),
            status: FileStatus::Downloading,
            archive_path: None,
        };

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        mod_list_store.add_archive(&m, archive.clone())?;
        let progress = DownloadProgress {
            file_name: archive.file_name.to_string(),
            progress_bytes: 1,
            started_at: 1,
            total_bytes: 1,
            updated_at: now - 61,
        };

        let tracking_file_opened = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&sut.tracking_file(archive.file_uid))?;
        serde_json::to_writer_pretty(tracking_file_opened, &progress)?;

        // Act
        let result = sut.reset_stuck_downloads();

        // Assert
        assert!(result.is_ok());

        let actual = result.unwrap();
        assert_eq!(actual, 1);

        let actual_mod_list = mod_list_store.read()?;
        let actual_archive = actual_mod_list
            .mods
            .first()
            .unwrap()
            .archives
            .first()
            .unwrap();

        assert_eq!(actual_archive.file_uid, archive.file_uid);
        assert_eq!(actual_archive.file_name, archive.file_name);
        assert_eq!(actual_archive.archive_path, archive.archive_path);
        assert_eq!(
            actual_archive.status,
            FileStatus::Failed("interrupted".to_string())
        );

        Ok(())
    }

    #[test]
    fn reset_stuck_downloads_should_return_zero_when_last_update_was_less_than_60_seconds_ago()
    -> anyhow::Result<()> {
        // Arrange
        let workspace = setup_workspace()?;
        workspace.prepare_file_system()?;

        let mod_list_store = ModListStore::new(workspace.clone());
        let sut = DownloadTracker::new(workspace.clone(), mod_list_store.clone());

        let m = Mod {
            uid: 1,
            name: "Test mod".to_string(),
            archives: vec![],
        };

        let archive = ModArchive {
            file_uid: 2,
            file_name: "test.7z".to_string(),
            status: FileStatus::Downloading,
            archive_path: None,
        };

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        mod_list_store.add_archive(&m, archive.clone())?;
        let progress = DownloadProgress {
            file_name: archive.file_name.to_string(),
            progress_bytes: 1,
            started_at: 1,
            total_bytes: 1,
            updated_at: now - 59,
        };

        let tracking_file_opened = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&sut.tracking_file(archive.file_uid))?;
        serde_json::to_writer_pretty(tracking_file_opened, &progress)?;

        // Act
        let result = sut.reset_stuck_downloads();

        // Assert
        assert!(result.is_ok());

        let actual = result.unwrap();
        assert_eq!(actual, 0);

        Ok(())
    }

    #[tokio::test]
    async fn stream_should_write_file_contents() -> anyhow::Result<()> {
        // Arrange
        let workspace = setup_workspace()?;
        workspace.prepare_file_system()?;

        let mod_list_store = ModListStore::new(workspace.clone());
        let sut = DownloadTracker::new(workspace.clone(), mod_list_store.clone());

        let data = b"hello world this is a test";
        let total_size = data.len() as u64;

        let tmp = TempDir::new().unwrap();
        let dest_path = tmp.path().join("out.txt");
        let progress_file = tmp.path().join("progress.json");

        // Act
        sut.stream_to_file_with_tracking(
            Cursor::new(&data[..]),
            &dest_path,
            total_size,
            &progress_file,
            "test.txt".to_string(),
        )
        .await
        .unwrap();

        // Assert
        let written = fs::read(&dest_path).unwrap();
        assert_eq!(written, data);

        // Progress file should have been deleted
        assert!(!progress_file.exists());

        Ok(())
    }

    #[tokio::test]
    async fn stream_should_create_and_update_progress_file() -> anyhow::Result<()> {
        // Arrange
        let workspace = setup_workspace()?;
        workspace.prepare_file_system()?;

        let mod_list_store = ModListStore::new(workspace.clone());
        let sut = DownloadTracker::new(workspace.clone(), mod_list_store.clone());

        let data = vec![42u8; 8192 * 4];
        let total_size = data.len() as u64;

        let tmp = TempDir::new().unwrap();
        let dest_path = tmp.path().join("out.txt");
        let progress_path = tmp.path().join("progress.json");

        // Act
        sut.stream_to_file_with_tracking(
            Cursor::new(data.clone()),
            &dest_path,
            total_size,
            &progress_path,
            "file.bin".to_string(),
        )
        .await
        .unwrap();

        // Assert
        let written = fs::read(&dest_path).unwrap();
        assert_eq!(written, data);

        assert!(!progress_path.exists());

        Ok(())
    }

    #[tokio::test]
    async fn stream_should_handle_empty_input() -> anyhow::Result<()> {
        // Arrange
        let workspace = setup_workspace()?;
        workspace.prepare_file_system()?;

        let mod_list_store = ModListStore::new(workspace.clone());
        let sut = DownloadTracker::new(workspace.clone(), mod_list_store.clone());

        let tmp = TempDir::new().unwrap();
        let dest_path = tmp.path().join("out.txt");
        let progress_path = tmp.path().join("progress.json");

        // Act
        sut.stream_to_file_with_tracking(
            Cursor::new(vec![]),
            &dest_path,
            0,
            &progress_path,
            "empty.txt".to_string(),
        )
        .await
        .unwrap();

        // Assert
        assert_eq!(fs::read(&dest_path).unwrap(), Vec::<u8>::new());
        assert!(!progress_path.exists());

        Ok(())
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

    #[test]
    fn ensure_tracking_file_should_create_tracking_file_if_non_existent() -> anyhow::Result<()> {
        // Arrange
        let workspace = setup_workspace()?;
        workspace.prepare_file_system()?;

        let mod_list_store = ModListStore::new(workspace.clone());
        let sut = DownloadTracker::new(workspace.clone(), mod_list_store);

        let expected = workspace.tracking_dir().join("1.json");

        assert!(!expected.exists());

        // Act
        let result = sut.ensure_tracking_file(1);

        // Assert
        assert!(result.is_ok());
        assert!(expected.exists());

        Ok(())
    }

    #[test]
    fn ensure_tracking_file_should_not_touch_tracking_file_if_already_exists() -> anyhow::Result<()>
    {
        // Arrange
        let workspace = setup_workspace()?;
        workspace.prepare_file_system()?;

        let mod_list_store = ModListStore::new(workspace.clone());
        let sut = DownloadTracker::new(workspace.clone(), mod_list_store);

        let expected = workspace.tracking_dir().join("1.json");

        let content = "Content!";
        fs::write(&expected, content)?;

        // Act
        let result = sut.ensure_tracking_file(1);

        // Assert
        assert!(result.is_ok());

        let actual_content = fs::read_to_string(&expected)?;

        assert_eq!(actual_content, content);

        Ok(())
    }

    #[test]
    fn ensure_tracking_file_should_return_err_when_path_is_non_existent() -> anyhow::Result<()> {
        // Arrange
        let workspace = setup_workspace()?;

        let mod_list_store = ModListStore::new(workspace.clone());
        let sut = DownloadTracker::new(workspace.clone(), mod_list_store);

        // Act
        let result = sut.ensure_tracking_file(1);

        // Assert
        assert!(result.is_err());

        Ok(())
    }
}
