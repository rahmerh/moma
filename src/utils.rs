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

pub trait ExpandTilde {
    fn expand(&self) -> PathBuf;
}

impl ExpandTilde for Path {
    fn expand(&self) -> PathBuf {
        let s = self.to_string_lossy();
        PathBuf::from(shellexpand::tilde(&s).to_string())
    }
}

pub fn extract_archive(
    archive_path: &Path,
    target_dir: &Path,
    flatten: bool,
) -> anyhow::Result<()> {
    sevenz_rust2::decompress_file(archive_path, target_dir)?;

    if flatten {
        let entries: Vec<_> = fs::read_dir(target_dir)?.filter_map(Result::ok).collect();

        if entries.len() == 1 && entries[0].file_type()?.is_dir() {
            let inner_dir = entries[0].path();

            for entry in fs::read_dir(&inner_dir)? {
                let entry = entry?;
                let from = entry.path();
                let file_name = entry.file_name();
                let to = target_dir.join(file_name);
                fs::rename(from, to)?;
            }

            fs::remove_dir_all(inner_dir)?;
        }
    }

    Ok(())
}

pub fn print_inline_status(message: &str) -> anyhow::Result<()> {
    let mut stdout = stdout();
    stdout
        .execute(MoveToColumn(0))?
        .execute(Clear(ClearType::CurrentLine))?
        .execute(Print(message))?;
    Ok(())
}

pub fn copy_dir(from: &Path, to: &Path, recursive: bool, flatten: bool) -> io::Result<()> {
    if !from.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Source must be a directory",
        ));
    }

    let target = if flatten {
        to.to_path_buf()
    } else {
        to.join(from.file_name().unwrap())
    };

    fs::create_dir_all(&target)?;

    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = target.join(entry.file_name());
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            if recursive {
                copy_dir(&src_path, &dest_path, true, true)?;
            } else if !flatten {
                fs::create_dir_all(&dest_path)?;
            }
        } else if metadata.is_file() {
            fs::copy(&src_path, &dest_path)?;
        }
    }

    Ok(())
}
