use crate::{sources::nexus::client::File, types};

pub fn map_mod_files(files: Vec<File>) -> Vec<types::ModFile> {
    files
        .into_iter()
        .map(|f| types::ModFile {
            id: f.file_id.to_string(),
            name: f.name.to_string(),
            category: f.category_name,
            size_kb: f.size_kb,
        })
        .collect()
}
