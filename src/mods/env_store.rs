use std::{collections::HashMap, fs};

use crate::games::workspace::Workspace;

pub struct EnvStore {
    workspace: Workspace,
}

impl EnvStore {
    pub fn new(workspace: Workspace) -> Self {
        Self { workspace }
    }

    pub fn read_env_vars(&self) -> anyhow::Result<HashMap<String, String>> {
        let path = self.workspace.work_dir().join("user-env.json");

        let contents = fs::read_to_string(path)?;
        let user_env: HashMap<String, String> = serde_json::from_str(&contents)?;

        Ok(user_env)
    }

    pub fn store_env_vars(&self, vars: HashMap<String, String>) -> anyhow::Result<()> {
        let path = self.workspace.work_dir().join("user-env.json");

        fs::write(path, serde_json::to_string_pretty(&vars)?)?;

        Ok(())
    }
}
