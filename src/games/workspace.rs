use std::{
    fs::{self},
    path::PathBuf,
};

use anyhow::{Context, bail};

use crate::{
    config::{Config, GameConfig},
    games::Game,
    utils::os::permissions,
};

const OVERLAY: &str = ".overlay";
const CACHE: &str = ".cache";
const MERGED: &str = "merged";
const WORK: &str = "work";
const ACTIVE: &str = "active";
const MODS: &str = "mods";
const STAGING: &str = "staging";
const SINK: &str = "sink";
const PROTON: &str = "proton";
const TRACKING: &str = "tracking";

pub const ARCHIVES: &str = "archives";

pub const MOD_LIST_FILE: &str = "mod-list.json";

/// Represents the game-specific working directory structure
#[derive(Clone)]
pub struct Workspace {
    /// The user-defined configuration for this game (install path, Proton version, etc.)
    game: GameConfig,

    /// The game's working directory under Moma's root (e.g. `~/.moma/skyrim`)
    root: PathBuf,
}

impl Workspace {
    pub fn active_dir(&self) -> PathBuf {
        self.root.join(ACTIVE)
    }

    pub fn tracking_dir(&self) -> PathBuf {
        self.cache_dir().join(TRACKING)
    }

    pub fn overlay_dir(&self) -> PathBuf {
        self.root.join(OVERLAY)
    }

    pub fn overlay_merged_dir(&self) -> PathBuf {
        self.overlay_dir().join(MERGED)
    }

    pub fn overlay_work_dir(&self) -> PathBuf {
        self.overlay_dir().join(WORK)
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.root.join(CACHE)
    }

    pub fn mods_dir(&self) -> PathBuf {
        self.root.join(MODS)
    }

    pub fn game_dir(&self) -> PathBuf {
        self.game.path.clone()
    }

    pub fn staging_dir(&self) -> PathBuf {
        self.root.join(STAGING)
    }

    pub fn sink_dir(&self) -> PathBuf {
        self.root.join(SINK)
    }

    pub fn proton_work_dir(&self) -> PathBuf {
        self.root.join(PROTON)
    }

    pub fn proton_binary(&self) -> PathBuf {
        self.game.proton_dir.join("proton")
    }

    pub fn work_dir(&self) -> PathBuf {
        self.root.clone()
    }

    pub fn new(game: &Game, config: &Config) -> anyhow::Result<Self> {
        let game_config = config.game_config_for(game)?;

        Ok(Self {
            game: game_config.clone(),
            root: config.base_working_dir().join(&game_config.game.id()),
        })
    }

