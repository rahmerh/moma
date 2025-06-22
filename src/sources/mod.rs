use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::{
    sources::nexus::Nexus,
    types::{ModFile, ModFiles},
};

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
    pub fn is_setup(&self) -> bool {
        match self {
            Source::Nexus => Nexus::is_setup(),
        }
    }

    pub fn setup(&self) -> anyhow::Result<()> {
        match self {
            Source::Nexus => Nexus::setup(),
        }
    }

    pub fn get_mod_files(&self, game: &str, mod_id: &str) -> anyhow::Result<ModFiles> {
        match self {
            Source::Nexus => Nexus::get_mod_files(game, mod_id),
        }
    }
}
