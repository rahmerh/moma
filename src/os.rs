use std::{io, process::Command};

use anyhow::Context;
use libc::{CLONE_NEWNS, getuid};

pub fn is_process_root() -> bool {
    unsafe { getuid() == 0 }
}

pub fn unshare_current_namespace() -> anyhow::Result<()> {
    let result = unsafe { libc::unshare(CLONE_NEWNS) };

    if result == -1 {
        let errno = io::Error::last_os_error();
        Err(anyhow::anyhow!("Failed to unshare namespace: {}", errno))
            .with_context(|| "unshare(CLONE_NEWNS | CLONE_NEWPID) failed")
    } else {
        Ok(())
    }
}

pub fn remount_current_namespace_as_private() -> anyhow::Result<()> {
    Command::new("mount")
        .args(["--make-rprivate", "/"])
        .status()
        .with_context(|| "Failed to set mount propagation to private")?
        .success()
        .then_some(())
        .ok_or_else(|| anyhow::anyhow!("mount --make-rprivate / failed"))
}

pub fn drop_privileges() -> anyhow::Result<()> {
    if !is_process_root() {
        return Err(anyhow::anyhow!(
            "Not running as root, cannot drop privileges"
        ));
    }

    unsafe {
        let uid = std::env::var("SUDO_UID")?.parse::<u32>()?;
        let gid = std::env::var("SUDO_GID")?.parse::<u32>()?;

        if libc::setgid(gid) != 0 {
            return Err(anyhow::anyhow!(
                "Failed to setgid: {}",
                std::io::Error::last_os_error()
            ));
        }

        if libc::setuid(uid) != 0 {
            return Err(anyhow::anyhow!(
                "Failed to setuid: {}",
                std::io::Error::last_os_error()
            ));
        }
    }

    Ok(())
}
