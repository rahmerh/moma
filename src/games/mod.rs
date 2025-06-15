use std::path::PathBuf;

use crate::{
    config::{Config, GameConfig},
    games::skyrim::Skyrim,
};

pub mod skyrim;

mod utils;

pub trait GameProfile {
    fn name(&self) -> &'static str;
    fn default_path(&self) -> PathBuf;
    fn game_executable(&self) -> &'static str;
    fn setup_modding(&self, config: &Config, config: &GameConfig) -> anyhow::Result<()>;
}

pub fn get_supported_games() -> Vec<Box<dyn GameProfile>> {
    vec![Box::new(Skyrim {})]
}
