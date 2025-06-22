use std::{fs, path::PathBuf};

use anyhow::{Context, bail};
use owo_colors::OwoColorize;

use crate::config::Config;

const STATE_FILE: &str = "/tmp/moma_state";

pub fn read_game_context() -> anyhow::Result<Option<String>> {
    let path = PathBuf::from(STATE_FILE);

    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("Failed to read state file: {}", e))?;

    let trimmed = contents.trim();

    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

pub fn clear_game_context() -> anyhow::Result<()> {
    let file = PathBuf::from(STATE_FILE);

    if file.exists() {
        fs::remove_file(file).with_context(|| "Could not remove moma's state file.")?;
        println!("{}", "Game context cleared.".cyan().bold());
    } else {
        println!("{}", "No game context was set.".yellow());
    }

    return Ok(());
}

pub fn set_game_context(game: &str) -> anyhow::Result<()> {
    let config = Config::load_or_default()?;

    let game = game.trim().to_lowercase();

    if !config.games.contains_key(&game) {
        bail!("No game configuration present for {}", game.bold().cyan());
    }

    fs::write(STATE_FILE, &game)?;
    println!("{} {}", "Game context set to".cyan().bold(), game.bold());

    Ok(())
}
