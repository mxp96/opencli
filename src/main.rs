use clap::Parser;
use dirs::config_dir;
use env_logger::Builder;
use log::LevelFilter;
use opencli::cli::Cli;
use opencli::result::Result;
use std::fs::OpenOptions;

/** Main entry point for the OpenCLI application
 *
 * # Process Flow
 * 1. Initialize logging system with file output
 * 2. Parse command line arguments using Clap
 * 3. Execute the requested command
 * 4. Handle errors and exit with appropriate codes
 *
 * # Error Handling
 * - Logging failures are non-fatal (fallback to creation)
 * - Clap parsing errors are displayed and exit with proper codes
 * - Command execution errors are propagated up and logged
 *
 * # Example
 * ```bash
 * # Run with default logging
 * opencli --help
 *
 * # Execute specific command
 * opencli hash document.pdf
 * ```
 */
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Initialize logging before any other operations
    init_logging().await;

    // Parse command line arguments with error handling
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // Print clap error message to stderr
            e.print().expect("Failed to print clap error");
            std::process::exit(e.exit_code());
        }
    };

    // Execute the parsed command
    cli.execute().await
}

/** Initializes the logging system with file-based output
 *
 * # Configuration
 * - Log file location: platform-specific config directory
 * - Log level: Info and above
 * - Output: Append mode to preserve historical logs
 * - Fallback: Current directory if config directory unavailable
 *
 * # Directory Structure
 * - Linux: `~/.config/opencli/opencli.log`
 * - macOS: `~/Library/Application Support/opencli/opencli.log`  
 * - Windows: `%APPDATA%\opencli\opencli.log`
 *
 * # Notes
 * - Creates directory structure if it doesn't exist
 * - Falls back to current directory if config directory inaccessible
 * - Uses async-compatible logging initialization
 */
async fn init_logging() {
    let log_file = get_log_file_path();

    // Ensure log directory exists
    if let Some(parent) = log_file.parent() {
        std::fs::create_dir_all(parent).ok(); // Non-fatal if directory creation fails
    }

    // Configure and initialize the logger
    Builder::from_default_env()
        .target(env_logger::Target::Pipe(Box::new(
            OpenOptions::new()
                .create(true) // Create file if it doesn't exist
                .append(true) // Append to existing logs
                .open(&log_file)
                .unwrap_or_else(|_| {
                    // Fallback: create new file if open fails
                    std::fs::File::create(&log_file).expect("Failed to create log file")
                }),
        )))
        .filter_level(LevelFilter::Info) // Log info level and above
        .init();

    log::info!("OpenCLI started");
}

/** Determines the appropriate log file path based on platform
 *
 * # Returns
 * - Platform-specific config directory path when available
 * - Current working directory as fallback
 * - Direct filename as last resort
 *
 * # Platform Support
 * - **Linux**: Follows XDG Base Directory specification
 * - **macOS**: Uses Application Support directory
 * - **Windows**: Uses AppData/Roaming directory
 * - **Fallback**: Current working directory
 *
 * # Notes
 * - Respects platform conventions for configuration files
 * - Gracefully handles missing config directories
 */
fn get_log_file_path() -> std::path::PathBuf {
    if let Some(config_dir) = config_dir() {
        // Use platform-specific config directory
        config_dir.join("opencli").join("opencli.log")
    } else {
        // Fallback to current directory
        std::env::current_dir()
            .map(|p| p.join("opencli.log"))
            .unwrap_or_else(|_| "opencli.log".into())
    }
}

/*
 * Performance and Design Considerations:
 *
 * 1. Async Runtime:
 *    - Uses `current_thread` flavor for lightweight operations
 *    - Suitable for I/O-bound tasks without parallel CPU work
 *    - Lower memory overhead compared to multi-threaded runtime
 *
 * 2. Error Handling:
 *    - Non-fatal logging initialization failures
 *    - Proper exit codes for command line errors
 *    - Graceful fallbacks for filesystem operations
 *
 * 3. Logging Strategy:
 *    - File-based logging for persistence
 *    - Append mode to preserve history across runs
 *    - Platform-appropriate directory structure
 *
 * 4. Maintenance:
 *    - Centralized logging configuration
 *    - Clear separation of concerns
 *    - Easy to modify log levels or destinations
 */
