use std::{fmt::Display, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Mod {
    pub uid: u64,
    pub name: String,
    pub downloaded_archives: Vec<ModArchive>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModArchive {
    pub file_uid: u64,
    pub file_name: String,
    pub archive_path: PathBuf,
    pub status: FileStatus,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FileStatus {
    Unknown,
    Downloading,
    Downloaded,
    Installed,
    Error,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModList {
    pub mods: Vec<Mod>,
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
            FileStatus::Error => "Something went wrong",
        };

        write!(f, "{}", message)
    }
}

impl Default for ModList {
    fn default() -> Self {
        Self { mods: vec![] }
    }
}
