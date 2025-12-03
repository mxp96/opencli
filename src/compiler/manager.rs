use crate::cache::CacheManager;
use crate::compiler::{CompilerConfig, CompilerDownloader, PlatformConfig};
use crate::result::{OpenCliError, Result};
use crate::security::SecurityManager;
use dirs::config_dir;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use std::path::{Path, PathBuf};
use tokio::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub struct CompilerManager {
    config: CompilerConfig,
    downloader: CompilerDownloader,
    base_dir: PathBuf,
    security: SecurityManager,
    cache: CacheManager,
}

impl CompilerManager {
    pub async fn new() -> Result<Self> {
        let base_dir = Self::get_base_directory()?;
        let config_path = base_dir.join("compilers.toml");

        fs::create_dir_all(&base_dir).await?;

        let config = if config_path.exists() {
            CompilerConfig::from_file(&config_path.to_string_lossy()).await?
        } else {
            Self::download_compilers_config(&config_path).await?
        };

        Ok(Self {
            config,
            downloader: CompilerDownloader::new(),
            base_dir: base_dir.clone(),
            security: SecurityManager::new(),
            cache: CacheManager::new(&base_dir),
        })
    }

    pub async fn new_with_update() -> Result<Self> {
        let base_dir = Self::get_base_directory()?;
        let config_path = base_dir.join("compilers.toml");

        fs::create_dir_all(&base_dir).await?;

        // Always download fresh config when update is requested
        let config = Self::download_compilers_config(&config_path).await?;

        Ok(Self {
            config,
            downloader: CompilerDownloader::new(),
            base_dir: base_dir.clone(),
            security: SecurityManager::new(),
            cache: CacheManager::new(&base_dir),
        })
    }

    fn get_base_directory() -> Result<PathBuf> {
        let config_dir = config_dir()
            .ok_or_else(|| OpenCliError::Config("Could not determine config directory".into()))?;

        Ok(config_dir.join("opencli"))
    }

    async fn download_compilers_config(config_path: &Path) -> Result<CompilerConfig> {
        const COMPILERS_CONFIG_URL: &str = "https://gist.githubusercontent.com/mxp96/798edeb8da39c7997948a9432d6f61bb/raw/compilers.toml";

        let client = reqwest::Client::new();
        let response = client
            .get(COMPILERS_CONFIG_URL)
            .header("User-Agent", "opencli/0.1.0")
            .send()
            .await
            .map_err(|e| {
                OpenCliError::Process(format!("Failed to download compilers config: {}", e).into())
            })?;

        if !response.status().is_success() {
            return Err(OpenCliError::Process(
                format!(
                    "Failed to download compilers config: HTTP {}",
                    response.status()
                )
                .into(),
            ));
        }

        let content = response.text().await.map_err(|e| {
            OpenCliError::Process(format!("Failed to read compilers config: {}", e).into())
        })?;

        fs::write(config_path, &content).await?;

        // Parse the config
        let config: CompilerConfig = toml::from_str(&content).map_err(|e| {
            OpenCliError::Config(format!("Invalid compilers config format: {}", e).into())
        })?;

        Ok(config)
    }

    pub async fn get_compiler_path(
        &mut self,
        version: &str,
        force_download: bool,
    ) -> Result<PathBuf> {
        let platform_config = self
            .config
            .get_platform_config()
            .ok_or_else(|| OpenCliError::Config("Unsupported platform".into()))?;

        let compiler_dir = self.base_dir.join("compilers").join(version);
        let binary_path = compiler_dir.join(&platform_config.binary);

        if binary_path.exists() && !force_download {
            if let Ok(Some(cached_hash)) = self.cache.get_hash(&platform_config.binary).await {
                match self.security.verify_file(&binary_path, &cached_hash).await {
                    Ok(true) => {
                        log::info!("Compiler verified successfully with cached hash");
                        return Ok(binary_path);
                    }
                    Ok(false) => {
                        log::warn!("Compiler hash verification failed, re-downloading");
                    }
                    Err(e) => {
                        log::error!("Hash verification error: {}", e);
                    }
                }
            }
        }

        self.download_and_install_compiler(version, platform_config)
            .await?;

        if binary_path.exists() {
            let security_spinner = ProgressBar::new_spinner();
            security_spinner.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.magenta} {msg}")
                    .unwrap(),
            );
            security_spinner.set_message("Generating security hash...");
            security_spinner.enable_steady_tick(std::time::Duration::from_millis(80));

            let file_hash = self.security.hash_file(&binary_path).await?;
            self.cache
                .store_hash(&platform_config.binary, &file_hash)
                .await?;

            security_spinner.finish_and_clear();

            println!("argon2:{}", file_hash);
            log::info!("Compiler installed with argon2:{}", file_hash);

