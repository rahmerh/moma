use std::{
    fmt::Display,
    io::{Write, stdout},
    path::{Path, PathBuf},
};

use anyhow::Context;
use crossterm::{
    cursor::{MoveDown, MoveToColumn, MoveUp},
    execute,
    terminal::{Clear, ClearType},
};
use dialoguer::{Confirm, Input, MultiSelect, Password, Select};
use owo_colors::OwoColorize;

use crate::{ui::theme, utils::fs::ExpandTilde};

pub fn path(prompt_text: &str, default_value: Option<&str>) -> anyhow::Result<PathBuf> {
    let theme = theme::default_theme();

    let mut input = Input::with_theme(&theme)
        .with_prompt(prompt_text)
        .validate_with(|input: &String| {
            let path = Path::new(input.trim());
            if !path.exists() {
                Err("Path does not exist.")
            } else {
                Ok(())
            }
        });

    if let Some(default) = default_value {
        input = input.default(default.to_string());
    }

    let result = input.interact_text()?;
    Ok(PathBuf::from(result.trim()).expand())
}

pub fn confirm(prompt_text: &str) -> anyhow::Result<bool> {
    let theme = theme::default_theme();

    let confirmation = Confirm::with_theme(&theme)
        .with_prompt(prompt_text)
        .interact()?;

    Ok(confirmation)
}

pub fn password(prompt: &str) -> anyhow::Result<String> {
    let theme = crate::ui::theme::default_theme();

    let input = Password::with_theme(&theme)
        .with_prompt(prompt)
        .interact()
        .context("Failed to read password input")?;

    Ok(input)
}

pub fn select<T: Display + Clone>(prompt: &str, options: &[T]) -> anyhow::Result<T> {
    let theme = crate::ui::theme::default_theme();

    println!("{}", prompt.bold().cyan());

    let index = Select::with_theme(&theme)
        .items(options)
        .default(0)
        .interact()?;

    options
        .get(index)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Invalid selection index"))
}

pub fn select_path(prompt: &str, entries: Vec<PathBuf>) -> anyhow::Result<PathBuf> {
    if entries.is_empty() {
        anyhow::bail!("No entries available to select from.");
    }

    let theme = crate::ui::theme::default_theme();

    let labels: Vec<String> = entries
        .iter()
        .map(|path| {
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        })
        .collect();

    println!("{}", prompt.bold().cyan());

    let index = Select::with_theme(&theme)
        .items(&labels)
        .default(0)
        .interact()?;

    Ok(entries[index].clone())
}

pub fn select_multiple<T: Display + Clone>(prompt: &str, options: &[T]) -> anyhow::Result<Vec<T>> {
    let theme = theme::default_theme();

    let mut sorted_items: Vec<T> = options.to_vec();
    sorted_items.sort_by(|a, b| a.to_string().cmp(&b.to_string()));

    let labels: Vec<String> = sorted_items.iter().map(|item| item.to_string()).collect();

    let mut errored = false;
    loop {
        let selection = MultiSelect::with_theme(&theme)
            .with_prompt(prompt)
            .items(&labels)
            .interact()?;

        if selection.is_empty() {
            let clear_lines = match errored {
                true => 2,
                false => 1,
            };
            clear_previous_lines(clear_lines, 0)?;
            println!("{}", "Please select at least one option.".red());
            errored = true;
        } else {
            let selected_items = selection
                .into_iter()
                .map(|i| sorted_items[i].clone())
                .collect();
            if errored {
                clear_previous_lines(1, 1)?;
            }
            return Ok(selected_items);
        }
    }
}

pub fn input(prompt: &str, allow_empty: bool) -> anyhow::Result<String> {
    let theme = theme::default_theme();

    let value = Input::with_theme(&theme)
        .with_prompt(prompt)
        .allow_empty(allow_empty)
        .interact_text()?;

    Ok(value)
}

/// Clears lines from the cursor's position upwards. Will skip x amount before erasing lines if value > 0 given.
fn clear_previous_lines(amount: u16, skip: u16) -> std::io::Result<()> {
    let mut stdout = stdout();

    if skip > 0 {
        execute!(stdout, MoveUp(skip), MoveToColumn(0))?;
    }

    for _ in 0..amount {
        execute!(
            stdout,
            MoveUp(1),
            MoveToColumn(0),
            Clear(ClearType::CurrentLine),
        )?;
    }

    if skip > 0 {
        execute!(stdout, MoveDown(1 + skip), MoveToColumn(0))?;
    }

    stdout.flush()
}
