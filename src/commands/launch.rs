use std::{collections::HashMap, fs, process::Command};

use anyhow::{Context, bail};
use clap::Args;
use owo_colors::OwoColorize;

use crate::{
    config::Config,
    games::context::GameContext,
    ui::print,
    utils::{
        fs::copy_dir,
        os::{mount, permissions},
        state,
    },
};

#[derive(Args)]
pub struct Launch {
    /// Name of the game to launch
    pub game_name: Option<String>,

    /// Forces the launch of the game, ignoring sanity checks like an empty sink folder.
    #[arg(short, long, global = true)]
    pub force: bool,
}

impl Launch {
    pub fn run(&self, config: &mut Config) -> anyhow::Result<()> {
        if !permissions::is_process_root() {
            bail!("This command must be run as root. Try again with `sudo`.");
        }

        let game = match self.game_name {
            Some(ref game) => game.clone(),
            None => {
                let state = state::read_game_context()?
                    .ok_or_else(|| anyhow::anyhow!(
                        "No game provided and no context is set. Either provide a game to launch or set a context."
                    ))?;
                state
            }
        };

        let steam_dir = config.get_steam_dir()?;
        let context = GameContext::new(config, &game)?;

        if !context.validate_sink_is_empty()? {
            if self.force {
                println!(
                    "{} You are running {} with a non-empty sink folder.\n{}",
                    "Warning!".red().bold().underline(),
                    game.underline().bold(),
                    "To prevent unexpected overwrites, move everything into appropriate mod folders.".yellow()
                );
            } else {
                bail!(
                    "Your sink folder '{}' is not empty. Move this to an appropriate mod folder or add the force flag to continue.",
                    context.sink_dir().display()
                );
            }
        }

        println!("Launching {}...", game.bold());

        print::print_inline_status(&format!("{}", "Mounting game folders...".bold()))?;

        mount::unshare_current_namespace()?;
        mount::remount_current_namespace_as_private()?;

        context.prepare_file_system()?;

        mount::mount_overlay_for(&context)
            .with_context(|| format!("Could not mount overlay folders for {}", game))?;

        print::print_inline_status(&format!("{}", "Copying mods into mounted folder...".bold()))?;

        for entry in fs::read_dir(context.mods_dir())? {
            let entry = entry?;
            if !entry.metadata()?.is_dir() {
                continue;
            }
            copy_dir(&entry.path(), &context.overlay_merged_dir(), true, true)?;
        }

        permissions::drop_privileges()?;

        if !context.proton_binary().exists() {
            bail!(
                "Proton binary not found at '{}'",
                context.proton_binary().display()
            );
        }

        print::print_inline_status(&format!("Launching {}...", game.bold()))?;

        let mut proton_cmd = Command::new(context.proton_binary());
        proton_cmd.current_dir(&context.active_dir());

        let mut merged_env = std::env::vars().collect::<HashMap<_, _>>();
        merged_env.extend(context.game.get_env_vars());
        proton_cmd.envs(&merged_env);

        proton_cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", steam_dir);
        proton_cmd.env("STEAM_COMPAT_DATA_PATH", &context.proton_work_dir());
        proton_cmd.arg("run");
        proton_cmd.arg(
            &context
                .active_dir()
                .join(&context.profile.game_mod_executable()),
        );
        proton_cmd
            .spawn()
            .with_context(|| "Failed to start Proton process")?;

        print::print_inline_status(&format!("{}", "Have fun!".cyan().bold().underline()))?;

        Ok(())
    }
}
