use crate::result::{OpenCliError, Result};
use std::path::Path;
use tokio::fs;

pub async fn execute(force: bool) -> Result<()> {
    let mut cmd = SetupCommand::new();
    cmd.execute(force).await
}

#[derive(Default)]
pub struct SetupCommand;

impl SetupCommand {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&mut self, force: bool) -> Result<()> {
        let config_file = "opencli.toml";

        log::info!("Starting setup process with force: {}", force);

        if Path::new(config_file).exists() && !force {
            log::warn!("opencli.toml already exists, setup aborted");
            return Err(OpenCliError::Config(
                "opencli.toml already exists. Use --force to overwrite."
                    .to_string()
                    .into(),
            ));
        }

        println!("Downloading opencli.toml from GitHub Gist...");
        log::info!("Downloading opencli.toml from remote source");

        let content = self.download_config().await?;

        fs::write(config_file, content).await?;

        println!("opencli.toml created successfully!");
        println!();
        println!("Please edit opencli.toml to match your project:");
        println!("   - Update entry_file to your main .pwn file");
        println!("   - Update output_file to your desired .amx file");
        println!("   - Adjust include paths if needed");
        println!("   - Modify compiler arguments as required");
        println!();
        println!("Then run: opencli build");

        log::info!("Setup completed successfully");

        Ok(())
    }

    async fn download_config(&self) -> Result<String> {
        const CONFIG_URL: &str = "https://gist.githubusercontent.com/mxp96/82fd1b1b17ccb23a11fcbe40b83ceaa5/raw/opencli.toml";

        let client = reqwest::Client::new();
        let response = client
            .get(CONFIG_URL)
            .header("User-Agent", "opencli/0.1.0")
            .send()
            .await
            .map_err(|e| {
                OpenCliError::Process(format!("Failed to download config: {}", e).into())
            })?;

        if !response.status().is_success() {
            return Err(OpenCliError::Process(
                format!("Failed to download config: HTTP {}", response.status()).into(),
            ));
        }

        let content = response
            .text()
            .await
            .map_err(|e| OpenCliError::Process(format!("Failed to read config: {}", e).into()))?;

        Ok(content)
    }
}
