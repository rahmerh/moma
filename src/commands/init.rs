use anyhow::Context;
use clap::Args;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use owo_colors::OwoColorize;
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use crate::{
    config::{Config, GameConfig},
    games::GameProfile,
    utils::{fs::ExpandTilde, theme},
};

#[derive(Args)]
pub struct Init;

impl Init {
    pub fn run(&self, config: &mut Config) -> anyhow::Result<()> {
        println!("\n{}\n", "Moma initial setup".bold().underline().cyan());

        let theme = theme::default_theme();

        let steam_dir = config
            .get_steam_dir()
            .with_context(|| "Could not determine steam dir")?;
        let game = match determine_game(&config, &theme)? {
            Some(game) => game,
            None => return Ok(()),
        };

        let game_key = game.name().to_lowercase();
        let game_ref = &*game;
        let game_install_dir = determine_game_installation_dir(game_ref, &steam_dir, &theme)?;
        let proton_dir = determine_proton(&steam_dir, game_ref, &theme)?;

        let mut env_vars = HashMap::new();
        env_vars.insert("DISPLAY".to_string(), env::var("DISPLAY")?);
        env_vars.insert("XDG_RUNTIME_DIR".to_string(), env::var("XDG_RUNTIME_DIR")?);

        if let Ok(val) = env::var("WAYLAND_DISPLAY") {
            env_vars.insert("WAYLAND_DISPLAY".to_string(), val);
        }

        println!();
        println!("{}", "Configuration Summary".bold().cyan());
        println!("Game: \"{}\"", &game.name().bold());
        println!("Proton path: \"{}\"", &proton_dir.display().bold());
        println!("Path: \"{}\"", game_install_dir.display().bold());
        println!(
            "Moma's game working directory: \"{}\"",
            config.work_dir.join(&game_key).display()
        );
        println!();

        let confirmed = Confirm::with_theme(&theme)
            .with_prompt("Do you want to save this configuration?")
            .interact()?;

        if !confirmed {
            println!("{}", "Configuration not saved. Exiting.".yellow());
            return Ok(());
        }

        let game_config = GameConfig {
            path: game_install_dir,
            name: game_key.clone(),
            proton_dir,
            env: env_vars,
        };

        config.games.insert(game_key.clone(), game_config);
        config.save()?;

        println!(
            "{}",
            "\nConfiguration saved successfully\n"
                .bold()
                .underline()
                .cyan()
        );

        let saved_config = config
            .games
            .get(&game_key)
            .expect("Could not store game configuration.");
        game.setup_modding(config, saved_config)?;

        Ok(())
    }
}

fn determine_game(
    config: &Config,
    theme: &ColorfulTheme,
) -> anyhow::Result<Option<Box<dyn GameProfile>>> {
    let games = crate::games::get_supported_games();
    let labels: Vec<String> = games.iter().map(|g| g.name().to_string()).collect();

    println!("{}", "Available games".bold().cyan());
    let selection = Select::with_theme(theme)
        .items(&labels)
        .default(0)
        .interact()?;
    let game = games
        .into_iter()
        .nth(selection)
        .ok_or_else(|| anyhow::anyhow!("Unexpected selection index"))?;

    let game_name = game.name().to_lowercase();

    println!(
        "\n{} {}\n",
        "Setting up modding support for".bold().cyan(),
        game_name.bold().white()
    );

    if config.games.contains_key(&game_name) {
        let confirmation = Confirm::with_theme(theme)
            .with_prompt(format!(
                "Configuration for {} already exists. Reconfigure?",
                game_name.yellow()
            ))
            .interact()?;

        if !confirmation {
            println!("{}", "Exiting setup.".yellow());
            return Ok(None);
        }
    }

    Ok(Some(game))
}

fn determine_game_installation_dir(
    game: &dyn GameProfile,
    steam_dir: &Path,
    theme: &ColorfulTheme,
) -> anyhow::Result<PathBuf> {
    let default_game_path = game.default_game_path(steam_dir).expand();
    let path = Input::with_theme(theme)
        .with_prompt(format!(
            "Enter installation path for {}",
            game.name().cyan()
        ))
        .default(default_game_path.display().to_string())
        .validate_with(|input: &String| {
            let path = Path::new(input.trim());

            if !path.exists() {
                return Err("Path does not exist.");
            }

            if !path.join(game.game_executable()).is_file() {
                return Err("Game executable not found in this folder.");
            }

            Ok(())
        })
        .interact_text()?;

    Ok(PathBuf::from(path.trim()).expand())
}

fn determine_proton(
    steam_dir: &Path,
    game: &dyn GameProfile,
    theme: &ColorfulTheme,
) -> anyhow::Result<PathBuf> {
    let proton_dir = steam_dir.join("steamapps/common");

    let entries = std::fs::read_dir(&proton_dir)?
        .filter_map(Result::ok)
        .filter(|e| {
            let fname = e.file_name().to_string_lossy().to_lowercase();
            e.path().is_dir() && fname.contains("proton")
        })
        .collect::<Vec<_>>();

    if entries.is_empty() {
        return Err(anyhow::anyhow!(
            "No Proton versions found in {}",
            proton_dir.display()
        ));
    }

    let labels: Vec<String> = entries
        .iter()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    println!(
        "{} {}",
        "Choose Proton version for".bold().cyan(),
        game.name().white()
    );

    let selection = Select::with_theme(theme)
        .items(&labels)
        .default(0)
        .interact()?;

    Ok(entries[selection].path())
}
