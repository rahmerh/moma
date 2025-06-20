use std::{io, process::Command};

use anyhow::Context;
use libc::CLONE_NEWNS;

use crate::games::context::GameContext;

pub fn mount_overlay_for(context: &GameContext) -> anyhow::Result<()> {
    Command::new("mount")
        .args([
            "-t",
            "overlay",
            "overlay",
            "-o",
            &format!(
                "lowerdir={}:{},upperdir={},workdir={}",
                context.game.path.display(),
                context.overlay_merged_dir().display(),
                context.sink_dir().display(),
                context.overlay_work_dir().display(),
            ),
            context.active_dir().to_str().unwrap(),
        ])
        .status()
        .with_context(|| "Overlay mount failed")?
        .success()
        .then_some(())
        .ok_or_else(|| anyhow::anyhow!("Failed to mount overlay"))?;

    Ok(())
}

pub fn unshare_current_namespace() -> anyhow::Result<()> {
    let result = unsafe { libc::unshare(CLONE_NEWNS) };

    if result == -1 {
        let errno = io::Error::last_os_error();
        Err(anyhow::anyhow!("Failed to unshare namespace: {}", errno))
            .with_context(|| "unshare(CLONE_NEWNS | CLONE_NEWPID) failed")
    } else {
        Ok(())
    }
}

pub fn remount_current_namespace_as_private() -> anyhow::Result<()> {
    Command::new("mount")
        .args(["--make-rprivate", "/"])
        .status()
        .with_context(|| "Failed to set mount propagation to private")?
        .success()
        .then_some(())
        .ok_or_else(|| anyhow::anyhow!("mount --make-rprivate / failed"))
}
