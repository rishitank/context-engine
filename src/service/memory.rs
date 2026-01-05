//! Memory service for persistent agent memory.
//!
//! This module provides a rich memory storage system compatible with
//! the m1rl0k/Context-Engine memory API, supporting:
//! - Rich metadata (kind, language, path, tags, priority, topic, code, author)
//! - Hybrid search (text matching + metadata filtering)
//! - Priority-based ranking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::error::Result;

/// Memory entry kind/category.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MemoryKind {
    /// Code snippet or pattern
    Snippet,
    /// Technical explanation
    Explanation,
    /// Design pattern or approach
    Pattern,
    /// Usage example
    Example,
    /// Reference information
    Reference,
    /// General memory (default)
    #[default]
    Memory,
}

impl std::fmt::Display for MemoryKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryKind::Snippet => write!(f, "snippet"),
            MemoryKind::Explanation => write!(f, "explanation"),
            MemoryKind::Pattern => write!(f, "pattern"),
            MemoryKind::Example => write!(f, "example"),
            MemoryKind::Reference => write!(f, "reference"),
            MemoryKind::Memory => write!(f, "memory"),
        }
    }
}

impl std::str::FromStr for MemoryKind {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "snippet" => Ok(MemoryKind::Snippet),
            "explanation" => Ok(MemoryKind::Explanation),
            "pattern" => Ok(MemoryKind::Pattern),
            "example" => Ok(MemoryKind::Example),
            "reference" => Ok(MemoryKind::Reference),
            "memory" | "general" => Ok(MemoryKind::Memory),
            _ => Ok(MemoryKind::Memory),
        }
    }
}

/// Rich metadata for memory entries (compatible with m1rl0k/Context-Engine).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryMetadata {
    /// Category type (snippet, explanation, pattern, example, reference)
    #[serde(default)]
    pub kind: MemoryKind,
    /// Programming language (e.g., "python", "javascript", "rust")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// File path context for code-related entries
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Searchable tags for categorization
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Importance ranking (1-10, higher = more important)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<u8>,
    /// High-level topic classification
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    /// Actual code content (for snippet kind)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Author or source attribution
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Additional custom metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty", flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A memory entry with rich metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Unique identifier (UUID)
    pub id: String,
    /// Entry key (user-provided or auto-generated)
    pub key: String,
    /// The information/content stored (natural language description)
    #[serde(alias = "value")]
    pub information: String,
    /// Entry type/category (legacy field, use metadata.kind instead)
    #[serde(default)]
    pub entry_type: String,
    /// Creation timestamp (ISO 8601)
    pub created_at: String,
    /// Last update timestamp (ISO 8601)
    pub updated_at: String,
    /// Rich metadata
    #[serde(default)]
    pub metadata: MemoryMetadata,
}

impl MemoryEntry {
    /// Get the value (alias for information for backwards compatibility)
    pub fn value(&self) -> &str {
        &self.information
    }
}

/// Memory store.
#[derive(Debug, Default, Serialize, Deserialize)]
struct MemoryStore {
    entries: HashMap<String, MemoryEntry>,
}

/// Search/filter options for memory_find.
#[derive(Debug, Clone, Default)]
pub struct MemorySearchOptions {
    /// Filter by entry kind
    pub kind: Option<MemoryKind>,
    /// Filter by programming language
    pub language: Option<String>,
    /// Filter by topic
    pub topic: Option<String>,
    /// Filter by tags (any match)
    pub tags: Option<Vec<String>>,
    /// Minimum priority threshold (1-10)
    pub priority_min: Option<u8>,
    /// Maximum number of results
    pub limit: Option<usize>,
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

    /// Store a memory entry (legacy API for backwards compatibility).
    pub async fn store(
        &self,
        key: String,
        value: String,
        entry_type: Option<String>,
    ) -> Result<MemoryEntry> {
        let metadata = MemoryMetadata {
            kind: entry_type
                .as_ref()
                .and_then(|t| t.parse().ok())
                .unwrap_or_default(),
            ..Default::default()
        };
        self.store_with_metadata(Some(key), value, metadata).await
    }

