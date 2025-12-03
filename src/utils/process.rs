use crate::result::{OpenCliError, Result};
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use which::which;

#[derive(Default)]
pub struct ProcessManager;

impl ProcessManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn find_executable(&self, name: &str) -> Result<String> {
        match which(name) {
            Ok(path) => Ok(path.to_string_lossy().to_string()),
            Err(_) => Err(OpenCliError::NotFound(
                format!("Executable not found: {}", name).into(),
            )),
        }
    }

    pub async fn exec_server(
        &mut self,
        args: Vec<String>,
        server_path: Option<String>,
    ) -> Result<()> {
        let executable = if let Some(custom_path) = server_path {
            if !Path::new(&custom_path).exists() {
                return Err(OpenCliError::NotFound(
                    format!("Custom server path not found: {}", custom_path).into(),
                ));
            }
            custom_path
        } else {
            let current_dir = std::env::current_dir().map_err(|e| {
                OpenCliError::Process(format!("Failed to get current directory: {}", e).into())
            })?;

            let server_binaries = if cfg!(windows) {
                vec![
                    "omp-server.exe",
                    "./omp-server.exe",
                    ".\\omp-server.exe",
                    "omp-server",
                ]
            } else {
                vec!["omp-server", "./omp-server", "omp-server.exe"]
            };

            let mut found_executable = None;

            for binary in &server_binaries {
                let full_path = if binary.starts_with("./") || binary.starts_with(".\\") {
                    current_dir.join(&binary[2..])
                } else {
                    current_dir.join(binary)
                };

                if full_path.exists() {
                    found_executable = Some(full_path.to_string_lossy().to_string());
                    break;
                }

                if Path::new(binary).exists() {
                    found_executable = Some(binary.to_string());
                    break;
                }
            }

            if found_executable.is_none() {
                for binary in &server_binaries {
                    if let Ok(path) = which(binary) {
                        found_executable = Some(path.to_string_lossy().to_string());
                        break;
                    }
                }
            }

            found_executable.ok_or_else(|| {
                OpenCliError::NotFound(
                    format!(
                        "omp-server executable not found.\nLooked for: {:?}\nCurrent directory: {}",
                        server_binaries,
                        current_dir.display()
                    )
                    .into(),
                )
            })?
        };

        let mut command = Command::new(&executable);
        command.args(args);
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());
        command.stdin(Stdio::inherit());

        let mut child = command
            .spawn()
            .map_err(|e| OpenCliError::Process(format!("Failed to start server: {}", e).into()))?;

        let status = child.wait().await.map_err(|e| {
            OpenCliError::Process(format!("Failed to wait for server: {}", e).into())
        })?;

        if !status.success() {
            if let Some(code) = status.code() {
                std::process::exit(code);
            } else {
                std::process::exit(1);
            }
        }

        Ok(())
    }
}
