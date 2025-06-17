use std::{fs, path::PathBuf, process::Command};

use anyhow::{Context, bail};
use clap::Args;

use crate::{config::Config, games::context::GameContext, os, overlay, utils::copy_dir};

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

        let steam_dir = config.get_steam_dir()?;
        let context = GameContext::new(config, &self.game_name)?;

        os::unshare_current_namespace()?;
        os::remount_current_namespace_as_private()?;

        context.reset_overlay_dirs()?;

        overlay::mount_overlay_for(&context)
            .with_context(|| format!("Could not mount overlay folders for {}", self.game_name))?;

        // TODO: Copy all mods automatically
        copy_dir(
            PathBuf::from("/home/bas/.moma/skyrim/mods/skse").as_ref(),
            &context.overlay_merged_dir(),
            true,
            true,
        )?;

        os::drop_privileges()?;

        fs::create_dir_all(context.proton_work_dir())?;

        let mut proton_cmd = Command::new(context.proton_binary());
        proton_cmd.current_dir(&context.active_dir());
        proton_cmd.envs(std::env::vars());
        proton_cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", steam_dir);
        proton_cmd.env("STEAM_COMPAT_DATA_PATH", &context.proton_work_dir());
        proton_cmd.arg("run");
        proton_cmd.arg(
            &context
                .active_dir()
                .join(&context.profile.game_mod_executable()),
        );
        proton_cmd.spawn()?;

        Ok(())
    }
}
