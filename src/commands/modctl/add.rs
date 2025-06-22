use crate::{
    config::Config,
    sources::nexus::{self, Nexus},
};

pub struct Add;

impl Add {
    pub fn run(game_name: &str, mod_id: &str, config: &mut Config) -> anyhow::Result<()> {
        Ok(())
    }
}