            Ok(binary_path)
        } else {
            Err(OpenCliError::NotFound(
                format!(
                    "Compiler binary not found after installation: {}",
                    binary_path.display()
                )
                .into(),
            ))
        }
    }

    async fn download_and_install_compiler(
        &self,
        version: &str,
        platform_config: &PlatformConfig,
    ) -> Result<()> {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );

        spinner.set_message("Fetching release information...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));

        let assets = self.downloader.get_release_assets(version).await?;
        let matching_asset = self
            .downloader
            .find_matching_asset(&assets, &platform_config.match_pattern)
            .await?;

        spinner.set_message("Preparing download directory...");
        let temp_dir = self.base_dir.join("temp");
        fs::create_dir_all(&temp_dir).await?;

        spinner.finish_and_clear();

        println!("Downloading version {}", version);

        let downloaded_file = temp_dir.join(&matching_asset.name);
        self.downloader
            .download_asset(matching_asset, &downloaded_file)
            .await?;

        let extract_spinner = ProgressBar::new_spinner();
        extract_spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.yellow} {msg}")
                .unwrap(),
        );
        extract_spinner.set_message("Extracting compiler...");
        extract_spinner.enable_steady_tick(std::time::Duration::from_millis(100));

        let extract_dir = self.base_dir.join("compilers").join(version);
        fs::create_dir_all(&extract_dir).await?;

        match platform_config.method.as_str() {
            "zip" => self.extract_zip(&downloaded_file, &extract_dir).await?,
            "tgz" => self.extract_tgz(&downloaded_file, &extract_dir).await?,
            _ => {
                return Err(OpenCliError::Config(
                    format!("Unsupported extraction method: {}", platform_config.method).into(),
                ))
            }
        }

        extract_spinner.set_message("Organizing files...");
        self.organize_files(&extract_dir, platform_config).await?;

        extract_spinner.set_message("Cleaning up temporary files...");
        fs::remove_file(&downloaded_file).await?;

        extract_spinner.finish_and_clear();

        Ok(())
    }

    async fn extract_zip(&self, archive_path: &Path, extract_to: &Path) -> Result<()> {
        let file = std::fs::File::open(archive_path)
            .map_err(|e| OpenCliError::Process(format!("Failed to open zip file: {}", e).into()))?;

        let mut archive = zip::ZipArchive::new(file).map_err(|e| {
            OpenCliError::Process(format!("Failed to read zip archive: {}", e).into())
        })?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                OpenCliError::Process(format!("Failed to read zip entry: {}", e).into())
            })?;

            let outpath = extract_to.join(file.name());

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath).await?;
            } else {
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent).await?;
                }

                let mut buffer = Vec::new();
                std::io::copy(&mut file, &mut buffer).map_err(|e| {
                    OpenCliError::Process(format!("Failed to read file: {}", e).into())
                })?;
                tokio::fs::write(&outpath, buffer).await?;
            }
        }

        Ok(())
    }

    async fn extract_tgz(&self, archive_path: &Path, extract_to: &Path) -> Result<()> {
        let file = tokio::fs::File::open(archive_path).await?;
        let decoder = flate2::read::GzDecoder::new(file.into_std().await);
        let mut archive = tar::Archive::new(decoder);

        archive.unpack(extract_to).map_err(|e| {
            OpenCliError::Process(format!("Failed to extract tar.gz: {}", e).into())
        })?;

        Ok(())
    }

    async fn organize_files(
        &self,
        extract_dir: &Path,
        platform_config: &PlatformConfig,
    ) -> Result<()> {
        for (pattern, target) in &platform_config.paths {
            let regex = Regex::new(pattern)
                .map_err(|e| OpenCliError::Config(format!("Invalid path pattern: {}", e).into()))?;

            self.search_and_move_recursively(extract_dir, &regex, target)
                .await?;
        }

        self.cleanup_empty_dirs(extract_dir).await?;

        Ok(())
    }

    async fn search_and_move_recursively(
        &self,
        dir: &Path,
        regex: &Regex,
        target: &str,
    ) -> Result<bool> {
        fn visit_dir(
            dir: &Path,
            regex: &Regex,
            target: &str,
            base_dir: &Path,
        ) -> std::result::Result<bool, std::io::Error> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    if visit_dir(&path, regex, target, base_dir)? {
                        return Ok(true);
                    }
                } else if let Ok(relative_path) = path.strip_prefix(base_dir) {
                    let relative_str = relative_path.to_string_lossy().replace('\\', "/");

                    if regex.is_match(&relative_str) {
                        let target_path = base_dir.join(target);
                        if let Some(parent) = target_path.parent() {
                            std::fs::create_dir_all(parent)?;
                        }

                        std::fs::copy(&path, &target_path)?;

                        #[cfg(unix)]
                        {
                            let mut perms = std::fs::metadata(&target_path)?.permissions();
                            perms.set_mode(0o755);
                            std::fs::set_permissions(&target_path, perms)?;
                        }

                        std::fs::remove_file(&path)?;
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        }

        let found = visit_dir(dir, regex, target, dir)
            .map_err(|e| OpenCliError::Process(format!("Failed to search files: {}", e).into()))?;

        Ok(found)
    }

    async fn cleanup_empty_dirs(&self, dir: &Path) -> Result<()> {
        fn remove_empty_dirs_recursive(dir: &Path) -> std::result::Result<(), std::io::Error> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    remove_empty_dirs_recursive(&path)?;

                    if std::fs::read_dir(&path)?.next().is_none() {
                        std::fs::remove_dir(&path)?;
                    }
                }
            }
            Ok(())
        }

        remove_empty_dirs_recursive(dir).map_err(|e| {
            OpenCliError::Process(format!("Failed to cleanup directories: {}", e).into())
        })?;

        Ok(())
    }
}
