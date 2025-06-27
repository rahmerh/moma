use std::{fs, path::Path};

use anyhow::{Context, bail};

use crate::games::Game;

pub const STATE_FILE_PATH: &str = "/tmp/moma_state";

pub fn current_context(state_file: &Path) -> anyhow::Result<Option<Game>> {
    if !state_file.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(state_file)
        .with_context(|| format!("Failed to read state file '{}'", state_file.display()))?;
    let trimmed = contents.trim();

    if trimmed.is_empty() {
        Ok(None)
    } else {
        match Game::from_id(trimmed) {
            Some(game) => Ok(Some(game)),
            None => bail!("Invalid game context in state file: '{}'", trimmed),
        }
    }
}

pub fn set_context(state_file: &Path, game: Game) -> anyhow::Result<()> {
    fs::write(state_file, game.id())?;
    Ok(())
}

pub fn clear_context(state_file: &Path) -> anyhow::Result<()> {
    if state_file.exists() {
        fs::remove_file(state_file).context("Could not remove state file")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup() -> TempDir {
        TempDir::new().unwrap()
    }

    #[test]
    fn current_context_should_return_none_when_state_file_is_non_existent() {
        // Act
        let context = current_context(&PathBuf::from("temp")).unwrap();

        // Assert
        assert!(context.is_none());
    }

    #[test]
    fn current_context_should_return_none_when_state_file_is_empty() {
        // Arrange
        let tmp_dir = setup();
        let state_file_path = tmp_dir.path().join("state-file");
        fs::write(&state_file_path, "").unwrap();

        // Act
        let context = current_context(&state_file_path).unwrap();

        // Assert
        assert!(context.is_none());
    }

    #[test]
    fn current_context_should_return_err_when_state_file_is_invalid() {
        // Arrange
        let tmp_dir = setup();
        let state_file_path = tmp_dir.path().join("state-file");
        fs::write(&state_file_path, "invalid").unwrap();

        // Act
        let result = current_context(&state_file_path);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn current_context_should_return_game_when_state_file_is_valid() {
        // Arrange
        let expected = Game::SkyrimSE;

        let tmp_dir = setup();
        let state_file_path = tmp_dir.path().join("state-file");
        fs::write(&state_file_path, expected.id()).unwrap();

        // Act
        let result = current_context(&state_file_path).unwrap();

        // Assert
        assert!(result.is_some());
        assert_eq!(result.unwrap().id(), expected.id());
    }

    #[test]
    fn set_context_should_create_file_if_it_doesnt_exist() {
        // Arrange
        let tmp_dir = setup();
        let state_file_path = tmp_dir.path().join("state-file");

        // Act
        let result = set_context(&state_file_path, Game::SkyrimSE);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn set_context_should_write_game_id_when_called() {
        // Arrange
        let tmp_dir = setup();
        let state_file_path = tmp_dir.path().join("state-file");

        // Act
        set_context(&state_file_path, Game::SkyrimSE).unwrap();

        // Assert
        let actual = fs::read_to_string(state_file_path).unwrap();
        assert_eq!(actual, Game::SkyrimSE.id());
    }

    #[test]
    fn clear_context_should_do_nothing_if_state_file_non_existent() {
        // Arrange
        let tmp_dir = setup();
        let state_file_path = tmp_dir.path().join("state-file");

        // Act
        let result = clear_context(&state_file_path);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn clear_context_should_remove_state_file() {
        // Arrange
        let tmp_dir = setup();
        let state_file_path = tmp_dir.path().join("state-file");
        fs::write(&state_file_path, Game::SkyrimSE.id()).unwrap();

        // Act
        let result = clear_context(&state_file_path);

        // Assert
        assert!(result.is_ok());
        assert!(!state_file_path.exists());
    }
}
