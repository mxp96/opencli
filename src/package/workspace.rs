use crate::result::{OpenCliError, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct WorkspaceDetector {
    root_path: PathBuf,
}

impl WorkspaceDetector {
    pub fn new<P: AsRef<Path>>(root_path: P) -> Self {
        Self {
            root_path: root_path.as_ref().to_path_buf(),
        }
    }

    pub async fn detect_components_folder(&self) -> Result<PathBuf> {
        let components_path = self.root_path.join("components");

        if !components_path.exists() {
            fs::create_dir_all(&components_path).await?;
            log::info!("Created components folder: {}", components_path.display());
        }

        Ok(components_path)
    }

    pub async fn detect_plugins_folder(&self) -> Result<PathBuf> {
        let plugins_path = self.root_path.join("plugins");

        if !plugins_path.exists() {
            fs::create_dir_all(&plugins_path).await?;
            log::info!("Created plugins folder: {}", plugins_path.display());
        }

        Ok(plugins_path)
    }

    pub async fn get_target_folder(&self, target: &str) -> Result<PathBuf> {
        match target.to_lowercase().as_str() {
            "components" => self.detect_components_folder().await,
            "plugins" => self.detect_plugins_folder().await,
            _ => Err(OpenCliError::Config(
                format!("Unknown target folder: {}", target).into(),
            )),
        }
    }

    pub async fn ensure_workspace_structure(&self) -> Result<()> {
        self.detect_components_folder().await?;
        self.detect_plugins_folder().await?;
        Ok(())
    }

    pub fn get_workspace_info(&self) -> WorkspaceInfo {
        WorkspaceInfo {
            root: self.root_path.clone(),
            components: self.root_path.join("components"),
            plugins: self.root_path.join("plugins"),
        }
    }
}

pub struct WorkspaceInfo {
    pub root: PathBuf,
    pub components: PathBuf,
    pub plugins: PathBuf,
}

impl WorkspaceInfo {
    pub async fn validate(&self) -> Result<()> {
        if !self.root.exists() {
            return Err(OpenCliError::NotFound("Workspace root not found".into()));
        }

        for path in [&self.components, &self.plugins] {
            if !path.exists() {
                fs::create_dir_all(path).await?;
                log::info!("Created workspace folder: {}", path.display());
            }
        }

        Ok(())
    }

    pub fn get_target_path(&self, target: &str) -> Result<&PathBuf> {
        match target.to_lowercase().as_str() {
            "components" => Ok(&self.components),
            "plugins" => Ok(&self.plugins),
            _ => Err(OpenCliError::Config(
                format!("Invalid target: {}", target).into(),
            )),
        }
    }
}
