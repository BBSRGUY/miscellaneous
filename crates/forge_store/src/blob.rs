//! # Blob Storage
//!
//! Filesystem-based blob storage for models, checkpoints, datasets, and logs.

use crate::{Result, StoreError};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use uuid::Uuid;

/// Blob storage manager.
pub struct BlobStorage {
    storage_root: PathBuf,
}

impl BlobStorage {
    /// Create a new blob storage manager.
    pub fn new<P: AsRef<Path>>(storage_root: P) -> Result<Self> {
        let storage_root = storage_root.as_ref().to_path_buf();

        // Create root directory if it doesn't exist
        if !storage_root.exists() {
            info!("Creating blob storage root: {}", storage_root.display());
            fs::create_dir_all(&storage_root)?;
        }

        // Create subdirectories
        for subdir in &["models", "checkpoints", "datasets", "logs"] {
            let path = storage_root.join(subdir);
            if !path.exists() {
                debug!("Creating blob storage directory: {}", path.display());
                fs::create_dir_all(&path)?;
            }
        }

        Ok(Self { storage_root })
    }

    /// Store a blob and return its path.
    pub fn store_blob(&self, category: BlobCategory, data: &[u8]) -> Result<String> {
        let blob_id = Uuid::new_v4().to_string();
        let category_dir = self.storage_root.join(category.as_str());
        let blob_path = category_dir.join(&blob_id);

        debug!("Storing blob: {}", blob_path.display());

        let mut file = fs::File::create(&blob_path)?;
        file.write_all(data)?;

        // Return relative path from storage root
        let relative_path = blob_path
            .strip_prefix(&self.storage_root)
            .map_err(|e| {
                StoreError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to compute relative path: {}", e),
                ))
            })?
            .to_string_lossy()
            .to_string();

        Ok(relative_path)
    }

    /// Store a blob with a specific name.
    pub fn store_blob_named(
        &self,
        category: BlobCategory,
        name: &str,
        data: &[u8],
    ) -> Result<String> {
        let category_dir = self.storage_root.join(category.as_str());
        let blob_path = category_dir.join(name);

        debug!("Storing named blob: {}", blob_path.display());

        // Create parent directories if needed
        if let Some(parent) = blob_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = fs::File::create(&blob_path)?;
        file.write_all(data)?;

        // Return relative path from storage root
        let relative_path = blob_path
            .strip_prefix(&self.storage_root)
            .map_err(|e| {
                StoreError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to compute relative path: {}", e),
                ))
            })?
            .to_string_lossy()
            .to_string();

        Ok(relative_path)
    }

    /// Retrieve a blob by its path.
    pub fn get_blob(&self, relative_path: &str) -> Result<Vec<u8>> {
        let blob_path = self.storage_root.join(relative_path);

        debug!("Retrieving blob: {}", blob_path.display());

        if !blob_path.exists() {
            return Err(StoreError::NotFound(format!(
                "Blob not found: {}",
                relative_path
            )));
        }

        let mut file = fs::File::open(&blob_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        Ok(data)
    }

    /// Get the absolute path for a blob.
    pub fn get_blob_path(&self, relative_path: &str) -> PathBuf {
        self.storage_root.join(relative_path)
    }

    /// Delete a blob.
    pub fn delete_blob(&self, relative_path: &str) -> Result<()> {
        let blob_path = self.storage_root.join(relative_path);

        debug!("Deleting blob: {}", blob_path.display());

        if blob_path.exists() {
            fs::remove_file(&blob_path)?;
        }

        Ok(())
    }

    /// List blobs in a category.
    pub fn list_blobs(&self, category: BlobCategory) -> Result<Vec<String>> {
        let category_dir = self.storage_root.join(category.as_str());

        if !category_dir.exists() {
            return Ok(Vec::new());
        }

        let mut blobs = Vec::new();

        for entry in fs::read_dir(&category_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Ok(relative_path) = entry.path().strip_prefix(&self.storage_root) {
                    blobs.push(relative_path.to_string_lossy().to_string());
                }
            }
        }

        Ok(blobs)
    }

    /// Get the size of a blob in bytes.
    pub fn get_blob_size(&self, relative_path: &str) -> Result<u64> {
        let blob_path = self.storage_root.join(relative_path);

        if !blob_path.exists() {
            return Err(StoreError::NotFound(format!(
                "Blob not found: {}",
                relative_path
            )));
        }

        let metadata = fs::metadata(&blob_path)?;
        Ok(metadata.len())
    }

    /// Check if a blob exists.
    pub fn blob_exists(&self, relative_path: &str) -> bool {
        self.storage_root.join(relative_path).exists()
    }

    /// Get the storage root path.
    pub fn storage_root(&self) -> &Path {
        &self.storage_root
    }
}

/// Blob storage category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobCategory {
    Models,
    Checkpoints,
    Datasets,
    Logs,
}

impl BlobCategory {
    /// Get the directory name for this category.
    pub fn as_str(&self) -> &str {
        match self {
            BlobCategory::Models => "models",
            BlobCategory::Checkpoints => "checkpoints",
            BlobCategory::Datasets => "datasets",
            BlobCategory::Logs => "logs",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_blob_storage_creation() {
        let temp_dir = tempdir().unwrap();
        let storage = BlobStorage::new(temp_dir.path()).unwrap();

        // Check that subdirectories were created
        for subdir in &["models", "checkpoints", "datasets", "logs"] {
            assert!(storage.storage_root().join(subdir).exists());
        }
    }

    #[test]
    fn test_store_and_retrieve_blob() {
        let temp_dir = tempdir().unwrap();
        let storage = BlobStorage::new(temp_dir.path()).unwrap();

        let data = b"test blob data";
        let path = storage
            .store_blob(BlobCategory::Models, data)
            .unwrap();

        let retrieved = storage.get_blob(&path).unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_store_named_blob() {
        let temp_dir = tempdir().unwrap();
        let storage = BlobStorage::new(temp_dir.path()).unwrap();

        let data = b"test model file";
        let path = storage
            .store_blob_named(BlobCategory::Models, "test-model.bin", data)
            .unwrap();

        assert!(path.contains("test-model.bin"));

        let retrieved = storage.get_blob(&path).unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_delete_blob() {
        let temp_dir = tempdir().unwrap();
        let storage = BlobStorage::new(temp_dir.path()).unwrap();

        let data = b"to be deleted";
        let path = storage
            .store_blob(BlobCategory::Datasets, data)
            .unwrap();

        assert!(storage.blob_exists(&path));

        storage.delete_blob(&path).unwrap();
        assert!(!storage.blob_exists(&path));
    }

    #[test]
    fn test_blob_size() {
        let temp_dir = tempdir().unwrap();
        let storage = BlobStorage::new(temp_dir.path()).unwrap();

        let data = b"12345678";
        let path = storage
            .store_blob(BlobCategory::Logs, data)
            .unwrap();

        let size = storage.get_blob_size(&path).unwrap();
        assert_eq!(size, 8);
    }

    #[test]
    fn test_list_blobs() {
        let temp_dir = tempdir().unwrap();
        let storage = BlobStorage::new(temp_dir.path()).unwrap();

        // Store multiple blobs
        storage
            .store_blob(BlobCategory::Models, b"blob1")
            .unwrap();
        storage
            .store_blob(BlobCategory::Models, b"blob2")
            .unwrap();

        let blobs = storage.list_blobs(BlobCategory::Models).unwrap();
        assert_eq!(blobs.len(), 2);
    }
}
