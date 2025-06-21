use clap::Args;

use crate::mod_platforms::ModPlatformKind;

#[derive(Args)]
pub struct Connect {
    #[arg(value_enum)]
    pub mod_platform: ModPlatformKind,
}

impl Connect {
    pub fn run(&self) -> anyhow::Result<()> {
        self.mod_platform.setup()?;
        Ok(())
    }
}
