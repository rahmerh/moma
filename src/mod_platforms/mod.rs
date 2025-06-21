use std::fmt::Display;

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

pub mod nexus;

pub trait ModPlatform {
    fn setup(&self) -> anyhow::Result<()>;
}

#[derive(clap::ValueEnum, Debug, Clone, Serialize, Deserialize, EnumIter)]
pub enum ModPlatformKind {
    Nexus,
}

impl ModPlatformKind {
    pub fn setup(&self) -> anyhow::Result<()> {
        match self {
            ModPlatformKind::Nexus => {
                let platform = nexus::NexusPlatform;
                platform.setup()
            }
        }
    }
}

impl Display for ModPlatformKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModPlatformKind::Nexus => write!(f, "Nexus"),
        }
    }
}
