use crate::compiler::CompilerManager;
use crate::result::{OpenCliError, Result};
use indicatif::{ProgressBar, ProgressStyle};

pub async fn execute_compiler(version: Option<&str>, force: bool) -> Result<()> {
    let mut cmd = InstallCommand::new();
    cmd.execute_compiler(version.map(|s| s.to_string()), force)
        .await
}

#[derive(Default)]
pub struct InstallCommand;

impl InstallCommand {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute_compiler(&mut self, version: Option<String>, force: bool) -> Result<()> {
        let version = version.unwrap_or_else(|| "v3.10.11".to_string());

        println!("Installing Pawn compiler version: {}", version);
        log::info!("Starting compiler installation for version: {}", version);

        let install_spinner = ProgressBar::new_spinner();
        install_spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.blue} {msg}")
                .unwrap(),
        );
        install_spinner.set_message("Initializing compiler manager...");
        install_spinner.enable_steady_tick(std::time::Duration::from_millis(120));

        let mut manager = CompilerManager::new().await?;
        install_spinner.finish_and_clear();

        match manager.get_compiler_path(&version, force).await {
            Ok(path) => {
                println!("\nCompleted successfully!");
                println!("Compiler installed at: {}", path.display());
                println!("You can now use 'opencli build' to compile your Pawn projects.");
                log::info!(
                    "Compiler installation completed successfully: {}",
                    path.display()
                );
            }
            Err(e) => {
                println!("\nInstallation failed!");
                eprintln!("Error: {}", e);
                log::error!("Compiler installation failed: {}", e);
                return Err(OpenCliError::Process(
                    format!("Failed to install compiler: {}", e).into(),
                ));
            }
        }

        Ok(())
    }
}
