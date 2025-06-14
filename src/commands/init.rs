use clap::Args;
use dialoguer::{Input, Select, theme::ColorfulTheme};
use owo_colors::OwoColorize;
use std::path::PathBuf;

use crate::config::{Config, GameConfig};

#[derive(Args)]
pub struct Init;

impl Init {
    pub fn run(&self, config: &mut Config) -> anyhow::Result<()> {
        println!("{}", "Moma Initial Setup".bold().underline().cyan());

        let theme = ColorfulTheme::default();

        let games = crate::games::get_supported_games();
        let labels: Vec<String> = games.iter().map(|g| g.name().to_string()).collect();

        let selection = Select::with_theme(&theme)
            .with_prompt("Which game do you want to set up?")
            .items(&labels)
            .default(0)
            .interact()?;
        let game = &games[selection];

        let game_name = game.name();
        let default_path = game.default_path();
        let path: String = Input::with_theme(&theme)
            .with_prompt(format!("Enter path to {} installation", &game_name))
            .with_initial_text(default_path.to_string_lossy())
            .interact_text()?;

        let expanded_path = if path.starts_with('~') {
            path.replacen("~", &dirs_next::home_dir().unwrap().to_string_lossy(), 1)
        } else {
            path
        };

        let game_config = GameConfig {
            game_path: PathBuf::from(expanded_path),
        };

        config.games.insert(game_name.to_string(), game_config);
        config.save()?;

        Ok(())
    }
}
