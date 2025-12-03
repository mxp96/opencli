use crate::build::BuildConfig;
use crate::compiler::CompilerManager;
use crate::result::{OpenCliError, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;

pub async fn execute(
    config_path: Option<&str>,
    verbose: bool,
    force_download: bool,
    update_config: bool,
) -> Result<()> {
    let mut cmd = BuildCommand::new();
    cmd.execute(
        config_path.map(|s| s.to_string()),
        verbose,
        force_download,
        update_config,
    )
    .await
}

#[derive(Default)]
pub struct BuildCommand;

impl BuildCommand {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(
        &mut self,
        config_path: Option<String>,
        verbose: bool,
        force_download: bool,
        update_config: bool,
    ) -> Result<()> {
        println!("Building project...");

        let build_spinner = ProgressBar::new_spinner();
        build_spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        build_spinner.set_message("Loading build configuration...");
        build_spinner.enable_steady_tick(std::time::Duration::from_millis(100));

        let config = self.load_build_config(config_path).await?;

        log::info!(
            "Starting build process for entry file: {}",
            config.build.entry_file.display()
        );

        if verbose {
            build_spinner.finish_and_clear();
            println!("Build configuration:");
            println!("  Entry file: {}", config.build.entry_file.display());
            println!("  Output file: {}", config.build.output_file.display());
            println!("  Compiler version: {}", config.build.compiler_version);

            build_spinner.set_message("Preparing compiler...");
            build_spinner.enable_steady_tick(std::time::Duration::from_millis(100));
        } else {
            build_spinner.set_message("Preparing compiler...");
        }

        let mut compiler_manager = if update_config {
            CompilerManager::new_with_update().await?
        } else {
            CompilerManager::new().await?
        };

        let compiler_path = compiler_manager
            .get_compiler_path(&config.build.compiler_version, force_download)
            .await?;

        if verbose {
            build_spinner.finish_and_clear();
            println!("Using compiler: {}", compiler_path.display());
        }

        build_spinner.set_message("Compiling project...");
        log::info!("Using compiler: {}", compiler_path.display());

        let result = self.compile_project(&config, &compiler_path, verbose).await;
        build_spinner.finish_and_clear();

        result
    }

    async fn load_build_config(&self, config_path: Option<String>) -> Result<BuildConfig> {
        let config_file = config_path.unwrap_or_else(|| "opencli.toml".to_string());

        if !Path::new(&config_file).exists() {
            return Err(OpenCliError::NotFound(
                format!(
                    "Configuration file '{}' not found. Run 'opencli setup' to create it.",
                    config_file
                )
                .into(),
            ));
        }

        BuildConfig::from_file(&config_file).await
    }

    async fn compile_project(
        &self,
        config: &BuildConfig,
        compiler_path: &Path,
        verbose: bool,
    ) -> Result<()> {
        let current_dir = std::env::current_dir().map_err(|e| {
            OpenCliError::Process(format!("Failed to get current directory: {}", e).into())
        })?;

        let entry_path = current_dir.join(&config.build.entry_file);
        if !entry_path.exists() {
            return Err(OpenCliError::NotFound(
                format!("Entry file not found: {}", entry_path.display()).into(),
            ));
        }

        let output_path = current_dir.join(&config.build.output_file);
        if let Some(output_dir) = output_path.parent() {
            tokio::fs::create_dir_all(output_dir).await?;
        }

        let compile_start = Instant::now();
        let mut cmd = Command::new(compiler_path);
        cmd.current_dir(&current_dir);

        if let Some(compiler_dir) = Path::new(compiler_path).parent() {
            let mut ld_path = compiler_dir.to_string_lossy().to_string();
            if let Ok(existing_ld) = std::env::var("LD_LIBRARY_PATH") {
                ld_path = format!("{}:{}", ld_path, existing_ld);
            }
            cmd.env("LD_LIBRARY_PATH", ld_path);
            log::debug!("Set LD_LIBRARY_PATH to: {}", compiler_dir.display());
        }

        let output_arg = format!("-o{}", config.build.output_file.display());
        cmd.arg(&output_arg);

        if let Some(includes) = &config.build.includes {
            for include_path in &includes.paths {
                let full_include_path = current_dir.join(include_path);
                if full_include_path.exists() {
                    let include_arg = format!("-i{}", full_include_path.display());
                    cmd.arg(&include_arg);
                }
            }
        }

        let mut has_debug_flags = false;
        let mut processed_args = Vec::new();

        if let Some(args) = &config.build.args {
            for arg in &args.args {
                if arg == "-d2" || arg == "-d3" {
                    has_debug_flags = true;
                    processed_args.push(arg.clone());
                } else if arg.starts_with("-O") && has_debug_flags {
                    continue;
                } else {
                    processed_args.push(arg.clone());
                }
            }
        }

        for arg in processed_args {
            cmd.arg(&arg);
        }

        cmd.arg(&config.build.entry_file);

        if verbose || has_debug_flags {
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());

            let mut child = cmd.spawn().map_err(|e| {
                OpenCliError::Process(format!("Failed to execute compiler: {}", e).into())
            })?;

            let status = child.wait().await.map_err(|e| {
                OpenCliError::Process(format!("Failed to wait for compiler: {}", e).into())
            })?;

            if status.success() {
                let compile_duration = compile_start.elapsed();
                let time_str = format_duration(compile_duration);
                println!(
                    "Build successful: {} ({})",
                    config.build.output_file.display(),
                    time_str
                );
                log::info!(
                    "Build completed successfully: {} in {}",
                    config.build.output_file.display(),
                    time_str
                );
            } else {
                return Err(OpenCliError::Process(
                    format!(
                        "Build failed with exit code: {}",
                        status.code().unwrap_or(-1)
                    )
                    .into(),
                ));
            }
        } else {
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::piped());

            let output = cmd.output().await.map_err(|e| {
                OpenCliError::Process(format!("Failed to execute compiler: {}", e).into())
            })?;

            if output.status.success() {
                let compile_duration = compile_start.elapsed();
                let time_str = format_duration(compile_duration);
                println!(
                    "Build successful: {} ({})",
                    config.build.output_file.display(),
                    time_str
                );
                log::info!(
                    "Build completed successfully: {} in {}",
                    config.build.output_file.display(),
                    time_str
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !stderr.is_empty() {
                    eprintln!("Compiler stderr:\n{}", stderr);
                }

                log::error!("Build failed with stderr: {}", stderr);

                return Err(OpenCliError::Process(
                    format!(
                        "Build failed with exit code: {}",
                        output.status.code().unwrap_or(-1)
                    )
                    .into(),
                ));
            }
        }

        Ok(())
    }
}

fn format_duration(duration: std::time::Duration) -> String {
    let total_ms = duration.as_millis();

    if total_ms >= 1000 {
        let seconds = duration.as_secs_f64();
        format!("{:.2}s", seconds)
    } else {
        format!("{}ms", total_ms)
    }
}
