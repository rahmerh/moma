use std::{
    fs::{self},
    path::PathBuf,
};

use anyhow::{Context, bail};

use crate::{
    games::workspace::{self, Workspace},
    types::{Mod, ModArchive},
    utils,
};

pub struct Manager {
    workspace: Workspace,
}

impl Manager {
    pub fn new(workspace: Workspace) -> Self {
        Self { workspace }
    }
}
