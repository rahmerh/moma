use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ModInfo {
    pub uid: u64,
    pub name: String,
    pub downloaded_archives: Vec<ModFileInfo>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModFileInfo {
    pub uid: u64,
    pub file_name: String,
    pub status: InstallationStatus,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum InstallationStatus {
    Pending,
    Installed,
    Error,
}

impl Display for ModInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for InstallationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            InstallationStatus::Pending => "Installation pending",
            InstallationStatus::Installed => "Installed",
            InstallationStatus::Error => "Something went wrong",
        };

        write!(f, "{}", message)
    }
}
