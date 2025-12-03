use crate::build::{BuildConfig, PackageSpec, PackageTarget};
use crate::cache::CacheManager;
use crate::package::{
    ConfigManager, PackageDownloader, PackageLock, VersionConstraint, WorkspaceDetector,
};
use crate::result::{OpenCliError, Result};
use crate::security::SecurityManager;
use indicatif::{ProgressBar, ProgressStyle};
use smol_str::SmolStr;
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct PackageManager {
    downloader: PackageDownloader,
    workspace: WorkspaceDetector,
    security: SecurityManager,
    cache: CacheManager,
    config_manager: ConfigManager,
    config_path: PathBuf,
    lock_path: PathBuf,
}

impl PackageManager {
    pub fn new<P: AsRef<Path>>(workspace_root: P, config_path: P) -> Self {
        let workspace_path = workspace_root.as_ref();
        let config_path_buf = config_path.as_ref().to_path_buf();
        let lock_path = config_path_buf.with_extension("lock");

        Self {
            downloader: PackageDownloader::new(),
            workspace: WorkspaceDetector::new(&workspace_root),
            security: SecurityManager::new(),
            cache: CacheManager::new(workspace_path),
            config_manager: ConfigManager::new(workspace_path),
            config_path: config_path_buf,
            lock_path,
        }
    }

    pub async fn install_package(
        &mut self,
        repo: &str,
        version_spec: Option<&str>,
        target: Option<PackageTarget>,
    ) -> Result<()> {
        let spinner = self.create_spinner("Installing package...");

        spinner.set_message("Checking lock file...");
        let mut lock = PackageLock::load_from_file(&self.lock_path).await?;

        if lock.is_package_installed(repo) {
            let installed_version = lock.get_installed_version(repo).unwrap();
            spinner.finish_with_message(format!(
                "Package {} {} already installed",
                repo, installed_version
            ));
            println!(
                "Package {} {} is already installed",
                repo, installed_version
            );
            return Ok(());
        }

        let constraint = if let Some(spec) = version_spec {
            VersionConstraint::parse(spec)?
        } else {
            VersionConstraint::parse("*")?
        };

        spinner.set_message(format!("Finding version for {}", repo));
        let release = self
            .downloader
            .find_matching_version(repo, &constraint)
            .await?;

        spinner.set_message("Downloading package files...");
        let temp_dir = self.get_temp_dir(repo)?;
        let package_files = self
            .downloader
            .download_package(repo, &release, &temp_dir, target.as_ref())
            .await?;

        spinner.set_message("Installing package files...");
        let installed_files = self
            .install_package_files(repo, &package_files, target.as_ref())
            .await?;

        spinner.set_message("Computing package hash...");
        let combined_hash = self.compute_package_hash(&installed_files).await?;
        println!("Package hash (Argon2): {}", combined_hash);
        log::info!("Package {} hash: {}", repo, combined_hash);

        spinner.set_message("Updating cache...");
        for file_path in &installed_files {
            if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
                self.cache.store_hash(file_name, &combined_hash).await?;
            }
        }

        spinner.set_message("Updating lock file...");
        let file_names: Vec<SmolStr> = installed_files
            .iter()
            .filter_map(|p| p.to_str().map(|s| s.into()))
            .collect();

        lock.add_package(
            repo.into(),
            release.tag_name.clone().into(),
            target.clone(),
            combined_hash.into(),
            file_names,
        );
        lock.save_to_file(&self.lock_path).await?;

        spinner.set_message("Updating configuration...");
        self.update_config(repo, &release.tag_name, target).await?;

        spinner.set_message("Updating config.json...");
        self.config_manager
            .update_legacy_plugins(&self.lock_path)
            .await?;

        self.cleanup_temp_dir(&temp_dir).await?;

        spinner.finish_with_message(format!(
            "Successfully installed {} {}",
            repo, release.tag_name
        ));
        log::info!("Package installed: {} {}", repo, release.tag_name);

