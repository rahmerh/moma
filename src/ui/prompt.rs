use std::path::{Path, PathBuf};

use dialoguer::Input;

use crate::{ui::theme, utils::fs::ExpandTilde};

pub fn prompt_for_path(prompt_text: &str, default_value: Option<&str>) -> anyhow::Result<PathBuf> {
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
