use std::{fs, path::PathBuf, process::Command};

use anyhow::{Context, bail};
use clap::Args;

use crate::{config::Config, games::get_game_profile_by_name, os, overlay, utils::copy_dir};

#[derive(Args)]
pub struct Launch {
    /// Name of the game to launch
    pub game_name: String,
}

impl Launch {
    pub fn run(&self, config: &mut Config) -> anyhow::Result<()> {
        if !os::is_process_root() {
            bail!("Must be run as root.");
        }

        println!("{}", Config::default_path().display());

        let game = config
            .games
            .get(&self.game_name.to_lowercase())
            .with_context(|| format!("No configuration found for game '{}'", &self.game_name))?;

        let game_profile = get_game_profile_by_name(&game.name)
            .ok_or_else(|| anyhow::anyhow!("Unknown game profile: {}", game.name))?;

        let game_dir = config.work_dir.join(&game.name);

        let overlay_merged_dir = overlay::merged_for(&game_dir);
        let overlay_active_dir = overlay::active_for(&game_dir);

        os::unshare_current_namespace()?;
        os::remount_current_namespace_as_private()?;
        overlay::reset_dirs(&game_dir)?;
        overlay::mount_overlay_for(&game_dir, &game.path)
            .with_context(|| format!("Could not mount overlay folders for {}", self.game_name))?;

        copy_dir(
            PathBuf::from("/home/bas/.moma/skyrim/mods/skse").as_ref(),
            &overlay_merged_dir,
            true,
            true,
        )?;

        os::drop_privileges()?;

        fs::create_dir_all("/home/bas/.moma/skyrim/proton")?;

        let mut proton_cmd = Command::new(game.proton_dir.join("proton"));
        proton_cmd.current_dir(overlay_active_dir);
        proton_cmd.envs(std::env::vars());
        proton_cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "/home/bas/.steam/steam");
        proton_cmd.env("STEAM_COMPAT_DATA_PATH", game_dir.join("proton"));
        proton_cmd.arg("run");
        proton_cmd.arg(overlay::active_for(&game_dir).join(game_profile.game_mod_executable()));
        proton_cmd.spawn()?;

        Ok(())
    }
}
