use crate::result::{OpenCliError, Result};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::fs;

/** Cryptographic manager for file hashing and verification operations
 *
 * # Architecture
 * - Two-stage hashing: SHA-256 → Argon2
 * - SHA-256 provides fast content fingerprinting
 * - Argon2 adds cryptographic security with salt
 * - Salt generation using OS-provided cryptographically secure RNG
 *
 * # Security Considerations
 * - Each file hash gets a unique salt
 * - Argon2 parameters provide memory-hard hashing
 * - Verification is constant-time to prevent timing attacks
 * - No sensitive data stored in memory longer than necessary
 */
#[derive(Default)]
pub struct SecurityManager {
    argon2: Argon2<'static>,
}

impl SecurityManager {
    /** Creates a new SecurityManager with default Argon2 parameters
     *
     * # Default Configuration
     * - Algorithm: Argon2id (hybrid version)
     * - Memory cost: 19,456 KiB
     * - Time cost: 2 iterations
     * - Parallelism: 1 thread
     *
     * # Notes
     * - Default parameters provide good security for most use cases
     * - Consider custom parameters for high-security applications
     */
    pub fn new() -> Self {
        Self::default()
    }

    /** Computes a secure hash of a file's content
     *
     * # Process
     * 1. Reads file content asynchronously
     * 2. Computes SHA-256 hash of content
     * 3. Generates cryptographically secure salt
     * 4. Applies Argon2 hashing to SHA-256 result with salt
     *
     * # Arguments
     * * `file_path` - Path to the file to hash
     *
     * # Returns
     * - Argon2 hash string in PHC format
     * - Format: `$argon2id$v=19$m=19456,t=2,p=1$salt$hash`
     *
     * # Errors
     * - `OpenCliError::Io` if file cannot be read
     * - `OpenCliError::Process` if hashing operation fails
     *
     * # Example
     * ```ignore
     * let security = SecurityManager::new();
     * let hash = security.hash_file(Path::new("document.pdf")).await?;
     * println!("File hash: {}", hash);
     * ```
     */
    pub async fn hash_file(&self, file_path: &Path) -> Result<String> {
        // Read file content asynchronously
        let content = fs::read(file_path).await?;

        // Compute SHA-256 hash of file content
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let file_hash = hasher.finalize();

        // Generate cryptographically secure salt
        let salt = SaltString::generate(&mut OsRng);

        // Apply Argon2 hashing with salt
        let argon2_hash = self
            .argon2
            .hash_password(&file_hash, &salt)
            .map_err(|e| OpenCliError::Process(format!("Failed to hash file: {}", e).into()))?;

        Ok(argon2_hash.to_string())
    }

    /** Verifies file content against a stored Argon2 hash
     *
     * # Process
     * 1. Computes SHA-256 hash of current file content
     * 2. Parses stored Argon2 hash string
     * 3. Verifies computed hash against stored hash
     *
     * # Arguments
     * * `file_path` - Path to the file to verify
     * * `stored_hash` - Previously computed Argon2 hash string
     *
     * # Returns
     * - `Ok(true)` if file content matches stored hash
     * - `Ok(false)` if file content does not match
     * - `Err` if file cannot be read or hash format is invalid
     *
     * # Security Notes
     * - Uses constant-time comparison to prevent timing attacks
     * - Returns false on mismatch rather than detailed error
     * - Invalid hash format returns error to indicate corruption
     *
     * # Example
     * ```ignore
     * let security = SecurityManager::new();
     * let is_valid = security.verify_file(
     *     Path::new("document.pdf"),
     *     "$argon2id$v=19$m=19456,t=2,p=1$salt$hash"
     * ).await?;
     *
     * if is_valid {
     *     println!("File integrity verified");
     * } else {
     *     println!("File has been modified");
     * }
     * ```
     */
    pub async fn verify_file(&self, file_path: &Path, stored_hash: &str) -> Result<bool> {
        // Compute current SHA-256 hash of file
        let content = fs::read(file_path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let file_hash = hasher.finalize();

        // Parse stored Argon2 hash
        let parsed_hash = PasswordHash::new(stored_hash)
            .map_err(|e| OpenCliError::Process(format!("Invalid hash format: {}", e).into()))?;

        // Verify against stored hash (constant-time operation)
        match self.argon2.verify_password(&file_hash, &parsed_hash) {
            Ok(_) => Ok(true),   // Hashes match
            Err(_) => Ok(false), // Hashes don't match
        }
    }

    /** Hashes pre-computed content hash with Argon2
     *
     * # Purpose
     * - Useful when SHA-256 hash is already computed
     * - Allows separation of content hashing from secure hashing
     * - Enables batch processing of multiple content hashes
     *
     * # Arguments
     * * `content_hash` - Pre-computed SHA-256 hash bytes
     *
     * # Returns
     * - Argon2 hash string in PHC format
     *
     * # Use Cases
     * - Batch processing of multiple files
     * - Re-hashing existing SHA-256 hashes
     * - Integration with external content hashing systems
     *
     * # Example
     * ```ignore
     * let security = SecurityManager::new();
     * let sha256_hash = compute_sha256("file content");
     * let secure_hash = security.hash_file_content(&sha256_hash).await?;
     * ```
     */
    pub async fn hash_file_content(&self, content_hash: &[u8]) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2_hash = self
            .argon2
            .hash_password(content_hash, &salt)
            .map_err(|e| OpenCliError::Process(format!("Failed to hash content: {}", e).into()))?;

        Ok(argon2_hash.to_string())
    }
}

/*
 * Cryptographic Security Considerations:
 *
 * 1. Algorithm Choices:
 *    - SHA-256: Industry standard for fast, collision-resistant hashing
 *    - Argon2: Memory-hard algorithm, winner of Password Hashing Competition
 *    - Argon2id: Hybrid version resistant to both GPU and side-channel attacks
 *
 * 2. Salt Generation:
 *    - Uses OsRng for cryptographically secure random numbers
 *    - Each hash gets unique 16-byte salt
 *    - Prevents rainbow table attacks
 *
 * 3. Performance Trade-offs:
 *    - SHA-256: Fast content fingerprinting (~500 MB/s on modern CPUs)
 *    - Argon2: Deliberately slow for security (~10-100ms per hash)
 *    - Suitable for file change detection, not real-time operations
 *
 * 4. Security Parameters:
 *    - Default Argon2 parameters balance security and performance
 *    - Memory cost: 19MB (adequate for most applications)
 *    - Time cost: 2 iterations (reasonable for file verification)
 *    - Can be customized for higher security requirements
 *
 * 5. Threat Model:
 *    - Protects against: Content tampering, Hash collision attacks
 *    - Resistant to: Rainbow tables, GPU cracking, Timing attacks
 *    - Assumes: Secure storage of resulting hash values
 */
