use clap::Args;

use crate::config::Config;

#[derive(Args)]
pub struct Launch;

impl Launch {
    pub fn run(&self, config: &mut Config) -> anyhow::Result<()> {
        Ok(())
    }
}
