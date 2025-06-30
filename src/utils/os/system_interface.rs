use std::{io, process::Command};

use anyhow::Context;
use libc::CLONE_NEWNS;
use mockall::automock;

#[automock]
pub trait SystemInterface {
    fn run_command(&self, cmd: &mut Command) -> anyhow::Result<()>;
    fn unshare_namespace(&self) -> anyhow::Result<()>;
}

pub struct System;

impl SystemInterface for System {
    fn run_command(&self, cmd: &mut Command) -> anyhow::Result<()> {
        let status = cmd.status().with_context(|| "Command failed to spawn")?;
        if !status.success() {
            return Err(anyhow::anyhow!("Command failed: {:?}", cmd));
        }
        Ok(())
    }

    fn unshare_namespace(&self) -> anyhow::Result<()> {
        let result = unsafe { libc::unshare(CLONE_NEWNS) };
        if result == -1 {
            return Err(io::Error::last_os_error().into());
        }
        Ok(())
    }
}
