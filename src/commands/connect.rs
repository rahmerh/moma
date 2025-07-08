use clap::Args;
use owo_colors::OwoColorize;

use crate::{
    sources::{
        Source,
        nexus::{self, Config},
    },
    ui::prompt,
};

#[derive(Args)]
pub struct Connect {
    #[arg(value_enum)]
    pub source: Source,
}

impl Connect {
    pub async fn run(&self) -> anyhow::Result<()> {
        match self.source {
            Source::Nexus => {
                let mut config = match Config::load() {
                    Ok(config) => {
                        if !prompt::confirm(
                            "Nexus connection already set up, do you want to overwrite?",
                        )? {
                            println!("{}", "Exiting setup.".yellow());
                            return Ok(());
                        }
                        config
                    }
                    Err(_) => Config::default(),
                };

                let api_key = nexus::resolve_api_key().await?;

                config.api_key = Some(api_key);
                config.save()?;

                nexus::configure_nxm_link_handler()?;
            }
        }
        Ok(())
    }
}
