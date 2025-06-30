use crate::{games::workspace::Workspace, utils::os::system_interface::SystemInterface};
use std::process::Command;

pub trait Mountable {
    fn mount_overlay(&self) -> anyhow::Result<()>;
    fn unshare_as_private_namespace(&self) -> anyhow::Result<()>;
}

pub struct OverlayMounter<'a> {
    workspace: &'a Workspace,
    system: &'a dyn SystemInterface,
}

impl<'a> OverlayMounter<'a> {
    pub fn new(workspace: &'a Workspace, system: &'a dyn SystemInterface) -> Self {
        Self { workspace, system }
    }
}

impl<'a> Mountable for OverlayMounter<'a> {
    fn mount_overlay(&self) -> anyhow::Result<()> {
        let mut cmd = Command::new("mount");
        cmd.args([
            "-t",
            "overlay",
            "overlay",
            "-o",
            &format!(
                "lowerdir={}:{},upperdir={},workdir={}",
                self.workspace.overlay_merged_dir().display(),
                self.workspace.game_dir().display(),
                self.workspace.sink_dir().display(),
                self.workspace.overlay_work_dir().display(),
            ),
            self.workspace.active_dir().to_str().unwrap(),
        ]);

        self.system.run_command(&mut cmd)
    }

    fn unshare_as_private_namespace(&self) -> anyhow::Result<()> {
        self.system.unshare_namespace()?;

        let mut cmd = Command::new("mount");
        cmd.args(["--make-rprivate", "/"]);
        self.system.run_command(&mut cmd)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use tempfile::TempDir;

    use crate::{
        config::{Config, GameConfig},
        games::Game,
        utils::os::system_interface::MockSystemInterface,
    };

    use super::*;

    fn setup_workspace(game: &Game) -> (Workspace, MockSystemInterface) {
        let temp_dir = TempDir::new().unwrap();
        let work_dir = temp_dir.path().join("working");
        let steam_dir = temp_dir.path().join("steam");
        let state_file = temp_dir.path().join("test-state");

        let mut games = HashMap::new();
        games.insert(
            game.id().to_string(),
            GameConfig {
                name: game.id().to_string(),
                path: PathBuf::from("/fake/game"),
                proton_dir: PathBuf::from("/fake/proton"),
                env: None,
                sources: vec![],
            },
        );

        let config = Config {
            games,
            work_dir,
            steam_dir: Some(steam_dir),
            nexus_api_key: Some("abc".to_string()),
            state_file,
        };

        let workspace = Workspace::new(game, &config).unwrap();
        workspace.prepare_file_system().unwrap();

        let mock = MockSystemInterface::new();

        (workspace, mock)
    }

    #[test]
    fn mount_overlay_should_invoke_mount_command_with_correct_args() {
        // Arrange
        let game = Game::SkyrimSE;
        let (workspace, mut mock) = setup_workspace(&game);

        mock.expect_run_command()
            .times(1)
            .withf(|cmd: &Command| {
                let args = cmd
                    .get_args()
                    .map(|s| s.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join(" ");

                args.contains("lowerdir=")
                    && args.contains("upperdir=")
                    && args.contains("workdir=")
                    && args.ends_with("active")
            })
            .returning(|_| Ok(()));

        let mounter = OverlayMounter::new(&workspace, &mock);

        // Act
        let result = mounter.mount_overlay();

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn unshare_as_private_namespace_should_delegate_to_system_interface() {
        // Arrange
        let game = Game::SkyrimSE;
        let (workspace, mut mock) = setup_workspace(&game);

        mock.expect_unshare_namespace()
            .times(1)
            .returning(|| Ok(()));

        mock.expect_run_command()
            .times(1)
            .withf(|cmd: &Command| {
                let args = cmd.get_args().collect::<Vec<_>>();
                args == ["--make-rprivate", "/"]
            })
            .returning(|_| Ok(()));

        let mounter = OverlayMounter::new(&workspace, &mock);

        // Act
        let result = mounter.unshare_as_private_namespace();

        // Assert
        assert!(result.is_ok());
    }
}
