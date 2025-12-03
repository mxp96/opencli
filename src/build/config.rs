use crate::result::{OpenCliError, Result};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub build: Build,
    pub packages: Option<HashMap<SmolStr, PackageSpec>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Build {
    pub entry_file: PathBuf,
    pub output_file: PathBuf,
    pub compiler_version: String,
    pub includes: Option<BuildIncludes>,
    pub args: Option<BuildArgs>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildIncludes {
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildArgs {
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PackageSpec {
    Simple(SmolStr),
    Detailed {
        version: SmolStr,
        target: Option<PackageTarget>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageTarget {
    Components,
    Plugins,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            build: Build {
                entry_file: "gamemode.pwn".into(),
                output_file: "gamemode.amx".into(),
                compiler_version: "v3.10.11".to_string(),
                includes: Some(BuildIncludes {
                    paths: vec!["include".into(), "qawno/include".into()],
                }),
                args: Some(BuildArgs {
                    args: vec![
                        "-d3".to_string(),
                        "-;+".to_string(),
                        "-(+".to_string(),
                        "-\\+".to_string(),
                        "-Z+".to_string(),
                    ],
                }),
            },
            packages: None,
        }
    }
}

impl BuildConfig {
    pub async fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path).await?;
        let config: BuildConfig = toml::from_str(&content).map_err(|e| {
            OpenCliError::Config(format!("Invalid build config format: {}", e).into())
        })?;

        Ok(config)
    }

    pub async fn save_to_file(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self).map_err(|e| {
            OpenCliError::Config(format!("Failed to serialize build config: {}", e).into())
        })?;

        fs::write(path, content).await?;
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if self.build.entry_file.as_os_str().is_empty() {
            return Err(OpenCliError::Config("Entry file cannot be empty".into()));
        }

        if self.build.output_file.as_os_str().is_empty() {
            return Err(OpenCliError::Config("Output file cannot be empty".into()));
        }

        if self.build.compiler_version.is_empty() {
            return Err(OpenCliError::Config(
                "Compiler version cannot be empty".into(),
            ));
        }

        Ok(())
    }

    pub fn add_package(&mut self, name: SmolStr, spec: PackageSpec) {
        if self.packages.is_none() {
            self.packages = Some(HashMap::new());
        }
        self.packages.as_mut().unwrap().insert(name, spec);
    }

    pub fn remove_package(&mut self, name: &str) -> bool {
        if let Some(packages) = &mut self.packages {
            packages.remove(name).is_some()
        } else {
            false
        }
    }

    pub fn get_packages(&self) -> Option<&HashMap<SmolStr, PackageSpec>> {
        self.packages.as_ref()
    }

    pub fn get_include_paths(&self) -> Vec<PathBuf> {
        self.build
            .includes
            .as_ref()
            .map(|inc| inc.paths.clone())
            .unwrap_or_default()
    }
}

impl PackageSpec {
    pub fn version(&self) -> &str {
        match self {
            PackageSpec::Simple(version) => version,
            PackageSpec::Detailed { version, .. } => version,
        }
    }

    pub fn target(&self) -> Option<&PackageTarget> {
        match self {
            PackageSpec::Simple(_) => None,
            PackageSpec::Detailed { target, .. } => target.as_ref(),
        }
    }

    pub fn new_simple(version: impl Into<SmolStr>) -> Self {
        PackageSpec::Simple(version.into())
    }

    pub fn new_detailed(version: impl Into<SmolStr>, target: Option<PackageTarget>) -> Self {
        PackageSpec::Detailed {
            version: version.into(),
            target,
        }
    }
}
