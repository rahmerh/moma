use clap::Args;

use crate::sources::Source;

#[derive(Args)]
pub struct Connect {
    #[arg(value_enum)]
    pub source: Source,
}

impl Connect {
    pub fn run(&self) -> anyhow::Result<()> {
        self.source.setup()?;
        Ok(())
    }
}
