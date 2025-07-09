use anyhow::bail;
use std::path::{Path, PathBuf};

use crate::{
    games::game::Game,
    utils::{fs::FileSystem, os::permissions::System},
};

pub struct Bay<'a> {
    file_system: &'a dyn FileSystem,
    system: &'a dyn System,
    working_dir: PathBuf,
}

const OVERLAY_MERGED: &str = ".overlay/merged";
const OVERLAY_WORK: &str = ".overlay/work";
const OVERLAY_ACTIVE: &str = ".overlay/active";
const SINK: &str = "sink";
const PROTON_WORKING: &str = "proton";

/// Represents the working directory for a game. Contains mods, configuration and overlay folders needed to launch this game.
impl<'a> Bay<'a> {
    pub fn new(
        file_system: &'a dyn FileSystem,
        system: &'a dyn System,
        base_work_dir: &Path,
        game: &Game,
    ) -> Self {
        Self {
            file_system,
            system,
            working_dir: base_work_dir.join(game.id()),
        }
    }

    pub fn initialize(&self) -> anyhow::Result<()> {
        // These folders shouldn't be made by a root process, this will result in issues when launching a game.
        if self.system.is_process_root() {
            bail!("Cannot prepare the bay as root.");
        }

        let paths = vec![
            self.working_dir.join(OVERLAY_MERGED),
            self.working_dir.join(OVERLAY_WORK),
            self.working_dir.join(OVERLAY_ACTIVE),
            self.working_dir.join(SINK),
            self.working_dir.join(PROTON_WORKING),
        ];

        for path in paths {
            self.file_system.create_dir_all(&path)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{fs::MockFileSystem, os::permissions::MockSystem};
    use std::path::PathBuf;

    #[test]
    fn initialize_should_return_err_when_process_is_root() -> anyhow::Result<()> {
        // Arrange
        let game = Game::SkyrimSE;
        let base_dir = PathBuf::from("/fake/path");

        let mut mock_fs = MockFileSystem::new();
        mock_fs.expect_create_dir_all().times(0);

        let mut mock_sys = MockSystem::new();
        mock_sys
            .expect_is_process_root()
            .returning(|| true)
            .times(1);

        let sut = Bay::new(&mock_fs, &mock_sys, &base_dir, &game);

        // Act
        let result = sut.initialize();

        // Assert
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn initialize_should_create_dirs_if_non_existent() -> anyhow::Result<()> {
        // Arrange
        let game = Game::SkyrimSE;
        let base_dir = PathBuf::from("/fake/path");

        let working_dir = base_dir.join(game.id());

        let expected_dirs = vec![
            working_dir.join(".overlay/merged"),
            working_dir.join(".overlay/work"),
            working_dir.join(".overlay/active"),
            working_dir.join("sink"),
            working_dir.join("proton"),
        ];

        let mut mock_fs = MockFileSystem::new();
        let mut mock_sys = MockSystem::new();
        mock_sys
            .expect_is_process_root()
            .returning(|| false)
            .times(1);

        for path in expected_dirs.iter() {
            let p = path.clone();
            mock_fs
                .expect_create_dir_all()
                .withf(move |arg| arg == &p)
                .returning(|_| Ok(()))
                .times(1);
        }

        let sut = Bay::new(&mock_fs, &mock_sys, &base_dir, &game);

        // Act
        let result = sut.initialize();

        // Assert
        assert!(result.is_ok());

        Ok(())
    }
}
