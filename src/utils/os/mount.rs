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
