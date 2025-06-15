use clap::Args;
use dialoguer::{Confirm, Input, Select};
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};

use crate::{
    config::{Config, GameConfig},
    theme,
    utils::ExpandTilde,
};

#[derive(Args)]
pub struct Init;

impl Init {
    pub fn run(&self, config: &mut Config) -> anyhow::Result<()> {
        println!("\n{}\n", "Moma initial setup".bold().underline().cyan());

        let theme = theme::default_theme();
        let games = crate::games::get_supported_games();
        let labels: Vec<String> = games.iter().map(|g| g.name().to_string()).collect();

        println!("{}", "Available games".bold().cyan());
        let selection = Select::with_theme(&theme)
            .items(&labels)
            .default(0)
            .interact()?;
        let game = &games[selection];
        let game_name = game.name();

        println!(
            "\n{} {}\n",
            "Setting up modding support for".bold().cyan(),
            game_name.bold().white()
        );

        if config.games.contains_key(game_name) {
            let confirmation = Confirm::with_theme(&theme)
                .with_prompt(format!(
                    "Configuration for {} already exists. Reconfigure?",
                    game_name.yellow()
                ))
                .interact()?;

            if !confirmation {
                println!("{}", "Exiting setup.".yellow());
                return Ok(());
            }
        }

        let default_game_path = game.default_path().expand();
        let path = Input::with_theme(&theme)
            .with_prompt(format!("Enter installation path for {}", game_name.cyan()))
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

        let expanded_path = PathBuf::from(path.trim()).expand();
        let game_working_dir = config.work_dir.join(game_name.to_lowercase());

        println!();
        println!("{}", "Configuration Summary".bold().cyan());
        println!("Game: \"{}\"", game_name.bold());
        println!("Path: \"{}\"", expanded_path.display().bold());
        println!(
            "Moma's game working directory: \"{}\"",
            game_working_dir.display().bold()
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
            path: expanded_path,
        };

        config.games.insert(game_name.to_string(), game_config);
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
            .get(game_name)
            .expect("Could not store game configuration.");
        game.setup_modding(config, saved_config)?;

        Ok(())
    }
}
