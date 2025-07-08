use owo_colors::OwoColorize;

use crate::{
    sources::{Source, nexus::config::Config},
    ui::prompt,
};

mod api;
mod client;
mod config;
mod setup;
mod types;

pub struct Nexus;

impl Source for Nexus {
    async fn setup_source() -> anyhow::Result<()> {
        let mut config = Config::default();

        println!("\n{}\n", "Setting up Nexus mods source...".cyan().bold());

        let api_key = setup::resolve_api_key().await?;

        config.api_key = Some(api_key);
        config.save()?;

        println!(
            "\n{}\n{}",
            "Nexus API key successfully stored.".cyan().bold(),
            "Do you want to set up the nxm link handler?".cyan().bold()
        );

        if !prompt::confirm("Confirm")? {
            return Ok(());
        }

        setup::configure_nxm_link_handler()?;

        Ok(())
    }

    fn bundle_info_for(bundle_uid: u64) -> anyhow::Result<crate::bundles::types::Bundle> {
        todo!()
    }
}
