use std::{env, ffi::CString, fs, os::unix::ffi::OsStrExt, path::Path};

use anyhow::Context;
use libc::{chown, getuid, gid_t, uid_t};

#[cfg_attr(test, mockall::automock)]
pub trait System {
    fn is_process_root(&self) -> bool;
    fn drop_privileges(&self) -> anyhow::Result<()>;
    fn chown_dir(&self, dir: &Path, recursive: bool) -> anyhow::Result<()>;
}

pub struct HostSystem;

impl System for HostSystem {
    fn is_process_root(&self) -> bool {
        unsafe { getuid() == 0 }
    }

    fn drop_privileges(&self) -> anyhow::Result<()> {
        if !self.is_process_root() {
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

    fn chown_dir(&self, dir: &Path, recursive: bool) -> anyhow::Result<()> {
        let uid = env::var("SUDO_UID")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or_else(|| unsafe { libc::getuid() });

        let gid = env::var("SUDO_GID")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or_else(|| unsafe { libc::getgid() });

        if recursive {
            for entry in fs::read_dir(dir)
                .with_context(|| format!("Failed to read dir: {}", dir.display()))?
            {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    self.chown_dir(&path, true).with_context(|| {
                        format!("Failed to recurse into dir: {}", path.display())
                    })?;
                }

                chown_path(&path, uid, gid)
                    .with_context(|| format!("Failed to chown: {}", path.display()))?;
            }
        }

        chown_path(dir, uid, gid)
    }
}

fn chown_path(path: &Path, uid: uid_t, gid: gid_t) -> anyhow::Result<()> {
    let c_path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid path"))?;

    let result = unsafe { chown(c_path.as_ptr(), uid, gid) };

    if result != 0 {
        return Err(anyhow::anyhow!(
            "Failed to chown '{}': {}",
            path.display(),
            std::io::Error::last_os_error()
        ));
    }

    Ok(())
}
