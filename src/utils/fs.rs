use std::{
    fs::{self, File},
    io::{self},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, bail};
use unrar::Archive;
use walkdir::WalkDir;
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

#[cfg_attr(test, mockall::automock)]
pub trait FileSystem {
    fn extract_archive(
        &self,
        archive_path: &Path,
        target_dir: &Path,
        flatten: bool,
    ) -> anyhow::Result<()>;
    fn copy_dir(&self, from: &Path, to: &Path, flatten_root: bool) -> anyhow::Result<()>;
    fn create_dir_all(&self, dir: &Path) -> anyhow::Result<()>;
}

pub struct StdFileSystem;

impl FileSystem for StdFileSystem {
    fn extract_archive(
        &self,
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

    fn copy_dir(&self, from: &Path, to: &Path, flatten_root: bool) -> anyhow::Result<()> {
        if !from.is_dir() {
            bail!("Source must be a directory");
        }

        let root_name = from.file_name().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Source has no final path component",
            )
        })?;

        for entry in WalkDir::new(from)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let src_path = entry.path();
            let relative = src_path.strip_prefix(from).unwrap();
            let dest_path = if flatten_root {
                to.join(relative)
            } else {
                to.join(root_name).join(relative)
            };

            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::copy(src_path, dest_path)?;
        }

        Ok(())
    }

    fn create_dir_all(&self, dir: &Path) -> anyhow::Result<()> {
        fs::create_dir_all(dir).with_context(|| format!("Could not create dir: {}", dir.display()))
    }
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

    let mut archive = Archive::new(&archive_path).open_for_processing()?;

    while let Some(header) = archive.read_header()? {
        println!(
            "{} bytes: {}",
            header.entry().unpacked_size,
            header.entry().filename.to_string_lossy(),
        );
        archive = if header.entry().is_file() {
            header.extract_with_base(target_dir)?
        } else {
            header.skip()?
        };
    }

    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::io::Write;
