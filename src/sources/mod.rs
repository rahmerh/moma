use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::sources::nexus::Nexus;

mod nexus;

#[derive(clap::ValueEnum, Debug, Clone, Serialize, Deserialize, EnumIter)]
pub enum Source {
    Nexus,
}

impl Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Source::Nexus => "Nexus",
        };

        write!(f, "{}", name)
    }
}

impl Source {
    pub async fn setup(&self) -> anyhow::Result<()> {
        match self {
            Source::Nexus => Nexus::setup().await,
        }
    }
}
