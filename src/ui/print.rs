use std::io::stdout;

use crossterm::{
    ExecutableCommand,
    cursor::MoveToColumn,
    style::Print,
    terminal::{Clear, ClearType},
};

pub trait Colorize {
    fn dark_cyan(self) -> String;
}

impl<T: AsRef<str>> Colorize for T {
    fn dark_cyan(self) -> String {
        hex_escape("#007392", self)
    }
}

pub fn print_inline_status(message: &str) -> anyhow::Result<()> {
    let mut stdout = stdout();
    stdout
        .execute(MoveToColumn(0))?
        .execute(Clear(ClearType::CurrentLine))?
        .execute(Print(message))?;
    Ok(())
}

pub fn hex_escape<T: AsRef<str>>(hex: &str, text: T) -> String {
    let hex = hex.trim_start_matches('#');
    let text = text.as_ref();

    if hex.len() != 6 {
        return text.to_string();
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);

    format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}
