use anyhow::Context;
use clap::Args;
use owo_colors::OwoColorize;
use std::path::PathBuf;
use strum::IntoEnumIterator;
use walkdir::WalkDir;

use crate::{
    app_context::AppContext,
    config::{Config, GameConfig},
    games::game::Game,
    ui::prompt::{self, Prompt, PromptExt},
};

#[derive(Args)]
pub struct Init;

impl Init {
    pub async fn run(&self, ctx: AppContext, config: &mut Config) -> anyhow::Result<()> {
        let prompter = ctx.prompt();

        println!("\n{}", "Moma initial setup".bold().underline().cyan());

        let all_games: Vec<Game> = Game::iter().collect();
        let game = prompter.select("Select game to initialize", &all_games)?;

        if config.game_config_for(&game).is_ok()
            && !prompter.confirm(&format!(
                "{} already setup, do you want to overwrite it?",
                &game.to_string()
            ))?
        {
            println!("{}", "Exiting setup.".yellow());
            return Ok(());
        }

        let game_install_dir = determine_game_installation_dir(prompter, &game, &config)?;
        let proton_dir = determine_proton(&game, &config)?;

        if !prompter.confirm("Save?")? {
            println!("{}", "Configuration not saved. Exiting.".yellow());
            return Ok(());
        }

        let game_config = GameConfig {
            path: game_install_dir,
            game: game.clone(),
            proton_dir,
            env: None,
        };

        config.add_game(game_config)?;

        let bay = ctx.bay(&game, &config.base_working_dir());

        bay.initialize()
            .with_context(|| "Could not initialize bay folders.")?;

        Ok(())
    }
}

fn determine_game_installation_dir(
    prompt: &dyn Prompt,
    game: &Game,
    config: &Config,
) -> anyhow::Result<PathBuf> {
    let default_game_path = game.default_game_path(config)?;

    let path = prompt.path(
        &format!("Enter installation path for {}", game.to_string().cyan()),
        Some(default_game_path.display().to_string()),
    )?;

    Ok(path)
}

fn determine_proton(prompt: &dyn Prompt, game: &Game, config: &Config) -> anyhow::Result<PathBuf> {
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

    let selected_proton = prompt
        .select(&format!("Choose Proton version for {}", game), entries)
        .with_context(|| format!("No proton could be found in {}", common_dir.display()))?;

    Ok(selected_proton)
}
