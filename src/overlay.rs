use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;

const OVERLAY: &str = ".overlay";
const MERGED: &str = "merged";
const WORK: &str = "work";
const ACTIVE: &str = "active";

pub fn merged_for(game_dir: &PathBuf) -> PathBuf {
    game_dir.join(OVERLAY).join(MERGED)
}

pub fn work_for(game_dir: &PathBuf) -> PathBuf {
    game_dir.join(OVERLAY).join(WORK)
}

pub fn active_for(game_dir: &PathBuf) -> PathBuf {
    game_dir.join(ACTIVE)
}

pub fn reset_dirs(game_dir: &PathBuf) -> anyhow::Result<()> {
    let merged = merged_for(&game_dir);
    let work = work_for(&game_dir);
    let active = active_for(&game_dir);
    let overlay_root = game_dir.join(OVERLAY);

    if overlay_root.exists() {
        for dir in [&merged, &work, &active] {
            match fs::remove_dir_all(dir) {
                Ok(_) => {}
                Err(err) if err.kind() == ErrorKind::NotFound => {}
                Err(err) => {
                    return Err(err).with_context(|| format!("Failed to remove {}", dir.display()));
                }
            }
        }
    }

    for dir in [&merged, &work, &active] {
        fs::create_dir_all(dir).with_context(|| format!("Failed to create {}", dir.display()))?;
    }

    Ok(())
}

pub fn mount_overlay_for(
    game_dir: &PathBuf,
    game_installation_dir: &PathBuf,
) -> anyhow::Result<()> {
    Command::new("mount")
        .args([
            "-t",
            "overlay",
            "overlay",
            "-o",
            &format!(
                "lowerdir={},upperdir={},workdir={}",
                game_installation_dir.display(),
                merged_for(&game_dir).display(),
                work_for(&game_dir).display()
            ),
            active_for(&game_dir).to_str().unwrap(),
        ])
        .status()
        .with_context(|| "Overlay mount failed")?
        .success()
        .then_some(())
        .ok_or_else(|| anyhow::anyhow!("Failed to mount overlay"))?;

    Ok(())
}
