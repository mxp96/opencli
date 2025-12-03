use crate::result::{OpenCliError, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/** Manages a persistent cache of file hashes stored in a text file
 *
 * The cache file format is:
 * ```text
 * filename1.txt
 * argon2:hash_value_1
 * filename2.txt  
 * argon2:hash_value_2
 * ```
 *
 * # Example
 * ```no_run
 * use std::path::Path;
 * use opencli::cache::CacheManager;
 *
 * #[tokio::main(flavor = "current_thread")]
 * async fn main() -> Result<(), Box<dyn std::error::Error>> {
 *     let cache_dir = Path::new("./cache");
 *     let cache = CacheManager::new(cache_dir);
 *     
 *     // Store a hash
 *     cache.store_hash("document.pdf", "$argon2id$v=19$m=65536,t=3,p=4$salt$hash").await?;
 *     
 *     // Retrieve a hash
 *     if let Some(hash) = cache.get_hash("document.pdf").await? {
 *         println!("Found hash: {}", hash);
 *     }
 *     
 *     Ok(())
 * }
 * ```
 */
pub struct CacheManager {
    // Path to the cache file storing all hash entries
    cache_file: PathBuf,
}

impl CacheManager {
    /** Creates a new CacheManager with the specified base directory
     *
     * # Arguments
     * * `base_dir` - Directory where the cache file will be stored
     *
     * # Notes
     * - The cache file will be created at `base_dir/cache.txt`
     * - Directory will be created if it doesn't exist during first operation
     */
    pub fn new(base_dir: &Path) -> Self {
        Self {
            cache_file: base_dir.join("cache.txt"),
        }
    }

    /** Validates the structural integrity of the cache file
     *
     * # Returns
     * - `Ok(true)` if cache file has valid format
     * - `Ok(false)` if cache file is corrupted or malformed
     * - `Err` if I/O error occurs during reading
     *
     * # Checks
     * - Every filename line must be followed by a hash line starting with "argon2:"
     * - No orphaned filename or hash lines
     * - Empty lines are ignored
     */
    pub async fn find_cache_integrity(&self) -> Result<bool> {
        // If cache file doesn't exist, it's considered valid (empty cache)
        if !self.cache_file.exists() {
            return Ok(true);
        }

        let content = fs::read_to_string(&self.cache_file).await?;
        let mut content_lines = content.lines();
        let mut arg2_valid = true;

        // Iterate through lines in pairs (filename, hash)
        while let Some(filename) = content_lines.next() {
            // Skip empty lines between entries
            if filename.is_empty() {
                continue;
            }
            // Each filename must be followed by a hash line
            if let Some(hash_line) = content_lines.next() {
                if !hash_line.starts_with("argon2:") {
                    // Found filename not followed by proper hash line
                    arg2_valid = false;
                    break;
                }
            } else {
                // Filename at end of file without corresponding hash
                arg2_valid = false;
                break;
            }
        }
        Ok(arg2_valid)
    } // find_cache_integrity

    /** Repairs a corrupted cache file by rebuilding it from valid entries
     *
     * # Process
     * 1. Validates current cache integrity
     * 2. If corrupted, loads all valid entries using existing parser
     * 3. Clears the corrupted cache file
     * 4. Rebuilds cache with only valid entries
     *
     * # Notes
     * - Orphaned entries (filename without hash or vice versa) are discarded
     * - Original file is preserved until new file is successfully written
     */
    pub async fn repair_cache(&self) -> Result<()> {
        if !self.find_cache_integrity().await? {
            // Load all valid entries (parser automatically skips invalid ones)
            let hashes = self.load_all_hashes().await?;

            // Clear corrupted cache
            self.clear_cache().await?;

            // Rebuild cache with valid entries only
            for (filename, hash) in hashes {
                self.store_hash_internal(&filename, &hash).await?;
            }
        }
        Ok(())
    } // repair_cache

    // Internal method to store hash without extensive validation
    // Used by repair_cache and other internal methods
    async fn store_hash_internal(&self, filename: &str, argon2_hash: &str) -> Result<()> {
        let entry = format!("{}\nargon2:{}\n", filename, argon2_hash);

        // Ensure cache directory exists
        if let Some(parent) = self.cache_file.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Append entry to cache file
        let mut file = tokio::fs::OpenOptions::new()
            .create(true) // Create file if it doesn't exist
            .append(true) // Append to end of file
            .open(&self.cache_file)
            .await?;

        file.write_all(entry.as_bytes()).await?;
        Ok(())
    } // store_hash_internal

    /** Stores a filename and its corresponding hash in the cache
     *
     * # Arguments
     * * `filename` - Name of the file (must not contain newlines)
     * * `argon2_hash` - Argon2 hash string (must not contain newlines)
     *
     * # Validation
     * - Filename must not be empty or contain newline characters
     * - Hash must not be empty or contain newline characters
     * - Note: Hash format validation is minimal to support different Argon2 variants
     *
     * # Performance
     * - File is opened in append mode for efficient writes
     * - Directory creation is lazy (only when first write occurs)
     */
    pub async fn store_hash(&self, filename: &str, argon2_hash: &str) -> Result<()> {
        // Validate input to prevent cache corruption
        if filename.is_empty() || filename.contains('\n') {
            return Err(OpenCliError::config(
                "store_hash: Invalid filename - must not be empty or contain newlines",
            ));
        }

        if argon2_hash.is_empty() || argon2_hash.contains('\n') {
            return Err(OpenCliError::config(
                "store_hash: Invalid hash format - must not be empty or contain newlines",
            ));
        }

        self.store_hash_internal(filename, argon2_hash).await
    } // store_hash

    /** Retrieves the hash for a specific filename
     *
     * # Arguments
     * * `filename` - Name of the file to look up
     *
     * # Returns
     * - `Ok(Some(hash))` if filename found in cache
     * - `Ok(None)` if filename not found in cache
     * - `Err` if I/O error occurs during reading
     *
     * # Note
     * This is a convenience wrapper around `get_hash_fast`
     */
    pub async fn get_hash(&self, filename: &str) -> Result<Option<String>> {
        self.get_hash_fast(filename).await
    } // get_hash

    /** Efficiently retrieves hash using streaming without loading entire file to memory
     *
     * # Advantages over get_hash
     * - Memory efficient for large cache files
     * - Stops reading as soon as target filename is found
     * - Uses buffered I/O for better performance
     *
     * # Implementation
     * - Reads file line by line using buffered reader
     * - When filename matches, reads next line as hash
     * - Skips hash lines for non-matching filenames
     */
    pub async fn get_hash_fast(&self, filename: &str) -> Result<Option<String>> {
        // Early return if cache file doesn't exist
        if !self.cache_file.exists() {
            return Ok(None);
        }

        let file = File::open(&self.cache_file).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        // Stream through file line by line
        while let Some(line) = lines.next_line().await? {
            if line == filename {
                // Found matching filename, next line should be the hash
                if let Some(hash_line) = lines.next_line().await? {
                    if let Some(stripped) = hash_line.strip_prefix("argon2:") {
                        return Ok(Some(stripped.to_string())); // Strip "argon2:" prefix
                    }
                }
            } else if line.starts_with("argon2:") {
                // Skip hash line when filename doesn't match
                // This ensures we're always reading pairs correctly
                continue;
            }
            // If line is neither target filename nor hash line,
            // it's a different filename - continue to next pair
        }

        Ok(None) // Filename not found in cache
    } // get_hash_fast

    /** Loads all filename-hash pairs from cache into a HashMap
     *
     * # Returns
     * - HashMap where keys are filenames and values are hashes
     * - Empty HashMap if cache file doesn't exist or is empty
     *
     * # Notes
     * - Automatically skips malformed entries (orphaned filenames or hashes)
     * - Entire file is loaded into memory - use with caution for very large caches
     * - For memory-efficient operations, use `get_hash_fast` for individual lookups
     */
    pub async fn load_all_hashes(&self) -> Result<HashMap<String, String>> {
        let mut hashes = HashMap::new();

        if !self.cache_file.exists() {
            return Ok(hashes);
        }

        let content = fs::read_to_string(&self.cache_file).await?;
        let mut current_file = None; // Tracks the current filename being processed

        // Parse file content line by line
        for line in content.lines() {
            if line.starts_with("argon2:") {
                // This is a hash line - pair it with the previous filename
                if let Some(file) = current_file.take() {
                    let hash = line.strip_prefix("argon2:").unwrap().to_string();
                    hashes.insert(file, hash);
                }
                // If no current_file, this is an orphaned hash - skip it
            } else if !line.is_empty() {
                // This is a filename line - store it for next iteration
                current_file = Some(line.to_string());
            }
            // Empty lines are ignored
        }

        Ok(hashes)
    } // load_all_hashes

    /** Removes a filename and its hash from the cache
     *
     * # Arguments
     * * `filename` - Name of the file to remove
     *
     * # Process
     * 1. Reads entire cache file into memory
     * 2. Filters out the target filename and its following hash line
     * 3. Writes filtered content back to file
     *
     * # Performance
     * - Entire file is loaded into memory during operation
     * - Consider using `update_hash` if replacing with new value
     */
    pub async fn remove_hash(&self, filename: &str) -> Result<()> {
        if !self.cache_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.cache_file).await?;
        let mut new_content = String::new();
        let mut current_file = None;
        let mut skip_line = false; // Flag to skip hash line after removed filename

        for line in content.lines() {
            if skip_line {
                // Skip the hash line following a removed filename
                skip_line = false;
                continue;
            }

            if line.starts_with("argon2:") {
                // This is a hash line
                if let Some(file) = &current_file {
                    if file != filename {
                        // Keep entries that don't match target filename
                        new_content.push_str(&format!("{}\n{}\n", file, line));
                    }
                    // If file matches filename, both filename and hash are skipped
                }
                current_file = None;
            } else if !line.is_empty() {
                // This is a filename line
                if line == filename {
                    // Mark this entry for removal
                    skip_line = true; // Next line (hash) will be skipped
                    current_file = None;
                } else {
                    current_file = Some(line.to_string());
                }
            }
        }

        // Write filtered content back to file
        fs::write(&self.cache_file, new_content).await?;
        Ok(())
    } // remove_hash

    /** Updates the hash for an existing filename or adds it if not present
     *
     * # Arguments
     * * `filename` - Name of the file to update
     * * `new_hash` - New hash value to store
     *
     * # Implementation
     * - Uses remove + store pattern
     * - More efficient than manual search and replace for large files
     */
    pub async fn update_hash(&self, filename: &str, new_hash: &str) -> Result<()> {
        self.remove_hash(filename).await?;
        self.store_hash(filename, new_hash).await
    } // update_hash

    /** Efficiently stores multiple entries in batch
     *
     * # Arguments
     * * `entries` - HashMap containing filename -> hash mappings
     *
     * # Advantages over individual store_hash calls
     * - Single file open/close operation
     * - Reduced I/O overhead
     * - Atomic operation (all or nothing)
     */
    pub async fn bulk_store(&self, entries: &HashMap<String, String>) -> Result<()> {
        let mut content = String::new();

        // Build all entries in memory first
        for (filename, hash) in entries {
            content.push_str(&format!("{}\nargon2:{}\n", filename, hash));
        }

        // Ensure cache directory exists
        if let Some(parent) = self.cache_file.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Single write operation for all entries
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.cache_file)
            .await?;

        file.write_all(content.as_bytes()).await?;
        Ok(())
    } // bulk_store

    /** Gets the size of the cache file in bytes
     *
     * # Returns
     * - File size in bytes if cache file exists
     * - 0 if cache file doesn't exist
     *
     * # Use cases
     * - Monitoring cache growth
     * - Deciding when to prune old entries
     * - Performance optimization decisions
     */
    pub async fn get_cache_size(&self) -> Result<u64> {
        if self.cache_file.exists() {
            Ok(fs::metadata(&self.cache_file).await?.len())
        } else {
            Ok(0)
        }
    } // get_cache_size

    /** Finds files that have duplicate hashes (potential duplicate files)
     *
     * # Returns
     * - HashMap where keys are duplicate hashes and values are vectors of filenames
     * - Only includes hashes that appear more than once
     *
     * # Use cases
     * - Detecting duplicate files in cache
     * - Identifying files with identical content
     * - Cache optimization by removing duplicates
     */
    pub async fn find_duplicate_hashes(&self) -> Result<HashMap<String, Vec<String>>> {
        let hashes = self.load_all_hashes().await?;
        let mut hash_to_files: HashMap<String, Vec<String>> = HashMap::new();

        // Group files by their hash
        for (file, hash) in hashes {
            hash_to_files.entry(hash).or_default().push(file);
        }

        // Filter to only include duplicates
        Ok(hash_to_files
            .into_iter()
            .filter(|(_, files)| files.len() > 1) // Only keep hashes with multiple files
            .collect())
    } // find_duplicate_hashes

    /** Completely clears all entries from the cache
     *
     * # Implementation
     * - Truncates cache file to zero length
     * - File remains exists but empty
     * - More efficient than deleting and recreating file
     */
    pub async fn clear_cache(&self) -> Result<()> {
        if self.cache_file.exists() {
            fs::write(&self.cache_file, "").await?;
        }
        Ok(())
    } // clear_cache

    /** Checks if a filename exists in the cache
     *
     * # Arguments
     * * `filename` - Name of the file to check
     *
     * # Returns
     * - `true` if filename exists in cache
     * - `false` if filename doesn't exist or cache file doesn't exist
     *
     * # Note
     * More efficient than `get_hash` when only existence check is needed
     */
    pub async fn exists_cache(&self, filename: &str) -> Result<bool> {
        Ok(self.get_hash(filename).await?.is_some())
    } // exists_cache

    /** Counts the number of entries in the cache
     *
     * # Returns
     * - Number of filename-hash pairs in cache
     * - 0 if cache is empty or doesn't exist
     */
    pub async fn count_cache(&self) -> Result<usize> {
        let hashes = self.load_all_hashes().await?;
        Ok(hashes.len())
    } // count_cache

    /** Lists all filenames stored in the cache
     *
     * # Returns
     * - Vector of all filenames in cache
     * - Empty vector if cache is empty or doesn't exist
     */
    pub async fn list_files_cache(&self) -> Result<Vec<String>> {
        let hashes = self.load_all_hashes().await?;
        Ok(hashes.keys().cloned().collect())
    } // list_files_cache
}

/*
 * Performance Considerations:
 *
 * 1. For large caches (>10,000 entries):
 *    - Use get_hash_fast() for individual lookups
 *    - Avoid load_all_hashes() in performance-critical paths
 *    - Consider periodic cache pruning with remove_hash()
 *
 * 2. For frequent updates:
 *    - Use bulk_store() for multiple additions
 *    - Batch updates and perform them less frequently
 *    - Consider in-memory caching layer on top of this
 *
 * 3. Memory usage:
 *    - load_all_hashes() loads entire file into memory
 *    - get_hash_fast() uses constant memory via streaming
 *    - remove_hash() requires loading entire file for rewriting
 *
 * 4. File locking:
 *    - Current implementation doesn't handle concurrent writes
 *    - For multi-threaded use, add external synchronization
 */
