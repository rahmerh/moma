#[derive(Default)]
pub struct ModFiles {
    pub main: Vec<ModFile>,
    pub optional: Vec<ModFile>,
    pub update: Vec<ModFile>,
    pub patch: Vec<ModFile>,
    pub misc: Vec<ModFile>,
    pub uncategorized: Vec<ModFile>,
    pub old_versions: Vec<ModFile>,
}

pub struct ModFile {
    pub id: String,
    pub category: String,
}
