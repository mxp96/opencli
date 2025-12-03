use crate::result::{OpenCliError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    pub darwin: Option<PlatformConfig>,
    pub linux: Option<PlatformConfig>,
    pub windows: Option<PlatformConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    #[serde(rename = "match")]
    pub match_pattern: String,
    pub method: String,
    pub binary: String,
    pub paths: HashMap<String, String>,
}

impl CompilerConfig {
    pub async fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path).await?;
        let config: CompilerConfig = toml::from_str(&content).map_err(|e| {
            OpenCliError::Config(format!("Invalid compiler config format: {}", e).into())
        })?;

        Ok(config)
    }

    pub async fn save_to_file(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self).map_err(|e| {
            OpenCliError::Config(format!("Failed to serialize compiler config: {}", e).into())
        })?;

        fs::write(path, content).await?;
        Ok(())
    }

    pub fn get_platform_config(&self) -> Option<&PlatformConfig> {
        if cfg!(target_os = "windows") {
            self.windows.as_ref()
        } else if cfg!(target_os = "linux") {
            self.linux.as_ref()
        } else if cfg!(target_os = "macos") {
            self.darwin.as_ref()
        } else {
            None
        }
    }
}
