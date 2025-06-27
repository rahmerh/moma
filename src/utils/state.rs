use std::{fs, path::PathBuf};

use anyhow::{Context, bail};
use owo_colors::OwoColorize;

use crate::games::Game;

pub const STATE_FILE_PATH: &str = "/tmp/moma_state";

pub fn current_context(state_file: &PathBuf) -> anyhow::Result<Option<Game>> {
    if !state_file.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(state_file).context("Failed to read state file")?;
    let trimmed = contents.trim();

    if trimmed.is_empty() {
        Ok(None)
    } else {
        match Game::from_id(trimmed) {
            Some(game) => Ok(Some(game)),
            None => bail!("Invalid game context in state file: '{}'", trimmed),
        }
    }
}

pub fn set_context(state_file: &PathBuf, game: Game) -> anyhow::Result<()> {
    fs::write(state_file, game.id())?;
    println!(
        "{} {}",
        "Game context set to".cyan().bold(),
        game.to_string().bold()
    );
    Ok(())
}

pub fn clear_context(state_file: &PathBuf) -> anyhow::Result<()> {
    if state_file.exists() {
        fs::remove_file(state_file).context("Could not remove state file")?;
        println!("{}", "Game context cleared.".cyan().bold());
    } else {
        println!("{}", "No game context was set.".yellow());
    }

    Ok(())
}
