use clap::Args;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use owo_colors::OwoColorize;
use std::path::PathBuf;

use crate::{
    config::{Config, GameConfig},
    games::GameProfile,
    theme,
};

#[derive(Args)]
pub struct Init;

fn determine_game_path(theme: &ColorfulTheme, game: &dyn GameProfile) -> anyhow::Result<PathBuf> {
    let game_name = game.name();
    let mut input = game.default_path().to_string_lossy().to_string();

    loop {
        input = Input::with_theme(theme)
            .with_prompt(format!("Enter installation path for {}", game_name.cyan()))
            .with_initial_text(&input)
            .interact_text()?;

        let expanded = if input.starts_with('~') {
            match dirs_next::home_dir() {
                Some(home) => input.replacen("~", &home.to_string_lossy(), 1),
                None => input.clone(),
            }
        } else {
            input.clone()
        };

        let path = PathBuf::from(expanded);

        if !path.exists() {
            eprintln!(
                "{}\n  {}",
                "Invalid folder path!".red().bold(),
                path.to_string_lossy().red()
            );
            continue;
        }

        let skyrim_exe = path.join("SkyrimSE.exe");
        if !skyrim_exe.exists() {
            eprintln!(
                "{}\n  {}",
                "SkyrimSE.exe not found in this directory!".red().bold(),
                skyrim_exe.to_string_lossy().red()
            );
            continue;
        }

        return Ok(path);
    }
}

impl Init {
    pub fn run(&self, config: &mut Config) -> anyhow::Result<()> {
        println!(
            "\n{}\n",
            "================= MOMA INITIAL SETUP ================="
                .bold()
                .underline()
                .cyan()
        );

        let theme = theme::default_theme();
        let games = crate::games::get_supported_games();
        let labels: Vec<String> = games.iter().map(|g| g.name().to_string()).collect();

        println!("{}", "Available Games".bold().cyan());
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
                .interact()
                .unwrap();

            if !confirmation {
                println!("{}", "Exiting setup.".yellow());
                return Ok(());
            }
        }

        let path = determine_game_path(&theme, game.as_ref()).expect("test");

        println!();
        println!("{}", "Configuration Summary".bold().cyan());
        println!("Game: \"{}\"", game_name.bold());
        println!("Path: \"{}\"", path.to_string_lossy().bold());
        println!();

        let confirmed = Confirm::with_theme(&theme)
            .with_prompt("Do you want to save this configuration?")
            .interact()?;

        if !confirmed {
            println!("{}", "Configuration not saved. Exiting.".yellow());
            return Ok(());
        }

        let game_config = GameConfig { game_path: path };

        config.games.insert(game_name.to_string(), game_config);
        config.save()?;

        println!("{}", "Configuration saved successfully.".cyan());
        Ok(())
    }
}
