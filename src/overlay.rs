use std::process::Command;

use anyhow::Context;

use crate::games::context::GameContext;

pub fn mount_overlay_for(context: &GameContext) -> anyhow::Result<()> {
    Command::new("mount")
        .args([
            "-t",
            "overlay",
            "overlay",
            "-o",
            &format!(
                "lowerdir={},upperdir={},workdir={}",
                context.game.path.display(),
                context.overlay_merged_dir().display(),
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
