use std::{fs, process::Command};

use anyhow::{Context, bail};
use clap::Args;
use owo_colors::OwoColorize;

use crate::{
    cli::Cli,
    config::Config,
    games::{Game, workspace::Workspace},
    mods::env_store::EnvStore,
    ui::print,
    usage_for,
    utils::{
        fs::copy_dir,
        os::{
            mount::{Mountable, OverlayMounter},
            permissions,
            system_interface::System,
        },
        state,
    },
};

#[derive(Args)]
pub struct Launch {
    /// Name of the game to launch
    pub game: Option<Game>,

    /// Forces the launch of the game, ignoring sanity checks like an empty sink folder.
    #[arg(short, long, global = true)]
    pub force: bool,
}

impl Launch {
    pub fn run(&self, config: &Config) -> anyhow::Result<()> {
        if !permissions::is_process_root() {
            bail!("This command must be run as root. Try again with `sudo`.");
        }

        let game = match self.game {
            Some(ref game) => game.clone(),
            None => state::current_context(&config.state_file)?.ok_or_else(|| {
                anyhow::anyhow!(
                    "No game specified and no context is set. (Try: '{}')",
                    usage_for!(Cli::CONTEXT)
                )
            })?,
        };

        let context = Workspace::new(&game, &config)?;
        let env_store = EnvStore::new(context.clone());
        let system = System;
        let mounter = OverlayMounter::new(&context, &system);

        println!("Launching {}...", game.bold());

        print::print_inline_status(&format!("{}", "Mounting game folders...".bold()))?;

        mounter.unshare_as_private_namespace()?;
        mounter
            .mount_overlay()
            .with_context(|| format!("Could not mount overlay folders for {}", game))?;

        print::print_inline_status(&format!("{}", "Copying mods into mounted folder...".bold()))?;

        for entry in fs::read_dir(context.mods_dir())? {
            let entry = entry?;
            if !entry.metadata()?.is_dir() {
                println!(
                    "{} Not a directory, skipping: {}",
                    "Warning:".yellow(),
                    entry.path().display()
                );
                continue;
            }
            copy_dir(&entry.path(), &context.overlay_merged_dir(), true)?;
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

        let game_config = config.games.get(game.id()).ok_or_else(|| {
            anyhow::anyhow!("No configuration found for game {}", game.to_string())
        })?;

        let mut env_vars = env_store.read_env_vars()?;
        env_vars.extend(game_config.get_env_vars());
        proton_cmd.envs(env_vars);

        proton_cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", &config.steam_dir);
        proton_cmd.env("STEAM_COMPAT_DATA_PATH", &context.proton_work_dir());
        proton_cmd.arg("run");
        proton_cmd.arg(&context.active_dir().join(&game.game_mod_executable()));
        proton_cmd
            .spawn()
            .with_context(|| "Failed to start Proton process")?;

        print::print_inline_status(&format!("{}", "Have fun!".cyan().bold().underline()))?;

        Ok(())
    }
}
