use crate::result::{OpenCliError, Result};
use indicatif::{ProgressBar, ProgressStyle};
use octocrab::Octocrab;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

static REGEX_CACHE: Lazy<HashMap<&'static str, Regex>> = Lazy::new(|| {
    let mut cache = HashMap::new();
    cache.insert("pawnc", Regex::new(r"pawnc-.+").unwrap());
    cache.insert("windows", Regex::new(r"windows").unwrap());
    cache.insert("linux", Regex::new(r"linux").unwrap());
    cache.insert("darwin", Regex::new(r"darwin|macos").unwrap());
    cache
});

pub struct CompilerDownloader {
    github: std::sync::Arc<Octocrab>,
    client: Client,
}

impl Default for CompilerDownloader {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilerDownloader {
    pub fn new() -> Self {
        let client = Client::new();

        let github = if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            if !token.is_empty() {
                let crab = Octocrab::builder()
                    .personal_token(token)
                    .build()
                    .unwrap_or_else(|_| Octocrab::default());
                octocrab::initialise(crab);
                octocrab::instance()
            } else {
                octocrab::instance()
            }
        } else {
            octocrab::instance()
        };

        Self { github, client }
    }

    pub async fn get_release_assets(&self, version: &str) -> Result<Vec<GitHubAsset>> {
        let (owner, repo) = if version == "v3.10.11" {
            ("openmultiplayer", "compiler")
        } else {
            ("pawn-lang", "compiler")
        };

        let release = self
            .github
            .repos(owner, repo)
            .releases()
            .get_by_tag(version)
            .await
            .map_err(|e| match e {
                octocrab::Error::GitHub { source, .. } if source.status_code.as_u16() == 404 => {
                    OpenCliError::NotFound(format!("Release {} not found", version).into())
                }
                _ => OpenCliError::Process(format!("Failed to fetch release info: {}", e).into()),
            })?;

        let assets = release
            .assets
            .into_iter()
            .map(|asset| GitHubAsset {
                name: asset.name,
                download_url: asset.browser_download_url.to_string(),
            })
            .collect();

        Ok(assets)
    }

    pub async fn find_matching_asset<'a>(
        &self,
        assets: &'a [GitHubAsset],
        pattern: &str,
    ) -> Result<&'a GitHubAsset> {
        let regex = if let Some(cached_regex) = REGEX_CACHE.get(pattern) {
            cached_regex
        } else {
            &Regex::new(pattern)
                .map_err(|e| OpenCliError::Config(format!("Invalid regex pattern: {}", e).into()))?
        };

        assets
            .iter()
            .find(|asset| regex.is_match(&asset.name))
            .ok_or_else(|| {
                OpenCliError::NotFound(format!("No asset matches pattern: {}", pattern).into())
            })
    }

    pub async fn download_asset(&self, asset: &GitHubAsset, output_path: &Path) -> Result<()> {
        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let response = self
            .client
            .get(&asset.download_url)
            .header("User-Agent", "opencli/0.1.0")
            .send()
            .await
            .map_err(|e| {
                OpenCliError::Process(format!("Failed to download asset: {}", e).into())
            })?;

        if !response.status().is_success() {
            return Err(OpenCliError::Process(
                format!("Download failed with status: {}", response.status()).into(),
            ));
        }

        let total_size = response.content_length();

        let pb = if let Some(size) = total_size {
            let pb = ProgressBar::new(size);
            pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"));
            pb.set_message("Downloading compiler");
            pb
        } else {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template(
                        "{spinner:.green} [{elapsed_precise}] Downloading compiler... {bytes}",
                    )
                    .unwrap(),
            );
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            pb
        };

        let bytes = response
            .bytes()
            .await
            .map_err(|e| OpenCliError::Process(format!("Download failed: {}", e).into()))?;

        if total_size.is_some() {
            pb.set_position(bytes.len() as u64);
        } else {
            pb.inc(bytes.len() as u64);
        }

        let mut file = File::create(output_path).await?;
        file.write_all(&bytes).await?;
        file.flush().await?;

        pb.finish_with_message(format!("Download complete ({} bytes)", bytes.len()));
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct GitHubAsset {
    pub name: String,
    pub download_url: String,
}
