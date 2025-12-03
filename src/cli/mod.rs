pub mod parser;

use crate::commands::CommandExecutor;
use crate::result::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "opencli")]
#[command(about = "CLI tool for open.mp server management")]
#[command(version = "0.1.0")]
#[command(author = "Matthias Theodore \"mxp96\" Bartholomew")]
#[command(arg_required_else_help = true)]
#[command(
    help_template = "{before-help}{name} v{version}\nAuthor: {author}\n\n{about-with-newline}\n{usage-heading} {usage}\n\n{all-args}{after-help}"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
pub enum Commands {
    #[command(about = "Run open.mp server")]
    Run {
        #[arg(long, help = "Custom path to omp-server executable")]
        server_path: Option<String>,
    },

    #[command(about = "Build open.mp project")]
    Build {
        #[arg(short, long, help = "Build configuration file")]
        config: Option<String>,

        #[arg(short, long, help = "Enable verbose output")]
        verbose: bool,

        #[arg(long, help = "Force compiler redownload")]
        force_download: bool,

        #[arg(long, help = "Update compiler configuration from remote")]
        update_config: bool,
    },

    #[command(about = "Setup project with default opencli.toml")]
    Setup {
        #[arg(long, help = "Force overwrite existing opencli.toml")]
        force: bool,
    },

    #[command(about = "Install components")]
    Install {
        #[command(subcommand)]
        component: InstallComponent,
    },

    #[command(about = "Package management commands")]
    Package {
        #[command(subcommand)]
        action: PackageAction,
    },
}

#[derive(Parser)]
pub enum InstallComponent {
    #[command(about = "Install Pawn compiler")]
    Compiler {
        #[arg(long, help = "Compiler version to install (default: v3.10.11)")]
        version: Option<String>,

        #[arg(long, help = "Force reinstall even if already exists")]
        force: bool,
    },
}

#[derive(Parser)]
pub enum PackageAction {
    #[command(about = "Install packages")]
    Install {
        #[arg(help = "Package to install (owner/repo or owner/repo=version)")]
        package: Option<String>,

        #[arg(long, help = "Target folder (components or plugins)")]
        target: Option<String>,
    },

    #[command(about = "Remove package")]
    Remove {
        #[arg(help = "Package to remove (owner/repo)")]
        package: String,
    },

    #[command(about = "List installed packages")]
    List,

    #[command(about = "Check package integrity")]
    Check,

    #[command(about = "Update package")]
    Update {
        #[arg(help = "Package to update (owner/repo)")]
        package: Option<String>,

        #[arg(long, help = "Update all packages")]
        all: bool,
    },
}

impl Default for Cli {
    fn default() -> Self {
        Self::parse()
    }
}

impl Cli {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn execute(self) -> Result<()> {
        let mut executor = CommandExecutor::new();

        match self.command {
            Commands::Run { server_path } => executor.run_server(server_path).await,
            Commands::Build {
                config,
                verbose,
                force_download,
                update_config,
            } => {
                executor
                    .build_project(config, verbose, force_download, update_config)
                    .await
            }
            Commands::Setup { force } => executor.setup_project(force).await,
            Commands::Install { component } => match component {
                InstallComponent::Compiler { version, force } => {
                    executor.install_compiler(version, force).await
                }
            },
            Commands::Package { action } => executor.handle_package_action(action).await,
        }
    }
}
