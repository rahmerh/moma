use clap::Args;

use crate::sources::{Source, nexus::Nexus};

#[derive(Args)]
pub struct Connect {
    #[arg(value_enum)]
    pub source: Source,
}

impl Connect {
    pub fn run(&self) -> anyhow::Result<()> {
        match self.source {
            Source::Nexus => {
                Nexus::setup()?;
            }
        }
        Ok(())
    }
}
