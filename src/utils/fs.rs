use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};
use unrar::{Archive, OpenArchive};
use zip::ZipArchive;

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
    if !archive_path.exists() {
        bail!("Archive at '{}' does not exist.", archive_path.display());
    }

    let ext = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    match ext.as_deref() {
        Some("zip") => extract_zip(archive_path, target_dir)?,
        Some("7z") => sevenz_rust2::decompress_file(archive_path, target_dir)?,
        Some("rar") => extract_rar(archive_path, target_dir)?,
        Some(ext) => bail!("Unsupported archive format: .{}", ext),
        None => bail!("Archive file has no extension"),
    }

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

fn extract_zip(archive_path: &Path, target_dir: &Path) -> anyhow::Result<()> {
    let file = File::open(archive_path).context("Failed to open ZIP archive")?;
    let mut archive = ZipArchive::new(file).context("Failed to read ZIP archive")?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = target_dir.join(file.mangled_name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(p) = out_path.parent() {
                fs::create_dir_all(p)?;
            }

            let mut out_file = fs::File::create(&out_path)?;
            std::io::copy(&mut file, &mut out_file)?;
        }
    }

    Ok(())
}

fn extract_rar(archive_path: &Path, target_dir: &Path) -> anyhow::Result<()> {
    let archive = Archive::new(&archive_path).open_for_processing()?;
    let archive = archive.read_header()?.expect("empty archive");
    archive.extract_with_base(target_dir)?;

    Ok(())
}
