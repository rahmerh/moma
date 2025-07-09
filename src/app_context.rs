use std::path::Path;

use crate::{
    bay::bay::Bay,
    games::game::Game,
    ui::prompt::{Prompt, Prompter},
    utils::{
        fs::{FileSystem, StdFileSystem},
        os::permissions::{HostSystem, System},
    },
};

pub struct AppContext {
    fs: Box<dyn FileSystem>,
    sys: Box<dyn System>,
    prompt: Box<dyn Prompt>,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            fs: Box::new(StdFileSystem),
            sys: Box::new(HostSystem),
            prompt: Box::new(Prompter),
        }
    }

    pub fn fs(&self) -> &dyn FileSystem {
        &*self.fs
    }

    pub fn sys(&self) -> &dyn System {
        &*self.sys
    }

    pub fn prompt(&self) -> &dyn Prompt {
        &*self.prompt
    }

    pub fn bay(&self, game: &Game, base_dir: &Path) -> Bay {
        Bay::new(self.fs(), self.sys(), base_dir, game)
    }
}
