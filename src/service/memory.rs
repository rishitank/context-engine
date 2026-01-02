//! Memory service for persistent agent memory.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::info;

use crate::error::Result;

/// A memory entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Entry key
    pub key: String,
    /// Entry value
    pub value: String,
    /// Entry type/category
    #[serde(default)]
    pub entry_type: String,
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
    /// Metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Memory store.
#[derive(Debug, Default, Serialize, Deserialize)]
struct MemoryStore {
    entries: HashMap<String, MemoryEntry>,
}

/// Memory service for persistent storage.
pub struct MemoryService {
    store: Arc<RwLock<MemoryStore>>,
    storage_path: PathBuf,
}

impl MemoryService {
    /// Create a new memory service.
    pub async fn new(workspace: &Path) -> Result<Self> {
        let storage_path = workspace.join(".context-engine").join("memory.json");
        
        // Create directory if needed
        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Load existing store
        let store = if storage_path.exists() {
            let content = fs::read_to_string(&storage_path).await?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            MemoryStore::default()
        };

        Ok(Self {
            store: Arc::new(RwLock::new(store)),
            storage_path,
        })
    }

    /// Save the store to disk.
    async fn save(&self) -> Result<()> {
        let store = self.store.read().await;
        let content = serde_json::to_string_pretty(&*store)?;
        fs::write(&self.storage_path, content).await?;
        Ok(())
    }

    /// Store a memory entry.
    pub async fn store(&self, key: String, value: String, entry_type: Option<String>) -> Result<MemoryEntry> {
        let now = chrono::Utc::now().to_rfc3339();
        
        let entry = MemoryEntry {
            key: key.clone(),
            value,
            entry_type: entry_type.unwrap_or_else(|| "general".to_string()),
            created_at: now.clone(),
            updated_at: now,
            metadata: HashMap::new(),
        };

        {
            let mut store = self.store.write().await;
            store.entries.insert(key, entry.clone());
        }

        self.save().await?;
        info!("Stored memory entry: {}", entry.key);
        
        Ok(entry)
    }

    /// Retrieve a memory entry.
    pub async fn retrieve(&self, key: &str) -> Option<MemoryEntry> {
        let store = self.store.read().await;
        store.entries.get(key).cloned()
    }

    /// List all memory entries.
    pub async fn list(&self, entry_type: Option<&str>) -> Vec<MemoryEntry> {
        let store = self.store.read().await;
        
        store.entries
            .values()
            .filter(|e| {
                entry_type.map_or(true, |t| e.entry_type == t)
            })
            .cloned()
            .collect()
    }

    /// Delete a memory entry.
    pub async fn delete(&self, key: &str) -> Result<bool> {
        let removed = {
            let mut store = self.store.write().await;
            store.entries.remove(key).is_some()
        };

        if removed {
            self.save().await?;
            info!("Deleted memory entry: {}", key);
        }

        Ok(removed)
    }

    /// Clear all memory entries.
    pub async fn clear(&self) -> Result<usize> {
        let count = {
            let mut store = self.store.write().await;
            let count = store.entries.len();
            store.entries.clear();
            count
        };

        self.save().await?;
        info!("Cleared {} memory entries", count);
        
        Ok(count)
    }

    /// Search memory entries by value.
    pub async fn search(&self, query: &str) -> Vec<MemoryEntry> {
        let store = self.store.read().await;
        let query_lower = query.to_lowercase();

        store.entries
            .values()
            .filter(|e| {
                e.key.to_lowercase().contains(&query_lower)
                    || e.value.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_service() -> (MemoryService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let service = MemoryService::new(temp_dir.path()).await.unwrap();
        (service, temp_dir)
    }

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let (service, _temp) = create_test_service().await;

        let entry = service.store(
            "test-key".to_string(),
            "test-value".to_string(),
            Some("test-type".to_string()),
        ).await.unwrap();

        assert_eq!(entry.key, "test-key");
        assert_eq!(entry.value, "test-value");
        assert_eq!(entry.entry_type, "test-type");

        let retrieved = service.retrieve("test-key").await.unwrap();
        assert_eq!(retrieved.value, "test-value");
    }

    #[tokio::test]
    async fn test_retrieve_nonexistent() {
        let (service, _temp) = create_test_service().await;
        let result = service.retrieve("nonexistent").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list_all() {
        let (service, _temp) = create_test_service().await;

        service.store("key1".to_string(), "value1".to_string(), None).await.unwrap();
        service.store("key2".to_string(), "value2".to_string(), None).await.unwrap();

        let all = service.list(None).await;
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_list_by_type() {
        let (service, _temp) = create_test_service().await;

        service.store("key1".to_string(), "value1".to_string(), Some("type-a".to_string())).await.unwrap();
        service.store("key2".to_string(), "value2".to_string(), Some("type-b".to_string())).await.unwrap();
        service.store("key3".to_string(), "value3".to_string(), Some("type-a".to_string())).await.unwrap();

        let type_a = service.list(Some("type-a")).await;
        assert_eq!(type_a.len(), 2);

        let type_b = service.list(Some("type-b")).await;
        assert_eq!(type_b.len(), 1);
    }

    #[tokio::test]
    async fn test_delete() {
        let (service, _temp) = create_test_service().await;

        service.store("to-delete".to_string(), "value".to_string(), None).await.unwrap();
        assert!(service.retrieve("to-delete").await.is_some());

        let deleted = service.delete("to-delete").await.unwrap();
        assert!(deleted);
        assert!(service.retrieve("to-delete").await.is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent() {
        let (service, _temp) = create_test_service().await;
        let deleted = service.delete("nonexistent").await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_clear() {
        let (service, _temp) = create_test_service().await;

        service.store("key1".to_string(), "value1".to_string(), None).await.unwrap();
        service.store("key2".to_string(), "value2".to_string(), None).await.unwrap();

        let cleared = service.clear().await.unwrap();
        assert_eq!(cleared, 2);

        let all = service.list(None).await;
        assert!(all.is_empty());
    }

    #[tokio::test]
    async fn test_search() {
        let (service, _temp) = create_test_service().await;

        service.store("config".to_string(), "database connection string".to_string(), None).await.unwrap();
        service.store("note".to_string(), "remember to check database".to_string(), None).await.unwrap();
        service.store("other".to_string(), "unrelated content".to_string(), None).await.unwrap();

        let results = service.search("database").await;
        assert_eq!(results.len(), 2);

        let results = service.search("CONFIG").await; // Case insensitive
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_default_entry_type() {
        let (service, _temp) = create_test_service().await;

        let entry = service.store("key".to_string(), "value".to_string(), None).await.unwrap();
        assert_eq!(entry.entry_type, "general");
    }

    #[tokio::test]
    async fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();

        // Create and populate
        {
            let service = MemoryService::new(temp_dir.path()).await.unwrap();
            service.store("persistent".to_string(), "data".to_string(), None).await.unwrap();
        }

        // Reload and verify
        {
            let service = MemoryService::new(temp_dir.path()).await.unwrap();
            let entry = service.retrieve("persistent").await;
            assert!(entry.is_some());
            assert_eq!(entry.unwrap().value, "data");
        }
    }

    #[test]
    fn test_memory_entry_serialization() {
        let entry = MemoryEntry {
            key: "test".to_string(),
            value: "value".to_string(),
            entry_type: "general".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            metadata: HashMap::new(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: MemoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.key, entry.key);
    }
}
