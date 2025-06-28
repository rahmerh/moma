use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, bail};
use unrar::Archive;
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
    let unrar_available = Command::new("unrar")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !unrar_available {
        bail!("Missing required dependency: `unrar`.");
    }

    let archive = Archive::new(&archive_path).open_for_processing()?;
    let archive = archive.read_header()?.expect("empty archive");
    archive.extract_with_base(target_dir)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::blocking::get;
    use std::io::Write;
    use tempfile::TempDir;
    use zip::{ZipWriter, write::SimpleFileOptions};

    fn setup() -> TempDir {
        TempDir::new().unwrap()
    }

    #[test]
    fn expand_should_expand_path_using_home_env_var() {
        // Arrange
        let home_dir = setup();

        unsafe { std::env::set_var("HOME", home_dir.path()) };
        let input = PathBuf::from("~/test");

        // Act
        let actual = input.expand();

        // Assert
        assert!(!actual.to_string_lossy().contains("~"));
        assert_eq!(
            actual.to_string_lossy(),
            home_dir.path().join("test").display().to_string()
        );
    }

    #[test]
    fn expand_should_do_nothing_if_no_tilde_given() {
        // Arrange
        let home_dir = setup();
        let input_str = "/tmp/test";

        unsafe { std::env::set_var("HOME", home_dir.path()) };
        let input = PathBuf::from(input_str);

        // Act
        let actual = input.expand();

        // Assert
        assert!(!actual.to_string_lossy().contains("~"));
        assert_eq!(actual.to_string_lossy(), input_str);
    }

    #[test]
    fn extract_archive_should_return_err_when_archive_non_existent() {
        // Arrange
        let tmp_dir = setup();

        // Act
        let result = extract_archive(
            &tmp_dir.path().join("text.zip"),
            &tmp_dir.path().join("output"),
            false,
        );

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn extract_archive_should_return_err_when_unsupported_extension_given() {
        // Arrange
        let tmp_dir = setup();
        let input_file = &tmp_dir.path().join("text.new");
        fs::write(input_file, "").unwrap();

        // Act
        let result = extract_archive(input_file, &tmp_dir.path().join("output"), false);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn extract_archive_should_return_err_when_file_has_no_extension() {
        // Arrange
        let tmp_dir = setup();
        let input_file = &tmp_dir.path().join("text");
        fs::write(input_file, "").unwrap();

        // Act
        let result = extract_archive(input_file, &tmp_dir.path().join("output"), false);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn extract_archive_should_extract_zip_correctly() {
        // Arrange
        let tmp_dir = setup();
        let input_file = tmp_dir.path().join("test.zip");
        let output_dir = tmp_dir.path().join("output");

        {
            let file = File::create(&input_file).unwrap();
            let mut zip = ZipWriter::new(file);

            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored)
                .unix_permissions(0o755);
            zip.start_file("nested/hello.txt", options).unwrap();
            zip.write_all(b"hello world").unwrap();
            zip.finish().unwrap();
        }

        // Act
        let result = extract_archive(&input_file, &output_dir, false);

        // Assert
        assert!(result.is_ok());

        assert!(output_dir.exists());

        let extracted_file = output_dir.join("nested").join("hello.txt");
        assert!(
            extracted_file.exists(),
            "Expected file {:?} to exist",
            extracted_file
        );

        let content = fs::read_to_string(&extracted_file).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn extract_archive_should_extract_7z_correctly() {
        // Arrange
        let tmp_dir = setup();
        let input_dir = tmp_dir.path().join("input/");
        fs::create_dir_all(&input_dir).unwrap();
        let input_file = input_dir.join("test.txt");
        let input_archive = tmp_dir.path().join("input.7z");

        let output_dir = tmp_dir.path().join("output/");
        fs::create_dir_all(&output_dir).unwrap();

        let expected = "Sample value";
        fs::write(&input_file, expected).unwrap();
        sevenz_rust2::compress_to_path(&input_dir, &input_archive).unwrap();

        // Act
        let result = extract_archive(&input_archive, &output_dir, false);

        // Assert
        assert!(result.is_ok());

        assert!(output_dir.exists());

        let extracted_file = output_dir.join("test.txt");
        assert!(extracted_file.exists());

        let content = fs::read_to_string(&extracted_file).unwrap();
        assert_eq!(content, expected);
    }

    #[test]
    fn extract_archive_should_flatten_output() {
        // Arrange
        let tmp_dir = setup();
        let input_file = tmp_dir.path().join("test.zip");
        let output_dir = tmp_dir.path().join("output");

        {
            let file = File::create(&input_file).unwrap();
            let mut zip = ZipWriter::new(file);

            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored)
                .unix_permissions(0o755);
            zip.start_file("nested/hello.txt", options).unwrap();
            zip.write_all(b"hello world").unwrap();
            zip.finish().unwrap();
        }

        // Act
        let result = extract_archive(&input_file, &output_dir, true);

        // Assert
        assert!(result.is_ok());

        assert!(output_dir.exists());

        let extracted_file = output_dir.join("hello.txt");
        assert!(extracted_file.exists());

        let content = fs::read_to_string(&extracted_file).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn extract_archive_should_not_flatten_when_there_are_no_sub_folders() {
        // Arrange
        let tmp_dir = setup();
        let input_file = tmp_dir.path().join("test.zip");
        let output_dir = tmp_dir.path().join("output");

        {
            let file = File::create(&input_file).unwrap();
            let mut zip = ZipWriter::new(file);

            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored)
                .unix_permissions(0o755);
            zip.start_file("hello.txt", options).unwrap();
            zip.write_all(b"hello world").unwrap();
            zip.finish().unwrap();
        }

        // Act
        let result = extract_archive(&input_file, &output_dir, true);

        // Assert
        assert!(result.is_ok());

        assert!(output_dir.exists());

        let extracted_file = output_dir.join("hello.txt");
        assert!(
            extracted_file.exists(),
            "Expected file {:?} to exist",
            extracted_file
        );

        let content = fs::read_to_string(&extracted_file).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn copy_dir_should_return_err_when_from_is_not_a_dir() {
        // Arrange
        let tmp_dir = setup();
    }
}
