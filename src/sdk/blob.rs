//! Blob naming and size calculations.
//!
//! Blobs are identified by SHA256 hash of path + contents.

use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

/// Version of the blob naming scheme.
pub const BLOB_NAMING_VERSION: u64 = 2023102300;

/// Default maximum blob size (1MB).
pub const DEFAULT_MAX_BLOB_SIZE: usize = 1024 * 1024;

/// Calculator for blob names (SHA256 hashes).
#[derive(Debug, Clone)]
pub struct BlobNameCalculator {
    max_blob_size: usize,
}

impl BlobNameCalculator {
    /// Create a new calculator with the given max blob size.
    pub fn new(max_blob_size: usize) -> Self {
        Self { max_blob_size }
    }

    /// Create a calculator with the default max size (1MB).
    pub fn default_size() -> Self {
        Self::new(DEFAULT_MAX_BLOB_SIZE)
    }

    /// Get the maximum blob size.
    pub fn max_blob_size(&self) -> usize {
        self.max_blob_size
    }

    /// Calculate the blob name (SHA256 hash) for a file.
    ///
    /// Returns an error if the file exceeds the maximum size.
    pub fn calculate_or_throw(&self, path: &str, contents: &[u8], check_size: bool) -> Result<String> {
        if check_size && contents.len() > self.max_blob_size {
            return Err(Error::BlobTooLarge {
                max_size: self.max_blob_size,
            });
        }

        Ok(self.hash(path, contents))
    }

    /// Calculate the blob name, returning None if the file is too large.
    pub fn calculate(&self, path: &str, contents: &[u8]) -> Option<String> {
        self.calculate_or_throw(path, contents, true).ok()
    }

    /// Calculate the blob name without size checking.
    pub fn calculate_no_throw(&self, path: &str, contents: &[u8]) -> String {
        self.calculate_or_throw(path, contents, false)
            .expect("calculate_no_throw should never fail")
    }

    /// Calculate the blob name from a string.
    pub fn calculate_from_str(&self, path: &str, contents: &str) -> Option<String> {
        self.calculate(path, contents.as_bytes())
    }

    /// Calculate the blob name from a string without size checking.
    pub fn calculate_from_str_no_throw(&self, path: &str, contents: &str) -> String {
        self.calculate_no_throw(path, contents.as_bytes())
    }

    /// Compute SHA256 hash of path + contents.
    fn hash(&self, path: &str, contents: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(path.as_bytes());
        hasher.update(contents);
        hex::encode(hasher.finalize())
    }
}

impl Default for BlobNameCalculator {
    fn default() -> Self {
        Self::default_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_name_calculation() {
        let calc = BlobNameCalculator::default();
        let name = calc.calculate("src/main.rs", b"fn main() {}").unwrap();
        
        // Should be a 64-character hex string (SHA256)
        assert_eq!(name.len(), 64);
        assert!(name.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_blob_name_consistency() {
        let calc = BlobNameCalculator::default();
        let name1 = calc.calculate("test.txt", b"hello").unwrap();
        let name2 = calc.calculate("test.txt", b"hello").unwrap();
        
        // Same input should produce same output
        assert_eq!(name1, name2);
    }

    #[test]
    fn test_blob_name_different_for_different_paths() {
        let calc = BlobNameCalculator::default();
        let name1 = calc.calculate("a.txt", b"hello").unwrap();
        let name2 = calc.calculate("b.txt", b"hello").unwrap();
        
        // Different paths should produce different hashes
        assert_ne!(name1, name2);
    }

    #[test]
    fn test_blob_too_large() {
        let calc = BlobNameCalculator::new(10);
        let result = calc.calculate("test.txt", b"this is more than 10 bytes");
        
        assert!(result.is_none());
    }

    #[test]
    fn test_blob_no_throw() {
        let calc = BlobNameCalculator::new(10);
        let name = calc.calculate_no_throw("test.txt", b"this is more than 10 bytes");
        
        // Should still produce a hash
        assert_eq!(name.len(), 64);
    }
}

