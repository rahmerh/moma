use owo_colors::OwoColorize;

use crate::{config::Config, utils::string::StringUtils};

pub struct Add;

impl Add {
    pub fn run(game_name: &str, mod_id: &str, config: &mut Config) -> anyhow::Result<()> {
        let game_config = match config.games.get(game_name) {
            Some(cfg) => cfg,
            None => {
                println!(
                    "Configuration for {} is missing, run 'moma init' to setup your game's configuration.",
                    game_name.red()
                );
                return Ok(());
            }
        };

        for source in &game_config.sources {
            if !source.is_setup() {
                println!(
                    "{} {} expects the source {}, but it’s not configured yet.\n\t{} 'moma connect {}' {}",
                    "Warning!".bold().underline().red(),
                    game_name.capitalize().bold(),
                    source.to_string().bold().cyan(),
                    "› Run".dimmed(),
                    source.to_string().to_lowercase().bold(),
                    "to finish the setup.".dimmed()
                );
                return Ok(());
            }

            let files = source.get_mod_files(game_name, mod_id)?;

            println!(
                "Main: {} - {}",
                files.main.len(),
                files
                    .main
                    .iter()
                    .map(|f| f.id.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        println!("Adding mod...");

        Ok(())
    }
}
