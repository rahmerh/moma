use clap::Args;

#[derive(Args)]
pub struct ListStaged {}

impl ListStaged {
    pub fn run(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
