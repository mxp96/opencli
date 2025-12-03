/// OpenCLI - A secure, cache-optimized build system
///
/// This crate provides a comprehensive build system with focus on:
/// - Security through cryptographic hashing (Argon2)
/// - Efficient caching mechanisms  
/// - Parallel compilation
/// - Dependency tracking
///
/// Main modules:
/// - build: Core build pipeline and dependency resolution
/// - cache: File-based caching system with integrity validation
/// - cli: Command-line interface parsing and execution
/// - commands: Implementation of build commands and subcommands
/// - compiler: Compiler abstraction and toolchain management
/// - package: Package configuration and manifest handling
/// - result: Error handling and result types
/// - security: Cryptographic utilities and hash management
/// - utils: Common utilities and helper functions
pub mod build;
pub mod cache;
pub mod cli;
pub mod commands;
pub mod compiler;
pub mod package;
pub mod result;
pub mod security;
pub mod utils;
