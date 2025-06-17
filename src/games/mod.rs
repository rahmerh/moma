use std::path::{Path, PathBuf};

use crate::{
    config::{Config, GameConfig},
    games::skyrim::SkyrimSe,
};

pub mod skyrim;

pub trait GameProfile {
    fn name(&self) -> &'static str;
    fn default_game_path(&self, steam_dir: &Path) -> PathBuf;
    fn game_executable(&self) -> &'static str;
    fn game_mod_executable(&self) -> &'static str;
    fn setup_modding(&self, config: &Config, config: &GameConfig) -> anyhow::Result<()>;
}

pub fn get_supported_games() -> Vec<Box<dyn GameProfile>> {
    vec![Box::new(SkyrimSe {})]
}

pub fn get_game_profile_by_name(name: &str) -> Option<Box<dyn GameProfile>> {
    get_supported_games()
        .into_iter()
        .find(|g| g.name().eq_ignore_ascii_case(name))
}
