use std::fmt::Display;

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

pub mod nexus;

#[derive(clap::ValueEnum, Debug, Clone, Serialize, Deserialize, EnumIter)]
pub enum Source {
    Nexus,
    Test,
}

impl Source {
    pub fn setup(&self) -> anyhow::Result<()> {
        match self {
            Source::Nexus => {
                let platform = nexus::NexusPlatform;
                platform.setup()
            }
            Source::Test => todo!(),
        }
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Nexus => write!(f, "Nexus"),
            Source::Test => write!(f, "Test"),
        }
    }
}
