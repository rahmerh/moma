use crate::{sources::nexus::client::File, types};

pub fn map_mod_files(files: Vec<File>) -> Vec<types::ModFile> {
    files
        .iter()
        .map(|f| types::ModFile {
            id: f.file_id.to_string(),
            category: "".to_string(),
        })
        .collect()
}
