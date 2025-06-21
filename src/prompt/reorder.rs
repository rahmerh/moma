use std::{
    fmt::Display,
    io::{Write, stdout},
    time::Duration,
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, queue,
    style::Print,
    terminal::{self, ClearType},
};
use owo_colors::OwoColorize;

pub fn reorder_items<T: Display + Clone>(mut items: Vec<T>) -> anyhow::Result<Vec<T>> {
    let mut cursor_index = 0;
    let mut selected_index: Option<usize> = None;

    terminal::enable_raw_mode()?;

    let mut stdout = stdout();

    let (_, y) = cursor::position()?;

    execute!(stdout, cursor::Hide)?;
    loop {
        for (i, item) in items.iter().enumerate() {
            queue!(stdout, terminal::Clear(ClearType::CurrentLine))?;
            let styled_item = if Some(i) == selected_index {
                format!("{}", item).bold().cyan().to_string()
            } else {
                format!("{}", item)
            };

            if i == cursor_index {
                queue!(stdout, Print(format!("\r> {}\n", styled_item)))?;
            } else {
                queue!(stdout, Print(format!("\r  {}\n", styled_item)))?;
            }
        }

        queue!(stdout, cursor::MoveUp(items.len() as u16))?;

        stdout.flush()?;

        if event::poll(Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => {
                        if let Some(selected) = selected_index {
                            if selected > 0 {
                                items.swap(selected, selected - 1);
                                selected_index = Some(selected - 1);
                                cursor_index = selected - 1;
                            }
                        } else if cursor_index > 0 {
                            cursor_index -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if let Some(selected) = selected_index {
                            if selected < items.len() - 1 {
                                items.swap(selected, selected + 1);
                                selected_index = Some(selected + 1);
                                cursor_index = selected + 1;
                            }
                        } else if cursor_index < items.len() - 1 {
                            cursor_index += 1;
                        }
                    }
                    KeyCode::Char(' ') => {
                        if selected_index.is_some() {
                            selected_index = None;
                        } else {
                            selected_index = Some(cursor_index);
                        }
                    }
                    KeyCode::Enter => break,
                    _ => {}
                }
            }
        }
    }
    execute!(stdout, terminal::Clear(terminal::ClearType::FromCursorDown))?;
    execute!(stdout, cursor::MoveTo(0, y))?;
    execute!(stdout, cursor::Show)?;

    terminal::disable_raw_mode()?;

    Ok(items)
}
