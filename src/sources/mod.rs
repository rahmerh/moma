use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

pub mod nexus;

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
