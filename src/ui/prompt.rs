use anyhow::Context;
use crossterm::{
    cursor::{MoveDown, MoveToColumn, MoveUp},
    queue,
    terminal::{Clear, ClearType},
};
use dialoguer::{Confirm, Input, MultiSelect, Password, Select};
use owo_colors::OwoColorize;
use std::{
    fmt::Display,
    io::{Write, stdout},
    path::{Path, PathBuf},
};

use crate::{ui::theme, utils::fs::ExpandTilde};

#[cfg_attr(test, mockall::automock)]
pub trait Prompt {
    fn confirm(&self, prompt_text: &str) -> anyhow::Result<bool>;
    fn select_string(&self, prompt: &str, options: &[String]) -> anyhow::Result<String>;
    fn select_multiple_strings(
        &self,
        prompt: &str,
        options: &[String],
    ) -> anyhow::Result<Vec<String>>;
    fn path(&self, prompt_text: &str, default_value: Option<String>) -> anyhow::Result<PathBuf>;
    fn password(&self, prompt: &str) -> anyhow::Result<String>;
}

pub struct Prompter;

impl Prompt for Prompter {
    fn confirm(&self, prompt_text: &str) -> anyhow::Result<bool> {
        let theme = theme::default_theme();

        let confirmation = Confirm::with_theme(&theme)
            .with_prompt(prompt_text)
            .interact()?;

        Ok(confirmation)
    }

    fn select_string(&self, prompt: &str, options: &[String]) -> anyhow::Result<String> {
        let theme = crate::ui::theme::default_theme();

        let index = Select::with_theme(&theme)
            .with_prompt(prompt)
            .items(options)
            .default(0)
            .interact()?;

        options
            .get(index)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Invalid selection index"))
    }

    fn select_multiple_strings(
        &self,
        prompt: &str,
        options: &[String],
    ) -> anyhow::Result<Vec<String>> {
        let theme = theme::default_theme();

        let mut errored = false;
        loop {
            let selection = MultiSelect::with_theme(&theme)
                .with_prompt(prompt)
                .items(options)
                .interact()?;

            if selection.is_empty() {
                let clear_lines = if errored { 2 } else { 1 };
                clear_previous_lines(clear_lines, 0)?;
                println!("{}", "Please select at least one option.".red());
                errored = true;
            } else {
                if errored {
                    clear_previous_lines(1, 1)?;
                }
                return Ok(selection.into_iter().map(|i| options[i].clone()).collect());
            }
        }
    }

    fn path(&self, prompt_text: &str, default_value: Option<String>) -> anyhow::Result<PathBuf> {
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
            input = input.default(default);
        }

        let result = input.interact_text()?;
        Ok(PathBuf::from(result.trim()).expand())
    }

    fn password(&self, prompt: &str) -> anyhow::Result<String> {
        let theme = crate::ui::theme::default_theme();

        let input = Password::with_theme(&theme)
            .with_prompt(prompt)
            .interact()
            .context("Failed to read password input")?;

        Ok(input)
    }
}

pub trait PromptExt: Prompt {
    fn select<T: Clone>(&self, prompt: &str, options: &[T]) -> anyhow::Result<T>
    where
        for<'a> &'a T: Display,
    {
        let labels: Vec<String> = options.iter().map(|o| o.to_string()).collect();
        let selected = self.select_string(prompt, &labels)?;
        let idx = labels
            .iter()
            .position(|s| s == &selected)
            .ok_or_else(|| anyhow::anyhow!("Invalid selection"))?;
        Ok(options[idx].clone())
    }

    fn select_multiple<T: Display + Clone>(
        &self,
        prompt: &str,
        options: &[T],
    ) -> anyhow::Result<Vec<T>> {
        let mut sorted_items: Vec<T> = options.to_vec();
        sorted_items.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
        let labels: Vec<String> = sorted_items.iter().map(|item| item.to_string()).collect();

        let selected_labels = self.select_multiple_strings(prompt, &labels)?;
        let selected = selected_labels
            .into_iter()
            .filter_map(|label| {
                sorted_items
                    .iter()
                    .find(|item| item.to_string() == label)
                    .cloned()
            })
            .collect();

        Ok(selected)
    }
}

impl<T: Prompt + ?Sized> PromptExt for T {}

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

/// Clears lines from the cursor's position upwards. Will skip x amount before erasing lines if skip > 0 given.
fn clear_previous_lines(amount: u16, skip: u16) -> std::io::Result<()> {
    let mut stdout = stdout();

    if skip > 0 {
        queue!(stdout, MoveUp(skip), MoveToColumn(0))?;
    }

    for _ in 0..amount {
        queue!(
            stdout,
            MoveUp(1),
            MoveToColumn(0),
            Clear(ClearType::CurrentLine),
        )?;
    }

    if skip > 0 {
        queue!(stdout, MoveDown(1 + skip), MoveToColumn(0))?;
    }

    stdout.flush()
}
