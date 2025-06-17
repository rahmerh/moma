use std::io::stdout;

use crossterm::{
    ExecutableCommand,
    cursor::MoveToColumn,
    style::Print,
    terminal::{Clear, ClearType},
};

pub fn print_inline_status(message: &str) -> anyhow::Result<()> {
    let mut stdout = stdout();
    stdout
        .execute(MoveToColumn(0))?
        .execute(Clear(ClearType::CurrentLine))?
        .execute(Print(message))?;
    Ok(())
}
