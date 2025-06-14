use std::path::PathBuf;

use crate::games::skyrim::Skyrim;

pub mod skyrim;

pub trait GameProfile {
    fn name(&self) -> &'static str;
    fn default_path(&self) -> PathBuf;
    fn setup_modding(&self) -> anyhow::Result<()>;
}

pub fn get_supported_games() -> Vec<Box<dyn GameProfile>> {
    vec![Box::new(Skyrim {})]
}
