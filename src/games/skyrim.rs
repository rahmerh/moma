use std::path::PathBuf;

use super::GameProfile;

pub struct Skyrim;

impl GameProfile for Skyrim {
    fn name(&self) -> &'static str {
        "Skyrim"
    }

    fn default_path(&self) -> PathBuf {
        PathBuf::from("~/.local/share/Steam/steamapps/common/Skyrim Special Edition")
    }

    fn setup_modding(&self) -> anyhow::Result<()> {
        println!("Setting up SKSE and default modding configuration for Skyrim...");

        // TODO: implement SKSE setup

        Ok(())
    }
}
