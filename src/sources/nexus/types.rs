use crate::types::{FileStatus, Mod, ModArchive};
use serde::Deserialize;

// Request structs

pub struct DownloadInfoRequest {
    pub game: String,
    pub mod_id: String,
    pub file_id: String,
    pub key: String,
    pub expires: String,
}

// Response structs

#[derive(Deserialize)]
pub struct ValidateResponse {
    pub name: String,
    pub is_premium: bool,
    pub key: String,
}

#[derive(Deserialize, Clone)]
pub struct DownloadInfoResponse {
    #[serde(rename = "URI")]
    pub uri: String,
}

#[derive(Deserialize)]
pub struct ModFileInfoResponse {
    pub uid: u64,
    pub file_name: String,
}

#[derive(Deserialize)]
pub struct ModInfoResponse {
    pub uid: u64,
    pub name: String,
}

impl From<ModFileInfoResponse> for ModArchive {
    fn from(value: ModFileInfoResponse) -> Self {
        ModArchive {
            file_uid: value.uid,
            archive_path: None,
            file_name: value.file_name,
            status: FileStatus::Unknown,
        }
    }
}

impl From<ModInfoResponse> for Mod {
    fn from(value: ModInfoResponse) -> Self {
        Mod {
            name: value.name,
            uid: value.uid,
            archives: vec![],
        }
    }
}