    /// Store a memory entry with rich metadata (m1rl0k/Context-Engine compatible).
    ///
    /// # Arguments
    /// * `key` - Optional key; if None, a UUID will be generated
    /// * `information` - The content to store (natural language description)
    /// * `metadata` - Rich metadata including kind, language, tags, priority, etc.
    pub async fn store_with_metadata(
        &self,
        key: Option<String>,
        information: String,
        metadata: MemoryMetadata,
    ) -> Result<MemoryEntry> {
        let now = chrono::Utc::now().to_rfc3339();
        let id = Uuid::new_v4().to_string();
        let key = key.unwrap_or_else(|| id.clone());

        let entry = MemoryEntry {
            id: id.clone(),
            key: key.clone(),
            information,
            entry_type: metadata.kind.to_string(),
            created_at: now.clone(),
            updated_at: now,
            metadata,
        };

        {
            let mut store = self.store.write().await;
            store.entries.insert(id, entry.clone());
        }

        self.save().await?;
        info!("Stored memory entry: {} (id: {})", entry.key, entry.id);

        Ok(entry)
    }

    /// Retrieve a memory entry by key or id.
    pub async fn retrieve(&self, key: &str) -> Option<MemoryEntry> {
        let store = self.store.read().await;
        // Try by id first, then by key
        store
            .entries
            .get(key)
            .cloned()
            .or_else(|| store.entries.values().find(|e| e.key == key).cloned())
    }

    /// List all memory entries.
    pub async fn list(&self, entry_type: Option<&str>) -> Vec<MemoryEntry> {
        let store = self.store.read().await;

        store
            .entries
            .values()
            .filter(|e| entry_type.is_none_or(|t| e.entry_type == t))
            .cloned()
            .collect()
    }

