use anyhow::Context;
use clap::Args;
use owo_colors::OwoColorize;
use std::path::PathBuf;
use strum::IntoEnumIterator;
use walkdir::WalkDir;

use crate::{
    config::{Config, GameConfig},
    games::{Game, workspace::Workspace},
    mods::env_store::EnvStore,
    sources::Source,
    ui::{prompt, reorder},
};

#[derive(Args)]
pub struct Init;

impl Init {
    pub async fn run(&self, config: &mut Config) -> anyhow::Result<()> {
        println!("\n{}\n", "Moma initial setup".bold().underline().cyan());

        let all_games: Vec<Game> = Game::iter().collect();
        let game = prompt::select("Select game to initialize", &all_games)?;

        if config.game_config_for(&game).is_ok()
            && !prompt::confirm(&format!(
                "{} already setup, do you want to overwrite it?",
                &game.to_string()
            ))?
        {
            println!("{}", "Exiting setup.".yellow());
            return Ok(());
        }

        let game_install_dir = determine_game_installation_dir(&game, &config)?;
        let proton_dir = determine_proton(&game, &config)?;
        let sources = determine_desired_sources(&game)
            .with_context(|| "Could not determine mod sources, please try again.")?;

        if !prompt::confirm("Save?")? {
            println!("{}", "Configuration not saved. Exiting.".yellow());
            return Ok(());
        }

        let game_config = GameConfig {
            path: game_install_dir,
            game: game.clone(),
            proton_dir,
            env: None,
            sources: sources,
        };

        config.add_game_config(game_config)?;

        let workspace = Workspace::new(&game, config)?;
        workspace.prepare_file_system()?;

        let env_store = EnvStore::new(workspace);
        env_store
            .store_env_vars(std::env::vars().collect())
            .with_context(|| "Could not store user's env vars.")?;

        println!(
            "{}",
            "\nConfiguration saved successfully\n"
                .bold()
                .underline()
                .cyan()
        );
        game.setup(config).await?;

        Ok(())
    }
}

fn determine_game_installation_dir(game: &Game, config: &Config) -> anyhow::Result<PathBuf> {
    let default_game_path = game.default_game_path(config)?;

    let path = prompt::path(
        &format!("Enter installation path for {}", game.to_string().cyan()),
        Some(&default_game_path.display().to_string()),
    )?;

    Ok(path)
}

fn determine_proton(game: &Game, config: &Config) -> anyhow::Result<PathBuf> {
    let common_dir = config.steam_dir().join("steamapps/common");

    let entries = WalkDir::new(&common_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().is_dir()
                && entry
                    .file_name()
                    .to_string_lossy()
                    .to_lowercase()
                    .contains("proton")
        })
        .map(|entry| entry.into_path())
        .collect::<Vec<_>>();

    let selected_proton =
        prompt::select_path(&format!("Choose Proton version for {}", game), entries)
            .with_context(|| format!("No proton could be found in {}", common_dir.display()))?;

    Ok(selected_proton)
}

fn determine_desired_sources(game: &Game) -> anyhow::Result<Vec<Source>> {
    let supported = game.default_mod_sources();

    if supported.len() == 1 {
        return Ok(supported);
    }

    let mut selected_platforms = prompt::select_multiple(
        &format!(
            "Which mod sources do you want to use for {}?",
            game.to_string()
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
