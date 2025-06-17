use std::{fs, process::Command};

use anyhow::{Context, bail};
use clap::Args;
use owo_colors::OwoColorize;

use crate::{
    config::Config,
    games::context::GameContext,
    utils::{fs::copy_dir, os, overlay},
};

#[derive(Args)]
pub struct Launch {
    /// Name of the game to launch
    pub game_name: String,

    /// Forces the launch of the game, ignoring sanity checks like an empty sink folder.
    #[arg(short, long, global = true)]
    pub force: bool,
}

impl Launch {
    pub fn run(&self, config: &mut Config) -> anyhow::Result<()> {
        if !os::is_process_root() {
            bail!("Must be run as root.");
        }

        let steam_dir = config.get_steam_dir()?;
        let context = GameContext::new(config, &self.game_name)?;

        if !context.validate_sink_is_empty()? {
            if self.force {
                println!(
                    "{} {}You are running {} with a non-empty sink folder.",
                    "Warning!".red().bold().underline(),
                    "".white(),
                    self.game_name.underline().bold()
                );
                println!("{}",
                    "To prevent unexpected overwrites, make sure to move everything into appropriate mod folders.".yellow()
                )
            } else {
                bail!(
                    "Your sink folder '{}' is not empty. Move this to an appropriate mod folder or add the force flag to continue.",
                    context.sink_dir().display()
                );
            }
        }

        os::unshare_current_namespace()?;
        os::remount_current_namespace_as_private()?;

        context.prepare_file_system()?;

        overlay::mount_overlay_for(&context)
            .with_context(|| format!("Could not mount overlay folders for {}", self.game_name))?;

        for entry in fs::read_dir(context.mods_dir())? {
            let entry = entry?;
            if !entry.metadata()?.is_dir() {
                continue;
            }
            copy_dir(&entry.path(), &context.overlay_merged_dir(), true, true)?;
        }

        os::drop_privileges()?;

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