    /// Delete a memory entry by key or id.
    pub async fn delete(&self, key: &str) -> Result<bool> {
        let removed = {
            let mut store = self.store.write().await;
            // Try by id first
            if store.entries.remove(key).is_some() {
                true
            } else {
                // Try by key
                let id_to_remove = store
                    .entries
                    .iter()
                    .find(|(_, e)| e.key == key)
                    .map(|(id, _)| id.clone());
                if let Some(id) = id_to_remove {
                    store.entries.remove(&id).is_some()
                } else {
                    false
                }
            }
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

    /// Search memory entries by query text (legacy API).
    pub async fn search(&self, query: &str) -> Vec<MemoryEntry> {
        self.find(query, MemorySearchOptions::default()).await
    }

    /// Find memory entries with hybrid search and filtering (m1rl0k/Context-Engine compatible).
    ///
    /// Performs text matching on information, key, tags, and topic fields,
    /// then applies metadata filters and returns results sorted by relevance.
    pub async fn find(&self, query: &str, options: MemorySearchOptions) -> Vec<MemoryEntry> {
        let store = self.store.read().await;
        let query_lower = query.to_lowercase();
        let query_tokens: Vec<&str> = query_lower.split_whitespace().collect();

        let mut results: Vec<(MemoryEntry, f64)> = store
            .entries
            .values()
            .filter(|e| {
                // Apply metadata filters
                if let Some(ref kind) = options.kind {
                    if &e.metadata.kind != kind {
                        return false;
                    }
                }
                if let Some(ref lang) = options.language {
                    if e.metadata.language.as_ref() != Some(lang) {
                        return false;
                    }
                }
                if let Some(ref topic) = options.topic {
                    if e.metadata.topic.as_ref() != Some(topic) {
                        return false;
                    }
                }
                if let Some(ref tags) = options.tags {
                    // Any tag match
                    if !tags.iter().any(|t| e.metadata.tags.contains(t)) {
                        return false;
                    }
                }
                if let Some(min_priority) = options.priority_min {
                    if e.metadata.priority.unwrap_or(0) < min_priority {
                        return false;
                    }
                }
                true
            })
            .map(|e| {
                // Calculate relevance score
                let mut score = 0.0;
                let info_lower = e.information.to_lowercase();
                let key_lower = e.key.to_lowercase();

                // Exact match bonus
                if info_lower.contains(&query_lower) {
                    score += 1.0;
                }
                if key_lower.contains(&query_lower) {
                    score += 0.5;
                }

                // Token matching
                for token in &query_tokens {
                    if info_lower.contains(token) {
                        score += 0.3;
                    }
                    if key_lower.contains(token) {
                        score += 0.2;
                    }
                    // Tag matching
                    if e.metadata
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(token))
                    {
                        score += 0.4;
                    }
                    // Topic matching
                    if let Some(ref topic) = e.metadata.topic {
                        if topic.to_lowercase().contains(token) {
                            score += 0.3;
                        }
                    }
                }

                // Priority boost
                if let Some(priority) = e.metadata.priority {
                    score += (priority as f64) * 0.05;
                }

                (e.clone(), score)
            })
            .filter(|(_, score)| *score > 0.0)
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Apply limit
        let limit = options.limit.unwrap_or(10);
        results.into_iter().take(limit).map(|(e, _)| e).collect()
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

        let entry = service
            .store(
                "test-key".to_string(),
                "test-value".to_string(),
                Some("snippet".to_string()),
            )
            .await
            .unwrap();

        assert_eq!(entry.key, "test-key");
        assert_eq!(entry.information, "test-value");
        assert_eq!(entry.entry_type, "snippet");

        let retrieved = service.retrieve("test-key").await.unwrap();
        assert_eq!(retrieved.information, "test-value");
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

        service
            .store("key1".to_string(), "value1".to_string(), None)
            .await
            .unwrap();
        service
            .store("key2".to_string(), "value2".to_string(), None)
            .await
            .unwrap();

        let all = service.list(None).await;
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_list_by_type() {
        let (service, _temp) = create_test_service().await;

        service
            .store(
                "key1".to_string(),
                "value1".to_string(),
                Some("snippet".to_string()),
            )
            .await
            .unwrap();
        service
            .store(
                "key2".to_string(),
                "value2".to_string(),
                Some("pattern".to_string()),
            )
            .await
            .unwrap();
        service
            .store(
                "key3".to_string(),
                "value3".to_string(),
                Some("snippet".to_string()),
            )
            .await
            .unwrap();

        let type_a = service.list(Some("snippet")).await;
        assert_eq!(type_a.len(), 2);

        let type_b = service.list(Some("pattern")).await;
        assert_eq!(type_b.len(), 1);
    }

    #[tokio::test]
    async fn test_delete() {
        let (service, _temp) = create_test_service().await;

        service
            .store("to-delete".to_string(), "value".to_string(), None)
            .await
            .unwrap();
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

        service
            .store("key1".to_string(), "value1".to_string(), None)
            .await
            .unwrap();
        service
            .store("key2".to_string(), "value2".to_string(), None)
            .await
            .unwrap();

        let cleared = service.clear().await.unwrap();
        assert_eq!(cleared, 2);

        let all = service.list(None).await;
        assert!(all.is_empty());
    }

    #[tokio::test]
    async fn test_search() {
        let (service, _temp) = create_test_service().await;

        service
            .store(
                "config".to_string(),
                "database connection string".to_string(),
                None,
            )
            .await
            .unwrap();
        service
            .store(
                "note".to_string(),
                "remember to check database".to_string(),
                None,
            )
            .await
            .unwrap();
        service
            .store("other".to_string(), "unrelated content".to_string(), None)
            .await
            .unwrap();

        let results = service.search("database").await;
        assert_eq!(results.len(), 2);

        let results = service.search("CONFIG").await; // Case insensitive
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_default_entry_type() {
        let (service, _temp) = create_test_service().await;

        let entry = service
            .store("key".to_string(), "value".to_string(), None)
            .await
            .unwrap();
        assert_eq!(entry.entry_type, "memory");
    }

    #[tokio::test]
    async fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();

        // Create and populate
        {
            let service = MemoryService::new(temp_dir.path()).await.unwrap();
            service
                .store("persistent".to_string(), "data".to_string(), None)
                .await
                .unwrap();
        }

        // Reload and verify
        {
            let service = MemoryService::new(temp_dir.path()).await.unwrap();
            let entry = service.retrieve("persistent").await;
            assert!(entry.is_some());
            assert_eq!(entry.unwrap().information, "data");
        }
    }

    #[test]
    fn test_memory_entry_serialization() {
        let entry = MemoryEntry {
            id: "test-id".to_string(),
            key: "test".to_string(),
            information: "value".to_string(),
            entry_type: "memory".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            metadata: MemoryMetadata::default(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: MemoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.key, entry.key);
    }

    // New tests for rich metadata features

    #[tokio::test]
    async fn test_store_with_rich_metadata() {
        let (service, _temp) = create_test_service().await;

        let metadata = MemoryMetadata {
            kind: MemoryKind::Pattern,
            language: Some("python".to_string()),
            path: Some("utils/file_processor.py".to_string()),
            tags: vec!["python".to_string(), "generators".to_string(), "performance".to_string()],
            priority: Some(8),
            topic: Some("performance optimization".to_string()),
            code: Some("def process_large_file(file_path):\n    with open(file_path) as f:\n        for line in f:\n            yield process_line(line)".to_string()),
            author: Some("developer".to_string()),
            extra: HashMap::new(),
        };

        let entry = service
            .store_with_metadata(
                Some("python-generator-pattern".to_string()),
                "Efficient Python pattern for processing large files using generators".to_string(),
                metadata,
            )
            .await
            .unwrap();

        assert_eq!(entry.key, "python-generator-pattern");
        assert_eq!(entry.metadata.kind, MemoryKind::Pattern);
        assert_eq!(entry.metadata.language, Some("python".to_string()));
        assert_eq!(entry.metadata.priority, Some(8));
        assert_eq!(entry.metadata.tags.len(), 3);
    }

    #[tokio::test]
    async fn test_find_with_filters() {
        let (service, _temp) = create_test_service().await;

        // Store entries with different metadata
        service
            .store_with_metadata(
                Some("py-pattern-1".to_string()),
                "Python async pattern".to_string(),
                MemoryMetadata {
                    kind: MemoryKind::Pattern,
                    language: Some("python".to_string()),
                    tags: vec!["async".to_string()],
                    priority: Some(8),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        service
            .store_with_metadata(
                Some("rs-pattern-1".to_string()),
                "Rust async pattern".to_string(),
                MemoryMetadata {
                    kind: MemoryKind::Pattern,
                    language: Some("rust".to_string()),
                    tags: vec!["async".to_string()],
                    priority: Some(9),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        service
            .store_with_metadata(
                Some("py-snippet-1".to_string()),
                "Python code snippet".to_string(),
                MemoryMetadata {
                    kind: MemoryKind::Snippet,
                    language: Some("python".to_string()),
                    priority: Some(5),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        // Find by language and kind
        let results = service
            .find(
                "pattern",
                MemorySearchOptions {
                    language: Some("python".to_string()),
                    kind: Some(MemoryKind::Pattern),
                    ..Default::default()
                },
            )
            .await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "py-pattern-1");

        // Find by kind
        let results = service
            .find(
                "pattern",
                MemorySearchOptions {
                    kind: Some(MemoryKind::Pattern),
                    ..Default::default()
                },
            )
            .await;
        assert_eq!(results.len(), 2);

        // Find by tags
        let results = service
            .find(
                "async",
                MemorySearchOptions {
                    tags: Some(vec!["async".to_string()]),
                    ..Default::default()
                },
            )
            .await;
        assert_eq!(results.len(), 2);

        // Find by priority
        let results = service
            .find(
                "pattern",
                MemorySearchOptions {
                    priority_min: Some(8),
                    ..Default::default()
                },
            )
            .await;
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_find_with_limit() {
        let (service, _temp) = create_test_service().await;

        for i in 0..10 {
            service
                .store(format!("key-{}", i), format!("test value {}", i), None)
                .await
                .unwrap();
        }

        let results = service
            .find(
                "test",
                MemorySearchOptions {
                    limit: Some(5),
                    ..Default::default()
                },
            )
            .await;
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_memory_kind_parsing() {
        assert_eq!(
            "snippet".parse::<MemoryKind>().unwrap(),
            MemoryKind::Snippet
        );
        assert_eq!(
            "pattern".parse::<MemoryKind>().unwrap(),
            MemoryKind::Pattern
        );
        assert_eq!(
            "explanation".parse::<MemoryKind>().unwrap(),
            MemoryKind::Explanation
        );
        assert_eq!(
            "example".parse::<MemoryKind>().unwrap(),
            MemoryKind::Example
        );
        assert_eq!(
            "reference".parse::<MemoryKind>().unwrap(),
            MemoryKind::Reference
        );
        assert_eq!("memory".parse::<MemoryKind>().unwrap(), MemoryKind::Memory);
        assert_eq!("general".parse::<MemoryKind>().unwrap(), MemoryKind::Memory);
        assert_eq!("unknown".parse::<MemoryKind>().unwrap(), MemoryKind::Memory);
    }
}
