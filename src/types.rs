use std::fmt::Display;

#[derive(Default)]
pub struct ModFiles {
    pub main: Vec<ModFile>,
    pub optional: Vec<ModFile>,
    pub misc: Vec<ModFile>,
    pub uncategorized: Vec<ModFile>,
    pub old_versions: Vec<ModFile>,
}

#[derive(Clone)]
pub struct ModFile {
    pub id: String,
    pub name: String,
    pub category: Option<String>,
    pub size_kb: Option<u64>,
}

impl Display for ModFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