        Ok(())
    }

    pub async fn install_all_packages(&mut self) -> Result<()> {
        let config = BuildConfig::from_file(self.config_path.to_string_lossy().as_ref()).await?;

        if let Some(packages) = config.get_packages() {
            for (repo, spec) in packages {
                let version = spec.version();
                let target = spec.target().cloned();

                println!("Installing package: {} = {}", repo, version);
                if let Err(e) = self.install_package(repo, Some(version), target).await {
                    eprintln!("Failed to install {}: {}", repo, e);
                    log::error!("Package installation failed: {} - {}", repo, e);
                }
            }
        } else {
            println!("No packages defined in configuration");
        }

        Ok(())
    }

    pub async fn remove_package(&mut self, repo: &str) -> Result<()> {
        let spinner = self.create_spinner(format!("Removing package {}...", repo));

        spinner.set_message("Checking lock file...");
        let mut lock = PackageLock::load_from_file(&self.lock_path).await?;

        if !lock.is_package_installed(repo) {
            spinner.finish_with_message(format!("Package {} is not installed", repo));
            println!("Package {} is not installed", repo);
            return Ok(());
        }

        spinner.set_message("Removing package files...");
        let removed_package = lock.get_package(repo).cloned();
        if let Some(package) = &removed_package {
            self.remove_package_files_from_lock(&package.files).await?;
        }

        spinner.set_message("Updating cache...");
        if let Some(package) = &removed_package {
            for file_path in &package.files {
                if let Some(file_name) = std::path::Path::new(file_path.as_str())
                    .file_name()
                    .and_then(|n| n.to_str())
                {
                    let _ = self.cache.remove_hash(file_name).await;
                }
            }
        }

        spinner.set_message("Updating config.json...");
        if let Some(package) = &removed_package {
            if let Some(PackageTarget::Plugins) = &package.target {
                self.config_manager
                    .remove_legacy_plugin_advanced(repo, package)
                    .await?;
            }
        }

        spinner.set_message("Updating lock file...");
        lock.remove_package(repo);
        lock.save_to_file(&self.lock_path).await?;

        spinner.set_message("Updating configuration...");
        self.remove_from_config(repo).await?;

        spinner.finish_with_message(format!("Successfully removed {}", repo));
        log::info!("Package removed: {}", repo);

        Ok(())
    }

    pub async fn list_packages(&self) -> Result<()> {
        let lock = PackageLock::load_from_file(&self.lock_path).await?;

        let packages = lock.list_packages();
        if !packages.is_empty() {
            println!("Installed packages:");
            for (repo, package) in packages {
                let target_info = package
                    .target
                    .as_ref()
                    .map(|t| {
                        format!(
                            " ({})",
                            match t {
                                PackageTarget::Components => "components",
                                PackageTarget::Plugins => "plugins",
                            }
                        )
                    })
                    .unwrap_or_default();

                println!("  {} = {}{}", repo, package.version, target_info);
                println!("    Installed: {}", package.installed_at);
                println!("    Hash: {}", &package.hash[..32]);
                println!("    Files: {}", package.files.len());
            }
        } else {
            println!("No packages installed");
        }

        Ok(())
    }

    pub async fn check_packages(&self) -> Result<()> {
        let lock = PackageLock::load_from_file(&self.lock_path).await?;
        let packages = lock.list_packages();

        if packages.is_empty() {
            println!("No packages to check");
            return Ok(());
        }

        println!("Checking package integrity...");
        let mut all_valid = true;

        for (repo, package) in packages {
            print!("Checking {} {}... ", repo, package.version);

            let mut files_exist = true;
            let mut valid_files = Vec::new();

            for file_path_str in &package.files {
                let file_path = std::path::Path::new(file_path_str.as_str());
                if file_path.exists() {
                    valid_files.push(file_path.to_path_buf());
                } else {
                    files_exist = false;
                    break;
                }
            }

            if !files_exist {
                println!("Missing files");
                all_valid = false;
                continue;
            }

            match self.compute_package_hash(&valid_files).await {
                Ok(computed_hash) => {
                    if computed_hash == package.hash.as_str() {
                        println!("Valid");
                    } else {
                        println!("Hash mismatch");
                        all_valid = false;
                    }
                }
                Err(_) => {
                    println!("Hash computation failed");
                    all_valid = false;
                }
            }
        }

        if all_valid {
            println!("\nAll packages are valid");
        } else {
            println!("\nSome packages have issues");
            println!("Run 'opencli package install' to reinstall packages");
        }

        Ok(())
    }

    pub async fn update_package(&mut self, repo: &str) -> Result<()> {
        let config = BuildConfig::from_file(self.config_path.to_string_lossy().as_ref()).await?;
        let lock = PackageLock::load_from_file(&self.lock_path).await?;

        if let Some(packages) = config.get_packages() {
            if let Some(spec) = packages.get(repo) {
                let _constraint = VersionConstraint::parse(spec.version())?;
                let target = spec.target().cloned();

                if let Some(package) = lock.get_package(repo) {
                    self.remove_package_files_from_lock(&package.files).await?;
                }
                self.install_package(repo, Some(spec.version()), target)
                    .await?;
            } else {
                return Err(OpenCliError::NotFound(
                    format!("Package {} not found", repo).into(),
                ));
            }
        } else {
            return Err(OpenCliError::NotFound("No packages installed".into()));
        }

        Ok(())
    }

    async fn install_package_files(
        &self,
        _repo: &str,
        package_files: &crate::package::downloader::PackageFiles,
        target: Option<&PackageTarget>,
    ) -> Result<Vec<PathBuf>> {
        self.workspace.ensure_workspace_structure().await?;

        let mut installed_files = Vec::new();
        let include_paths = self.get_include_paths().await?;
        let workspace_info = self.workspace.get_workspace_info();

        for include_file in &package_files.includes {
            if let Some(include_path) = include_paths.first() {
                let dest_path = include_path.join(include_file.file_name().unwrap());
                fs::copy(include_file, &dest_path).await?;
                installed_files.push(dest_path.clone());
                log::info!(
                    "Copied include: {} -> {}",
                    include_file.display(),
                    dest_path.display()
                );
            }
        }

        for binary_file in &package_files.root_binaries {
            let dest_path = workspace_info.root.join(binary_file.file_name().unwrap());
            fs::copy(binary_file, &dest_path).await?;
            installed_files.push(dest_path.clone());
            log::info!(
                "Copied root binary: {} -> {}",
                binary_file.display(),
                dest_path.display()
            );
        }

        let component_files = if !package_files.component_binaries.is_empty() {
            &package_files.component_binaries
        } else {
            &package_files.binaries
        };

        let plugin_files = if !package_files.plugin_binaries.is_empty() {
            &package_files.plugin_binaries
        } else {
            &package_files.binaries
        };

        match target {
            Some(PackageTarget::Components) => {
                for binary_file in component_files {
                    let dest_path = workspace_info
                        .components
                        .join(binary_file.file_name().unwrap());
                    fs::copy(binary_file, &dest_path).await?;
                    installed_files.push(dest_path.clone());
                    log::info!(
                        "Copied component binary: {} -> {}",
                        binary_file.display(),
                        dest_path.display()
                    );
                }
            }
            Some(PackageTarget::Plugins) => {
                for binary_file in plugin_files {
                    let dest_path = workspace_info
                        .plugins
                        .join(binary_file.file_name().unwrap());
                    fs::copy(binary_file, &dest_path).await?;
                    installed_files.push(dest_path.clone());
                    log::info!(
                        "Copied plugin binary: {} -> {}",
                        binary_file.display(),
                        dest_path.display()
                    );
                }
            }
            None => {
                for binary_file in component_files {
                    let target_folder = self.detect_binary_target(binary_file).await?;
                    let dest_path = target_folder.join(binary_file.file_name().unwrap());
                    fs::copy(binary_file, &dest_path).await?;
                    installed_files.push(dest_path.clone());
                    log::info!(
                        "Copied auto-detected binary: {} -> {}",
                        binary_file.display(),
                        dest_path.display()
                    );
                }
            }
        }

        installed_files.sort();
        installed_files.dedup();
        Ok(installed_files)
    }

    async fn compute_package_hash(&self, installed_files: &[PathBuf]) -> Result<String> {
        use sha2::{Digest, Sha256};

        let mut combined_content = Vec::new();

        for file_path in installed_files {
            if file_path.exists() {
                let content = fs::read(file_path).await?;
                combined_content.extend_from_slice(&content);
                combined_content.extend_from_slice(file_path.to_string_lossy().as_bytes());
            }
        }

        let mut hasher = Sha256::new();
        hasher.update(&combined_content);
        let combined_sha = hasher.finalize();

        let argon2_hash = self.security.hash_file_content(&combined_sha).await?;
        Ok(argon2_hash)
    }

    async fn remove_package_files_from_lock(&self, files: &[smol_str::SmolStr]) -> Result<()> {
        for file_path in files {
            let path = std::path::Path::new(file_path.as_str());
            if path.exists() {
                fs::remove_file(path).await.map_err(|e| {
                    log::warn!("Failed to remove file {}: {}", path.display(), e);
                    e
                })?;
                log::info!("Removed file: {}", path.display());
                println!("Removed: {}", path.display());
            } else {
                log::warn!("File not found (already removed?): {}", path.display());
            }
        }
        Ok(())
    }

    async fn update_config(
        &self,
        repo: &str,
        version: &str,
        target: Option<PackageTarget>,
    ) -> Result<()> {
        let mut config =
            BuildConfig::from_file(self.config_path.to_string_lossy().as_ref()).await?;

        let spec = if let Some(target) = target {
            PackageSpec::new_detailed(version, Some(target))
        } else {
            PackageSpec::new_simple(version)
        };

        config.add_package(repo.into(), spec);
        config
            .save_to_file(self.config_path.to_string_lossy().as_ref())
            .await?;

        Ok(())
    }

    async fn remove_from_config(&self, repo: &str) -> Result<()> {
        let mut config =
            BuildConfig::from_file(self.config_path.to_string_lossy().as_ref()).await?;

        if config.remove_package(repo) {
            config
                .save_to_file(self.config_path.to_string_lossy().as_ref())
                .await?;
        }

        Ok(())
    }

    async fn get_include_paths(&self) -> Result<Vec<PathBuf>> {
        let config = BuildConfig::from_file(self.config_path.to_string_lossy().as_ref()).await?;
        Ok(config.get_include_paths())
    }

    async fn detect_binary_target(&self, binary_path: &Path) -> Result<PathBuf> {
        if let Some(file_name) = binary_path.file_name().and_then(|n| n.to_str()) {
            let file_name_lower = file_name.to_lowercase();

            if file_name_lower.contains("omp") || file_name_lower.contains("component") {
                self.workspace.detect_components_folder().await
            } else {
                self.workspace.detect_plugins_folder().await
            }
        } else {
            self.workspace.detect_plugins_folder().await
        }
    }

    fn get_temp_dir(&self, repo: &str) -> Result<PathBuf> {
        let temp_name = repo.replace('/', "_");
        let temp_dir = std::env::temp_dir()
            .join("opencli")
            .join("packages")
            .join(temp_name);
        Ok(temp_dir)
    }

    async fn cleanup_temp_dir(&self, temp_dir: &Path) -> Result<()> {
        if temp_dir.exists() {
            fs::remove_dir_all(temp_dir).await?;
        }
        Ok(())
    }

    fn create_spinner(&self, message: impl Into<String>) -> ProgressBar {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        spinner.set_message(message.into());
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));
        spinner
    }
}
