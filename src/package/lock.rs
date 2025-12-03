use crate::build::PackageTarget;
use crate::result::Result;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPackage {
    pub version: SmolStr,
    pub target: Option<PackageTarget>,
    pub hash: SmolStr,
    pub installed_at: SmolStr,
    pub files: Vec<SmolStr>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PackageLock {
    pub installed: HashMap<SmolStr, InstalledPackage>,
}

impl PackageLock {
    pub async fn load_from_file(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path).await?;
        let lock: PackageLock = toml::from_str(&content)?;
        Ok(lock)
    }

    pub async fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(path, content).await?;
        Ok(())
    }

    pub fn add_package(
        &mut self,
        name: SmolStr,
        version: SmolStr,
        target: Option<PackageTarget>,
        hash: SmolStr,
        files: Vec<SmolStr>,
    ) {
        let installed_at = chrono::Utc::now().to_rfc3339().into();
        let package = InstalledPackage {
            version,
            target,
            hash,
            installed_at,
            files,
        };
        self.installed.insert(name, package);
    }

    pub fn remove_package(&mut self, name: &str) -> Option<InstalledPackage> {
        self.installed.remove(name)
    }

    pub fn get_package(&self, name: &str) -> Option<&InstalledPackage> {
        self.installed.get(name)
    }

    pub fn is_package_installed(&self, name: &str) -> bool {
        self.installed.contains_key(name)
    }

    pub fn get_installed_version(&self, name: &str) -> Option<&str> {
        self.installed.get(name).map(|p| p.version.as_str())
    }

    pub fn list_packages(&self) -> Vec<(&str, &InstalledPackage)> {
        self.installed
            .iter()
            .map(|(k, v)| (k.as_str(), v))
            .collect()
    }
}
