use std::{fs, path::PathBuf};

use anyhow::{Context, bail};
use owo_colors::OwoColorize;

use crate::games::Game;

const STATE_FILE: &str = "/tmp/moma_state";

pub fn current_context() -> anyhow::Result<Option<Game>> {
    let path = PathBuf::from(STATE_FILE);
    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(&path).context("Failed to read state file")?;
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

pub fn set_context(game: Game) -> anyhow::Result<()> {
    fs::write(STATE_FILE, game.id())?;
    println!(
        "{} {}",
        "Game context set to".cyan().bold(),
        game.to_string().bold()
    );
    Ok(())
}

pub fn clear_context() -> anyhow::Result<()> {
    let path = PathBuf::from(STATE_FILE);

    if path.exists() {
        fs::remove_file(path).context("Could not remove state file")?;
        println!("{}", "Game context cleared.".cyan().bold());
    } else {
        println!("{}", "No game context was set.".yellow());
    }

    Ok(())
}
