use serde::Deserialize;

use crate::types::ModInfo;

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
pub struct ModInfoResponse {
    pub name: String,
}

impl From<ModInfoResponse> for ModInfo {
    fn from(value: ModInfoResponse) -> Self {
        ModInfo { name: value.name }
    }
}
