pub mod nexus;

#[derive(clap::ValueEnum, Clone)]
pub enum ModPlatform {
    Nexus,
}

impl ModPlatform {
    pub fn name(&self) -> &'static str {
        match self {
            ModPlatform::Nexus => "Nexus",
        }
    }
}

pub fn get_supported_mod_platforms() -> Vec<ModPlatform> {
    vec![ModPlatform::Nexus]
}
