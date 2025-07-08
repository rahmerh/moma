use std::{fmt::Display, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Bundle {
    pub uid: u64,
    pub name: String,
    pub archives: Vec<Archive>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Archive {
    pub file_uid: u64,
    pub file_name: String,
    pub archive_path: Option<PathBuf>,
    pub status: ArchiveStatus,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ArchiveStatus {
    Unknown,
    Downloading,
    Downloaded,
    Installed,
    Failed(String),
}

impl Display for Bundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for Archive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.file_name)
    }
}

impl Display for ArchiveStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            ArchiveStatus::Unknown => "Unknown",
            ArchiveStatus::Downloading => "Downloading",
            ArchiveStatus::Downloaded => "Downloaded",
            ArchiveStatus::Installed => "Installed",
            ArchiveStatus::Failed(str) => &format!("A problem occurred: '{}'", str),
        };

        write!(f, "{}", message)
    }
}
