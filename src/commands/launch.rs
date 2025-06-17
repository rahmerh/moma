use std::{
    fs::File,
    path::PathBuf,
    process::{self, Command, Stdio},
};

use anyhow::{Context, bail};
use clap::Args;
use dialoguer::Confirm;
use libc::CLONE_NEWNS;

use crate::{config::Config, overlay, utils::copy_dir};

#[derive(Args)]
pub struct Launch {
    /// Name of the game to launch
    game_name: String,
}

impl Launch {
    pub fn run(&self, config: &mut Config) -> anyhow::Result<()> {
        if unsafe { libc::getuid() != 0 } {
            bail!("This command must be run as root");
        }

        unsafe {
            if libc::unshare(CLONE_NEWNS) != 0 {
                return Err(std::io::Error::last_os_error())
                    .with_context(|| "Failed to unshare mount namespace");
            }
        }

        Command::new("mount")
            .args(["--make-rprivate", "/"])
            .status()
            .with_context(|| "Failed to set mount propagation to private")?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("Failed to make / private"))?;

        let game = config
            .games
            .get(&self.game_name.to_lowercase())
            .with_context(|| format!("No configuration found for game '{}'", &self.game_name))?;

        let game_dir = config.work_dir.join(&game.name);

        let overlay_merged_dir = overlay::merged_for(&game_dir);
        let overlay_active_dir = overlay::active_for(&game_dir);

        overlay::reset_dirs(&game_dir)?;

        overlay::mount_overlay_for(&game_dir, &game.path)
            .with_context(|| format!("Could not mount overlay folders for {}", self.game_name))?;

        let mut cmd =
            Command::new("/home/bas/.local/share/Steam/steamapps/common/Proton Hotfix/proton");
        cmd.arg("run")
            .arg("winecfg") // Replace with actual binary later
            .env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "/home/bas/.steam/steam")
            .env("STEAM_COMPAT_DATA_PATH", "/tmp/proton-test")
            .env("STEAM_RUNTIME", "1")
            .env("PROTON_NO_ESYNC", "1")
            .env("PROTON_NO_FSYNC", "1")
            .env("PROTON_USE_WINED3D", "1")
            .env("XDG_RUNTIME_DIR", "/run/user/1000")
            .env("WAYLAND_DISPLAY", "wayland-1")
            .env("DISPLAY", ":0");

        let output = cmd.output()?;
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

        println!("Unshared namespace PID: {}", process::id());
        println!("Mounted overlay to {}", overlay_active_dir.display());

        copy_dir(
            PathBuf::from("/home/bas/.moma/skyrim/mods/skse").as_ref(),
            &overlay_merged_dir,
            true,
            true,
        )?;

        Confirm::new().with_prompt("Continue?").interact()?;

        Ok(())
    }
}
