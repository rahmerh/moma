use clap::Args;

#[derive(Args)]
pub struct Connect {}

impl Connect {
    pub async fn run(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