//     use tempfile::TempDir;
//     use zip::{ZipWriter, write::SimpleFileOptions};
//
//     fn setup() -> TempDir {
//         TempDir::new().unwrap()
//     }
//
//     #[test]
//     fn expand_should_expand_path_using_home_env_var() {
//         // Arrange
//         let home_dir = setup();
//
//         unsafe { std::env::set_var("HOME", home_dir.path()) };
//         let input = PathBuf::from("~/test");
//
//         // Act
//         let actual = input.expand();
//
//         // Assert
//         assert!(!actual.to_string_lossy().contains("~"));
//         assert_eq!(
//             actual.to_string_lossy(),
//             home_dir.path().join("test").display().to_string()
//         );
//     }
//
//     #[test]
//     fn expand_should_do_nothing_if_no_tilde_given() {
//         // Arrange
//         let home_dir = setup();
//         let input_str = "/tmp/test";
//
//         unsafe { std::env::set_var("HOME", home_dir.path()) };
//         let input = PathBuf::from(input_str);
//
//         // Act
//         let actual = input.expand();
//
//         // Assert
//         assert!(!actual.to_string_lossy().contains("~"));
//         assert_eq!(actual.to_string_lossy(), input_str);
//     }
//
//     #[test]
//     fn extract_archive_should_return_err_when_archive_non_existent() {
//         // Arrange
//         let tmp_dir = setup();
//
//         // Act
//         let result = extract_archive(
//             &tmp_dir.path().join("text.zip"),
//             &tmp_dir.path().join("output"),
//             false,
//         );
//
//         // Assert
//         assert!(result.is_err());
//     }
//
//     #[test]
//     fn extract_archive_should_return_err_when_unsupported_extension_given() -> anyhow::Result<()> {
//         // Arrange
//         let tmp_dir = setup();
//         let input_file = &tmp_dir.path().join("text.new");
//         fs::write(input_file, "")?;
//
//         // Act
//         let result = extract_archive(input_file, &tmp_dir.path().join("output"), false);
//
//         // Assert
//         assert!(result.is_err());
//
//         Ok(())
//     }
//
//     #[test]
//     fn extract_archive_should_return_err_when_file_has_no_extension() -> anyhow::Result<()> {
//         // Arrange
//         let tmp_dir = setup();
//         let input_file = &tmp_dir.path().join("text");
//         fs::write(input_file, "")?;
//
//         // Act
//         let result = extract_archive(input_file, &tmp_dir.path().join("output"), false);
//
//         // Assert
//         assert!(result.is_err());
//
//         Ok(())
//     }
//
//     #[test]
//     fn extract_archive_should_extract_zip_correctly() -> anyhow::Result<()> {
//         // Arrange
//         let tmp_dir = setup();
//         let input_file = tmp_dir.path().join("input.zip");
//         let output_dir = tmp_dir.path().join("output/");
//
//         {
//             let file = File::create(&input_file)?;
//             let mut zip = ZipWriter::new(file);
//
//             let options = SimpleFileOptions::default()
//                 .compression_method(zip::CompressionMethod::Stored)
//                 .unix_permissions(0o755);
//             zip.start_file("hello.txt", options)?;
//             zip.write_all(b"hello world")?;
//             zip.finish()?;
//         }
//
//         // Act
//         let result = extract_archive(&input_file, &output_dir, false);
//
//         // Assert
//         assert!(result.is_ok());
//
//         assert!(output_dir.exists());
//
//         let extracted_file = output_dir.join("hello.txt");
//         assert!(
//             extracted_file.exists(),
//             "Expected file {:?} to exist",
//             extracted_file
//         );
//
//         let content = fs::read_to_string(&extracted_file)?;
//         assert_eq!(content, "hello world");
//
//         Ok(())
//     }
//
//     #[test]
//     fn extract_archive_should_extract_7z_correctly() -> anyhow::Result<()> {
//         // Arrange
//         let tmp_dir = setup();
//         let input_dir = tmp_dir.path().join("input/");
//         fs::create_dir_all(&input_dir)?;
//         let input_file = input_dir.join("test.txt");
//         let input_archive = tmp_dir.path().join("input.7z");
//
//         let output_dir = tmp_dir.path().join("output/");
//         fs::create_dir_all(&output_dir)?;
//
//         let expected = "Sample value";
//         fs::write(&input_file, expected)?;
//         sevenz_rust2::compress_to_path(&input_dir, &input_archive)?;
//
//         // Act
//         let result = extract_archive(&input_archive, &output_dir, false);
//
//         // Assert
//         assert!(result.is_ok());
//
//         assert!(output_dir.exists());
//
//         let extracted_file = output_dir.join("test.txt");
//         assert!(extracted_file.exists());
//
//         let content = fs::read_to_string(&extracted_file)?;
//         assert_eq!(content, expected);
//
//         Ok(())
//     }
//
//     #[test]
//     fn extract_archive_should_flatten_output() -> anyhow::Result<()> {
//         // Arrange
//         let tmp_dir = setup();
//         let input_file = tmp_dir.path().join("test.zip");
//         let output_dir = tmp_dir.path().join("output");
//
//         {
//             let file = File::create(&input_file)?;
//             let mut zip = ZipWriter::new(file);
//
//             let options = SimpleFileOptions::default()
//                 .compression_method(zip::CompressionMethod::Stored)
//                 .unix_permissions(0o755);
//             zip.start_file("nested/hello.txt", options)?;
//             zip.write_all(b"hello world")?;
//             zip.finish()?;
//         }
//
//         // Act
//         let result = extract_archive(&input_file, &output_dir, true);
//
//         // Assert
//         assert!(result.is_ok());
//
//         assert!(output_dir.exists());
//
//         let extracted_file = output_dir.join("hello.txt");
//         assert!(extracted_file.exists());
//
//         let content = fs::read_to_string(&extracted_file)?;
//         assert_eq!(content, "hello world");
//
//         Ok(())
//     }
//
//     #[test]
//     fn extract_archive_should_not_flatten_when_there_are_no_sub_folders() -> anyhow::Result<()> {
//         // Arrange
//         let tmp_dir = setup();
//         let input_file = tmp_dir.path().join("test.zip");
//         let output_dir = tmp_dir.path().join("output");
//
//         {
//             let file = File::create(&input_file)?;
//             let mut zip = ZipWriter::new(file);
//
//             let options = SimpleFileOptions::default()
//                 .compression_method(zip::CompressionMethod::Stored)
//                 .unix_permissions(0o755);
//             zip.start_file("hello.txt", options)?;
//             zip.write_all(b"hello world")?;
//             zip.finish()?;
//         }
//
//         // Act
//         let result = extract_archive(&input_file, &output_dir, true);
//
//         // Assert
//         assert!(result.is_ok());
//
//         assert!(output_dir.exists());
//
//         let extracted_file = output_dir.join("hello.txt");
//         assert!(
//             extracted_file.exists(),
//             "Expected file {:?} to exist",
//             extracted_file
//         );
//
//         let content = fs::read_to_string(&extracted_file)?;
//         assert_eq!(content, "hello world");
//
//         Ok(())
//     }
//
//     #[test]
//     fn copy_dir_should_return_err_when_from_is_not_a_dir() {
//         // Arrange
//         let tmp_dir = setup();
//
//         let input = tmp_dir.path().join("file.txt");
//         let output = tmp_dir.path().join("output/");
//
//         // Act
//         let result = copy_dir(&input, &output, false);
//
//         // Assert
//         assert!(result.is_err());
//     }
//
//     #[test]
//     fn copy_dir_should_return_err_when_from_has_no_final_component() {
//         // Arrange
//         let tmp_dir = setup();
//
//         let input = Path::new("/");
//         let output = tmp_dir.path().join("output/");
//
//         // Act
//         let result = copy_dir(&input, &output, false);
//
//         // Assert
//         assert!(result.is_err());
//     }
//
//     #[test]
//     fn copy_dir_should_copy_file_structure() -> anyhow::Result<()> {
//         // Arrange
//         let tmp_dir = setup();
//
//         let input_dir = tmp_dir.path().join("input");
//         let nested = input_dir.join("nested");
//         fs::create_dir_all(&nested)?;
//
//         let input_content = "Sample value";
//         let input = nested.join("file.txt");
//         fs::write(&input, input_content)?;
//
//         let output = tmp_dir.path().join("output");
//
//         // Act
//         let result = copy_dir(&input_dir, &output, false);
//
//         // Assert
//         assert!(result.is_ok());
//
//         let output_file = tmp_dir.path().join("output/input/nested/file.txt");
//         let _ = tmp_dir.keep();
//
//         assert!(output_file.exists());
//
//         let content = fs::read_to_string(output_file)?;
//
//         assert_eq!(content, input_content);
//
//         Ok(())
//     }
//
//     #[test]
//     fn copy_dir_should_should_flatten_root() -> anyhow::Result<()> {
//         // Arrange
//         let tmp_dir = setup();
//
//         let input_dir = tmp_dir.path().join("input");
//         let nested = input_dir.join("nested");
//         fs::create_dir_all(&nested)?;
//
//         let input_content = "Sample value";
//         let input = nested.join("file.txt");
//         fs::write(&input, input_content)?;
//
//         let output = tmp_dir.path().join("output");
//
//         // Act
//         let result = copy_dir(&input_dir, &output, true);
//
//         // Assert
//         assert!(result.is_ok());
//
//         let output_file = tmp_dir.path().join("output/nested/file.txt");
//         let _ = tmp_dir.keep();
//
//         assert!(output_file.exists());
//
//         let content = fs::read_to_string(output_file)?;
//
//         assert_eq!(content, input_content);
//
//         Ok(())
//     }
// }
