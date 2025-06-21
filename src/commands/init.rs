use anyhow::Context;
use clap::Args;
use dialoguer::{Confirm, MultiSelect, Select, theme::ColorfulTheme};
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};

use crate::{
    config::{Config, GameConfig},
    games::{self, GameProfile},
    sources::Source,
    ui::{prompt, reorder},
    utils::fs::ExpandTilde,
};

#[derive(Args)]
pub struct Init;

impl Init {
    pub fn run(&self, config: &mut Config) -> anyhow::Result<()> {
        println!("\n{}\n", "Moma initial setup".bold().underline().cyan());

        let steam_dir = config
            .get_steam_dir()
            .with_context(|| "Could not determine steam dir")?;
        let game = match determine_game(games::get_supported_games(), &config)? {
            Some(game) => game,
            None => return Ok(()),
        };

        let game_key = game.name().to_lowercase();
        let game_ref = &*game;
        let game_install_dir = determine_game_installation_dir(game_ref, &steam_dir)?;
        let proton_dir = determine_proton(&steam_dir, game_ref, &theme)?;
        let sources = determine_desired_sources(game_ref, &theme)
            .with_context(|| "Could not determine mod sources, please try again.")?;

        println!();
        println!("{}", "Configuration Summary".bold().cyan());
        println!("Game: \"{}\"", &game.name().bold());
        println!("Proton path: \"{}\"", &proton_dir.display().bold());
        println!("Path: \"{}\"", game_install_dir.display().bold());
        println!(
            "Mod platforms: \"{}\"",
            sources
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
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
            env: None,
            sources: sources,
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
    games: Vec<Box<dyn GameProfile>>,
    config: &Config,
) -> anyhow::Result<Option<Box<dyn GameProfile>>> {
    let game_names: Vec<String> = games.iter().map(|g| g.name().to_string()).collect();

    let selection = Select::new().items(&game_names).default(0).interact()?;

    let game = games
        .into_iter()
        .nth(selection)
        .ok_or_else(|| anyhow::anyhow!("Unexpected selection index"))?;

    let game_name = game.name().to_lowercase();

    if config.games.contains_key(&game_name) {
        return Ok(None); // Signal reconfigure prompt
    }

    Ok(Some(game))
}

fn determine_game_installation_dir(
    game: &dyn GameProfile,
    steam_dir: &Path,
) -> anyhow::Result<PathBuf> {
    let default_game_path = game.default_game_path(steam_dir).expand();

    let path = prompt::prompt_for_path(
        &format!("Enter installation path for {}", game.name().cyan()),
        Some(&default_game_path.display().to_string()),
    )?;

    Ok(path)
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

fn determine_desired_sources(
    game: &dyn GameProfile,
    theme: &ColorfulTheme,
) -> anyhow::Result<Vec<Source>> {
    let supported = game.supported_sources();

    if supported.len() == 1 {
        return Ok(supported);
    }

    let options: Vec<String> = supported.iter().map(|p| p.to_string()).collect();

    let selection;
    loop {
        let chosen = MultiSelect::with_theme(theme)
            .with_prompt(format!(
                "Which mod platforms do you want to use for {}? (Space to check, enter to submit)",
                game.name()
            ))
            .items(&options)
            .interact()?;

        if chosen.is_empty() {
            println!("{}", "Please select at least one platform.".red());
        } else {
            selection = chosen;
            break;
        }
    }

    let mut selected_platforms: Vec<Source> = selection
        .into_iter()
        .map(|i| supported[i].clone())
        .collect();

    if selected_platforms.len() == 1 {
        return Ok(selected_platforms);
    }

    println!(
        "{}",
        "\nPlease order the following mod sources in order of priority:"
            .bold()
            .cyan()
    );

    selected_platforms = reorder::reorder_items(selected_platforms)?;

    Ok(selected_platforms)
}
