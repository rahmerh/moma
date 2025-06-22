use anyhow::Context;
use clap::Args;
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
        let game = determine_game(games::get_supported_games())?;

        let game_name_lower = game.name().to_lowercase();
        let game_ref = &*game;

        if config.games.contains_key(&game_name_lower) {
            if !prompt::confirm(&format!(
                "{} already setup, do you want to overwrite it?",
                &game_name_lower
            ))? {
                println!("{}", "Exiting setup.".yellow());
                return Ok(());
            }
        }

        let game_install_dir = determine_game_installation_dir(game_ref, &steam_dir)?;
        let proton_dir = determine_proton(&steam_dir, game_ref)?;
        let sources = determine_desired_sources(game_ref)
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
            config.work_dir.join(&game_name_lower).display()
        );
        println!();

        if !prompt::confirm("Do you want to save this configuration?")? {
            println!("{}", "Configuration not saved. Exiting.".yellow());
            return Ok(());
        }

        let game_config = GameConfig {
            path: game_install_dir,
            name: game_name_lower.clone(),
            proton_dir,
            env: None,
            sources: sources,
        };

        config.games.insert(game_name_lower.clone(), game_config);
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
            .get(&game_name_lower)
            .expect("Could not store game configuration.");
        game.setup_modding(config, saved_config)?;

        Ok(())
    }
}

fn determine_game(games: Vec<Box<dyn GameProfile>>) -> anyhow::Result<Box<dyn GameProfile>> {
    let labeled_games: Vec<(String, Box<dyn GameProfile>)> = games
        .into_iter()
        .map(|g| (g.name().to_string(), g))
        .collect();

    let labels: Vec<String> = labeled_games.iter().map(|(name, _)| name.clone()).collect();

    let selected_name = prompt::select("Available games", &labels)?;

    let (_, game) = labeled_games
        .into_iter()
        .find(|(name, _)| name == &selected_name)
        .ok_or_else(|| anyhow::anyhow!("Game not found"))?;

    Ok(game)
}

fn determine_game_installation_dir(
    game: &dyn GameProfile,
    steam_dir: &Path,
) -> anyhow::Result<PathBuf> {
    let default_game_path = game.default_game_path(steam_dir).expand();

    let path = prompt::path(
        &format!("Enter installation path for {}", game.name().cyan()),
        Some(&default_game_path.display().to_string()),
    )?;

    Ok(path)
}

fn determine_proton(steam_dir: &Path, game: &dyn GameProfile) -> anyhow::Result<PathBuf> {
    let common_dir = steam_dir.join("steamapps/common");

    let entries = std::fs::read_dir(&common_dir)?
        .filter_map(Result::ok)
        .filter(|e| {
            let fname = e.file_name().to_string_lossy().to_lowercase();
            e.path().is_dir() && fname.contains("proton")
        })
        .map(|e| e.path())
        .collect::<Vec<_>>();

    let selected_proton = prompt::select_path(
        &format!("Choose Proton version for {}", game.name()),
        entries,
    )
    .with_context(|| format!("No proton could be found in {}", common_dir.display()))?;

    Ok(selected_proton)
}

fn determine_desired_sources(game: &dyn GameProfile) -> anyhow::Result<Vec<Source>> {
    let supported = game.supported_sources();

    if supported.len() == 1 {
        return Ok(supported);
    }

    let mut selected_platforms = prompt::select_multiple(
        &format!(
            "Which mod platforms do you want to use for {}?",
            game.name()
        ),
        &supported,
    )?;

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
