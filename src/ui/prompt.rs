use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use anyhow::Context;
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

pub fn password_with_retry<F, T>(prompt: &str, validate: F) -> anyhow::Result<T>
where
    F: Fn(&str) -> anyhow::Result<T>,
{
    let theme = crate::ui::theme::default_theme();

    loop {
        let input = Password::with_theme(&theme)
            .with_prompt(prompt)
            .interact()
            .context("Failed to read password input")?;

        match validate(&input) {
            Ok(res) => return Ok(res),
            Err(_) => {
                eprintln!("{}{}", "Invalid input".red(), ", please try again.");
            }
        }
    }
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

pub fn select_multiple<'a>(prompt: &str, options: &'a [String]) -> anyhow::Result<Vec<usize>> {
    let theme = theme::default_theme();

    loop {
        let selection = MultiSelect::with_theme(&theme)
            .with_prompt(prompt)
            .items(options)
            .interact()?;

        if selection.is_empty() {
            println!("{}", "Please select at least one option.".red());
        } else {
            return Ok(selection);
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
