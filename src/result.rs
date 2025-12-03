use std::borrow::Cow;
use thiserror::Error;

/** Main Result type alias for OpenCLI operations
 *
 * # Usage
 * ```no_run
 * use opencli::result::Result;
 *
 * async fn read_config() -> Result<String> {
 *     // Function automatically propagates OpenCliError
 *     let content = std::fs::read_to_string("config.toml")?;
 *     Ok(toml::from_str(&content)?)
 * }
 * ```
 */
pub type Result<T> = std::result::Result<T, OpenCliError>;

/** Comprehensive error enumeration for OpenCLI application
 *
 * # Error Categories
 * - **Io**: File system and I/O operations
 * - **Process**: External process execution failures
 * - **Config**: Configuration parsing and validation errors
 * - **Server**: HTTP server and network-related issues
 * - **NotFound**: Resource missing errors
 * - **TomlParse**: TOML configuration parsing failures
 * - **TomlSerialize**: TOML serialization errors
 * - **JsonError**: JSON processing failures
 *
 * # Design Notes
 * - Uses `Cow<'static, str>` for efficient string storage
 * - Automatic From implementations for common error types
 * - Rich error messages with context information
 */
#[derive(Error, Debug)]
pub enum OpenCliError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Process error: {0}")]
    Process(Cow<'static, str>),

    #[error("Config error: {0}")]
    Config(Cow<'static, str>),

    #[error("Server error: {0}")]
    Server(Cow<'static, str>),

    #[error("Not found: {0}")]
    NotFound(Cow<'static, str>),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/** Error constants and constructor methods
 *
 * # Purpose
 * - Provides commonly used error messages as constants
 * - Offers convenient constructor methods for each error variant
 * - Ensures consistent error messaging across the codebase
 *
 * # Usage Examples
 * ```ignore
 * use opencli::result::OpenCliError;
 *
 * // Using constant error messages
 * return Err(OpenCliError::process(OpenCliError::COMPILER_NOT_FOUND));
 *
 * // Using constructor methods
 * return Err(OpenCliError::config("Invalid cache directory"));
 *
 * // Using dynamic messages
 * return Err(OpenCliError::not_found(format!("File {} not found", filename)));
 * ```
 */
impl OpenCliError {
    // Process-related error constants
    pub const COMPILER_NOT_FOUND: &'static str = "Compiler binary not found";
    pub const DOWNLOAD_FAILED: &'static str = "Download failed";
    pub const EXTRACTION_FAILED: &'static str = "Extraction failed";

    // Configuration-related error constants
    pub const INVALID_CONFIG: &'static str = "Invalid configuration format";

    // Server-related error constants
    pub const SERVER_START_FAILED: &'static str = "Failed to start server";

    /** Creates a Process error with flexible message input
     *
     * # Arguments
     * * `msg` - Message implementing Into<Cow<'static, str>>
     *
     * # Supported Input Types
     * - `&'static str` for static strings (no allocation)
     * - `String` for dynamic strings
     * - Any type implementing `Into<Cow<'static, str>>`
     *
     * # Example
     * ```ignore
     * OpenCliError::process("Custom process error");
     * OpenCliError::process(format!("Process {} failed", pid));
     * ```
     */
    pub fn process(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Process(msg.into())
    }

    /** Creates a Config error with flexible message input
     *
     * # Use Cases
     * - Invalid configuration formats
     * - Missing required configuration fields
     * - Configuration validation failures
     * - File permission issues
     */
    pub fn config(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Config(msg.into())
    }

    /** Creates a Server error with flexible message input
     *
     * # Use Cases
     * - HTTP server startup failures
     * - Network connectivity issues
     * - API endpoint errors
     * - Server configuration problems
     */
    pub fn server(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Server(msg.into())
    }

    /** Creates a NotFound error with flexible message input
     *
     * # Use Cases
     * - Missing files or directories
     * - Resource not found in cache
     * - Missing dependencies
     * - Configuration files not found
     */
    pub fn not_found(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::NotFound(msg.into())
    }
}

/*
 * Error Handling Best Practices:
 *
 * 1. When to use each error variant:
 *    - Io: File operations, network I/O, system calls
 *    - Process: External command execution, subprocess failures
 *    - Config: Configuration loading, parsing, validation
 *    - Server: HTTP server, API endpoints, network services
 *    - NotFound: Missing files, resources, dependencies
 *    - TomlParse/Serialize: TOML-specific parsing issues
 *    - JsonError: JSON serialization/deserialization
 *
 * 2. Performance Considerations:
 *    - Cow<'static, str> avoids allocation for static strings
 *    - Constructor methods provide zero-cost abstraction for literals
 *    - Automatic From conversions reduce boilerplate
 *
 * 3. Maintenance Guidelines:
 *    - Add new variants for distinct error categories
 *    - Use constants for commonly repeated error messages
 *    - Prefer specific error variants over generic ones
 *    - Provide context in error messages for better debugging
 *
 * 4. Integration with Logging:
 *    - Errors automatically implement Display via thiserror
 *    - Structured logging can extract error variants and context
 *    - Consider adding error codes for programmatic handling
 */