    pub fn prepare_file_system(&self) -> anyhow::Result<()> {
        // These folders shouldn't be made by a root process, this will result in issues when launching a game.
        if permissions::is_process_root() {
            bail!("Cannot prepare the filesystem as root.");
        }

        let mut paths = vec![
            self.proton_work_dir(),
            self.overlay_merged_dir(),
            self.overlay_work_dir(),
            self.active_dir(),
            self.tracking_dir(),
            self.sink_dir(),
            self.mods_dir(),
        ];

        paths.sort_by_key(|p| p.components().count());

        println!("\nPreparing file system.");
        for path in paths {
            if !path.exists() {
                println!("- '{}'", path.display());
            }

            fs::create_dir_all(&path)
                .with_context(|| format!("Failed to create '{}'", path.display()))?;
        }

        println!();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::bail;
    use tempfile::TempDir;
    use walkdir::{DirEntry, WalkDir};

    fn is_leaf(entry: &DirEntry) -> anyhow::Result<&DirEntry> {
        if !entry.metadata()?.is_dir() {
            bail!("Not a dir");
        }

        for e in fs::read_dir(entry.path())? {
            if e?.path().is_dir() {
                bail!("Not a leaf");
            }
        }

        Ok(entry)
    }

    fn setup(game: &Game) -> anyhow::Result<Config> {
        let tmp_dir = TempDir::new()?;

        let game_config = GameConfig {
            game: game.clone(),
            path: PathBuf::from("/fake/skyrimse"),
            proton_dir: PathBuf::from("/fake/proton"),
            env: None,
            sources: vec![],
        };

        let mut config = Config::test(tmp_dir.path().to_owned());
        config.add_game_config(game_config)?;

        Ok(config)
    }

    #[test]
    fn new_should_return_err_when_game_config_non_existant() {
        // Arrange
        let config: Config = Config::test(TempDir::new().unwrap().path().to_owned());

        // Act
        let result = Workspace::new(&Game::SkyrimSE, &config);

        // Arrange
        assert!(result.is_err());
    }

    #[test]
    fn active_dir_should_return_correct_path() -> anyhow::Result<()> {
        // Arrange
        let game = Game::SkyrimSE;
        let config = setup(&game)?;

        let sut = Workspace::new(&game, &config)?;

        // Act
        let actual = sut.active_dir();

        // Assert
        assert_eq!(
            actual,
            config.base_working_dir().join(game.id()).join(ACTIVE)
        );

        Ok(())
    }

    #[test]
    fn tracking_dir_should_return_correct_path() -> anyhow::Result<()> {
        // Arrange
        let game = Game::SkyrimSE;
        let config = setup(&game)?;

        let sut = Workspace::new(&game, &config)?;

        // Act
        let actual = sut.tracking_dir();

        // Assert
        assert_eq!(
            actual,
            config
                .base_working_dir()
                .join(game.id())
                .join(CACHE)
                .join(TRACKING)
        );

        Ok(())
    }

    #[test]
    fn overlay_dir_should_return_correct_path() -> anyhow::Result<()> {
        // Arrange
        let game = Game::SkyrimSE;
        let config = setup(&game)?;

        let sut = Workspace::new(&game, &config)?;

        // Act
        let actual = sut.overlay_dir();

        // Assert
        assert_eq!(
            actual,
            config.base_working_dir().join(game.id()).join(OVERLAY)
        );

        Ok(())
    }

    #[test]
    fn overlay_merged_dir_should_return_correct_path() -> anyhow::Result<()> {
        // Arrange
        let game = Game::SkyrimSE;
        let config = setup(&game)?;

        let sut = Workspace::new(&game, &config)?;

        // Act
        let actual = sut.overlay_merged_dir();

        // Assert
        assert_eq!(
            actual,
            config
                .base_working_dir()
                .join(game.id())
                .join(OVERLAY)
                .join(MERGED)
        );

        Ok(())
    }

    #[test]
    fn overlay_work_dir_should_return_correct_path() -> anyhow::Result<()> {
        // Arrange
        let game = Game::SkyrimSE;
        let config = setup(&game)?;

        let sut = Workspace::new(&game, &config)?;

        // Act
        let actual = sut.overlay_work_dir();

        // Assert
        assert_eq!(
            actual,
            config
                .base_working_dir()
                .join(game.id())
                .join(OVERLAY)
                .join(WORK)
        );

        Ok(())
    }

    #[test]
    fn cache_dir_should_return_correct_path() -> anyhow::Result<()> {
        // Arrange
        let game = Game::SkyrimSE;
        let config = setup(&game)?;

        let sut = Workspace::new(&game, &config)?;

        // Act
        let actual = sut.cache_dir();

        // Assert
        assert_eq!(
            actual,
            config.base_working_dir().join(game.id()).join(CACHE)
        );

        Ok(())
    }

    #[test]
    fn mods_dir_should_return_correct_path() -> anyhow::Result<()> {
        // Arrange
        let game = Game::SkyrimSE;
        let config = setup(&game)?;

        let sut = Workspace::new(&game, &config)?;

        // Act
        let actual = sut.mods_dir();

        // Assert
        assert_eq!(actual, config.base_working_dir().join(game.id()).join(MODS));

        Ok(())
    }

    #[test]
    fn game_dir_should_return_correct_path() -> anyhow::Result<()> {
        // Arrange
        let game = Game::SkyrimSE;
        let config = setup(&game)?;

        let sut = Workspace::new(&game, &config)?;

        // Act
        let actual = sut.game_dir();

        // Assert
        let expected = config.game_config_for(&game).unwrap().path.clone();

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn staging_dir_should_return_correct_path() -> anyhow::Result<()> {
        // Arrange
        let game = Game::SkyrimSE;
        let config = setup(&game)?;

        let sut = Workspace::new(&game, &config)?;

        // Act
        let actual = sut.staging_dir();

        // Assert
        assert_eq!(
            actual,
            config.base_working_dir().join(game.id()).join(STAGING)
        );

        Ok(())
    }

    #[test]
    fn sink_dir_should_return_correct_path() -> anyhow::Result<()> {
        // Arrange
        let game = Game::SkyrimSE;
        let config = setup(&game)?;

        let sut = Workspace::new(&game, &config)?;

        // Act
        let actual = sut.sink_dir();

        // Assert
        assert_eq!(actual, config.base_working_dir().join(game.id()).join(SINK));

        Ok(())
    }

    #[test]
    fn proton_work_dir_should_return_correct_path() -> anyhow::Result<()> {
        // Arrange
        let game = Game::SkyrimSE;
        let config = setup(&game)?;

        let sut = Workspace::new(&game, &config)?;

        // Act
        let actual = sut.proton_work_dir();

        // Assert
        assert_eq!(
            actual,
            config.base_working_dir().join(game.id()).join(PROTON)
        );

        Ok(())
    }

    #[test]
    fn proton_binary_should_return_correct_path() -> anyhow::Result<()> {
        // Arrange
        let game = Game::SkyrimSE;
        let config = setup(&game)?;

        let sut = Workspace::new(&game, &config)?;

        // Act
        let actual = sut.proton_binary();

        // Assert
        let expected = config
            .game_config_for(&game)
            .unwrap()
            .proton_dir
            .clone()
            .join("proton");

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn work_dir_should_return_correct_path() -> anyhow::Result<()> {
        // Arrange
        let game = Game::SkyrimSE;
        let config = setup(&game)?;

        let sut = Workspace::new(&game, &config)?;

        // Act
        let actual = sut.work_dir();

        // Assert
        assert_eq!(actual, config.base_working_dir().join(game.id()));

        Ok(())
    }

    #[test]
    fn prepare_file_system_should_create_dirs_if_non_existent() -> anyhow::Result<()> {
        // Arrange
        let game = Game::SkyrimSE;
        let config = setup(&game)?;

        let sut = Workspace::new(&game, &config)?;

        // Act
        let result = sut.prepare_file_system();

        // Assert
        assert!(result.is_ok());

        let actual: Vec<DirEntry> = WalkDir::new(&config.base_working_dir().parent().unwrap())
            .into_iter()
            .filter_map(Result::ok)
            .filter(|d| is_leaf(d).is_ok())
            .collect();

        assert_eq!(actual.len(), 7);

        let folder_names: Vec<String> = actual
            .iter()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();

        assert!(folder_names.contains(&PROTON.to_string()));
        assert!(folder_names.contains(&MERGED.to_string()));
        assert!(folder_names.contains(&WORK.to_string()));
        assert!(folder_names.contains(&ACTIVE.to_string()));
        assert!(folder_names.contains(&TRACKING.to_string()));
        assert!(folder_names.contains(&SINK.to_string()));
        assert!(folder_names.contains(&MODS.to_string()));

        Ok(())
    }

    #[test]
    fn prepare_file_system_should_not_overwrite_folders_if_they_already_exist() -> anyhow::Result<()>
    {
        // Arrange
        let game = Game::SkyrimSE;
        let config = setup(&game)?;

        let sut = Workspace::new(&game, &config)?;

        sut.prepare_file_system()?;

        let file_path = sut.work_dir().join("test.txt");
        fs::write(&file_path, "Sample value")?;

        // Act
        let result = sut.prepare_file_system();

        // Assert
        assert!(result.is_ok());

        assert!(file_path.exists());
        let content = fs::read_to_string(file_path)?;
        assert_eq!("Sample value", content);

        Ok(())
    }
}
