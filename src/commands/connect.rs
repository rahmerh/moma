use clap::Args;
use owo_colors::OwoColorize;

#[derive(Args)]
pub struct Connect;

impl Connect {
    pub fn run(&self) -> anyhow::Result<()> {
        println!("\n{}\n\n", "Connect to Nexus Mods".bold());

        let _ = std::io::stdin().read_line(&mut String::new());

        Ok(())
    }
}
