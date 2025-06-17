use std::{
    fs,
    io::{self, stdout},
    path::{Path, PathBuf},
};

use crossterm::{
    ExecutableCommand,
    cursor::MoveToColumn,
    style::Print,
    terminal::{Clear, ClearType},
};
