use std::{fmt::Display, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Mod {
    pub uid: u64,
    pub name: String,
    pub archives: Vec<ModArchive>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModArchive {
    pub file_uid: u64,
    pub file_name: String,
    pub archive_path: Option<PathBuf>,
    pub status: FileStatus,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum FileStatus {
    Unknown,
    Downloading,
    Downloaded,
    Installed,
    Failed(String),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModList {
    pub mods: Vec<Mod>,
}

#[derive(Serialize, Deserialize)]
pub struct DownloadProgress {
    pub file_name: String,
    pub progress_bytes: u64,
    pub total_bytes: u64,
    pub started_at: u64,
    pub updated_at: u64,
}

impl Display for ModArchive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.file_name)
    }
}

impl Display for Mod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for FileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            FileStatus::Unknown => "Unknown",
            FileStatus::Downloading => "Downloading",
            FileStatus::Downloaded => "Downloaded",
            FileStatus::Installed => "Installed",
            FileStatus::Failed(str) => &format!("A problem occurred: '{}'", str),
        };

        write!(f, "{}", message)
    }
}

impl Default for ModList {
    fn default() -> Self {
        Self { mods: vec![] }
    }
}
