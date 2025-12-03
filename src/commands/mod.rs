pub mod build;
pub mod install;
pub mod run;
pub mod setup;

use crate::cli::PackageAction;
use crate::result::Result;
use smol_str::SmolStr;

#[derive(Debug)]
pub enum CommandType {
    Run {
        server_path: Option<SmolStr>,
    },
    Build {
        config: Option<SmolStr>,
        verbose: bool,
        force_download: bool,
        update_config: bool,
    },
    Setup {
        force: bool,
    },
    InstallCompiler {
        version: Option<SmolStr>,
        force: bool,
    },
}

impl CommandType {
    pub async fn execute(self) -> Result<()> {
        match self {
            CommandType::Run { server_path } => run::execute(server_path.as_deref()).await,
            CommandType::Build {
                config,
                verbose,
                force_download,
                update_config,
            } => build::execute(config.as_deref(), verbose, force_download, update_config).await,
            CommandType::Setup { force } => setup::execute(force).await,
            CommandType::InstallCompiler { version, force } => {
                install::execute_compiler(version.as_deref(), force).await
            }
        }
    }
}

#[derive(Default)]
pub struct CommandExecutor;

impl CommandExecutor {
    pub fn new() -> Self {
        Self
    }

    pub async fn run_server(&mut self, server_path: Option<String>) -> Result<()> {
        CommandType::Run {
            server_path: server_path.map(|s| s.into()),
        }
        .execute()
        .await
    }

    pub async fn build_project(
        &mut self,
        config: Option<String>,
        verbose: bool,
        force_download: bool,
        update_config: bool,
    ) -> Result<()> {
        CommandType::Build {
            config: config.map(|s| s.into()),
            verbose,
            force_download,
            update_config,
        }
        .execute()
        .await
    }

    pub async fn setup_project(&mut self, force: bool) -> Result<()> {
        CommandType::Setup { force }.execute().await
    }

    pub async fn install_compiler(&mut self, version: Option<String>, force: bool) -> Result<()> {
        CommandType::InstallCompiler {
            version: version.map(|s| s.into()),
            force,
        }
        .execute()
        .await
    }

    pub async fn handle_package_action(&mut self, action: PackageAction) -> Result<()> {
        use crate::build::PackageTarget;
        use crate::package::PackageManager;

        let workspace_root = std::env::current_dir()?;
        let config_path = workspace_root.join("opencli.toml");
        let mut manager = PackageManager::new(&workspace_root, &config_path);

        match action {
            PackageAction::Install { package, target } => {
                if let Some(package_spec) = package {
                    let (repo, version) = if let Some(pos) = package_spec.find('=') {
                        let repo_part = &package_spec[..pos];
                        let version_part = &package_spec[pos + 1..];
                        let clean_version = version_part.trim_matches('"').trim_matches('\'');
                        (repo_part, Some(clean_version))
                    } else {
                        (package_spec.as_str(), None)
                    };

                    let target_type =
                        target
                            .as_deref()
                            .and_then(|t| match t.to_lowercase().as_str() {
                                "components" => Some(PackageTarget::Components),
                                "plugins" => Some(PackageTarget::Plugins),
                                _ => None,
                            });

                    manager.install_package(repo, version, target_type).await
                } else {
                    manager.install_all_packages().await
                }
            }
            PackageAction::Remove { package } => manager.remove_package(&package).await,
            PackageAction::List => manager.list_packages().await,
            PackageAction::Check => manager.check_packages().await,
            PackageAction::Update { package, all } => {
                if all {
                    manager.install_all_packages().await
                } else if let Some(repo) = package {
                    manager.update_package(&repo).await
                } else {
                    println!("Specify a package to update or use --all");
                    Ok(())
                }
            }
        }
    }
}
