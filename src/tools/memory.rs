//! Memory tools for persistent storage.
//!
//! This module provides memory tools compatible with m1rl0k/Context-Engine:
//! - `memory_store`: Store memories with rich metadata (kind, language, tags, priority, etc.)
//! - `memory_find`: Hybrid search with metadata filtering
//! - Legacy tools: `add_memory`, `retrieve-memory`, `list_memories`, `delete-memory`

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::Result;
use crate::mcp::handler::{
    error_result, get_optional_string_arg, get_string_arg, success_result, ToolHandler,
};
use crate::mcp::protocol::{Tool, ToolResult};
use crate::service::memory::{MemoryKind, MemoryMetadata, MemorySearchOptions};
use crate::service::MemoryService;

/// Store memory tool.
pub struct StoreMemoryTool {
    service: Arc<MemoryService>,
}

impl StoreMemoryTool {
    pub fn new(service: Arc<MemoryService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for StoreMemoryTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "add_memory".to_string(),
            description: "Store a piece of information in persistent memory for later retrieval."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "Unique key to identify this memory"
                    },
                    "value": {
                        "type": "string",
                        "description": "The information to store"
                    },
                    "type": {
                        "type": "string",
                        "description": "Optional category/type for the memory"
                    }
                },
                "required": ["key", "value"]
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let key = get_string_arg(&args, "key")?;
        let value = get_string_arg(&args, "value")?;
        let entry_type = get_optional_string_arg(&args, "type");

        match self.service.store(key, value, entry_type).await {
            Ok(entry) => Ok(success_result(format!("Stored memory: {}", entry.key))),
            Err(e) => Ok(error_result(format!("Failed to store memory: {}", e))),
        }
    }
}

/// Retrieve memory tool.
pub struct RetrieveMemoryTool {
    service: Arc<MemoryService>,
}

impl RetrieveMemoryTool {
    pub fn new(service: Arc<MemoryService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for RetrieveMemoryTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "retrieve-memory".to_string(),
            description: "Retrieve a previously stored memory by its key.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "The key of the memory to retrieve"
                    }
                },
                "required": ["key"]
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let key = get_string_arg(&args, "key")?;

        match self.service.retrieve(&key).await {
            Some(entry) => {
                let json = serde_json::to_string_pretty(&entry)?;
                Ok(success_result(json))
            }
            None => Ok(error_result(format!("Memory not found: {}", key))),
        }
    }
}

/// List memory tool.
pub struct ListMemoryTool {
    service: Arc<MemoryService>,
}

impl ListMemoryTool {
    pub fn new(service: Arc<MemoryService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for ListMemoryTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "list_memories".to_string(),
            description: "List all stored memories, optionally filtered by type.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "type": {
                        "type": "string",
                        "description": "Optional type to filter memories"
                    }
                },
                "required": []
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let entry_type = get_optional_string_arg(&args, "type");
        let entries = self.service.list(entry_type.as_deref()).await;
        let json = serde_json::to_string_pretty(&entries)?;
        Ok(success_result(json))
    }
}

/// Delete memory tool.
pub struct DeleteMemoryTool {
    service: Arc<MemoryService>,
}

impl DeleteMemoryTool {
    pub fn new(service: Arc<MemoryService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for DeleteMemoryTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "delete-memory".to_string(),
            description: "Delete a stored memory by its key.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "The key of the memory to delete"
                    }
                },
                "required": ["key"]
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let key = get_string_arg(&args, "key")?;

        match self.service.delete(&key).await {
            Ok(true) => Ok(success_result(format!("Deleted memory: {}", key))),
            Ok(false) => Ok(error_result(format!("Memory not found: {}", key))),
            Err(e) => Ok(error_result(format!("Failed to delete memory: {}", e))),
        }
    }
}

// ============================================================================
// New m1rl0k/Context-Engine compatible tools
// ============================================================================

/// Memory store tool with rich metadata (m1rl0k/Context-Engine compatible).
pub struct MemoryStoreTool {
    service: Arc<MemoryService>,
}

impl MemoryStoreTool {
    pub fn new(service: Arc<MemoryService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for MemoryStoreTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "memory_store".to_string(),
            description: "Store information in persistent memory with rich metadata for later retrieval. \
                Supports categorization by kind (snippet, explanation, pattern, example, reference), \
                programming language, tags, priority, and more.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "information": {
                        "type": "string",
                        "description": "The information to store (natural language description)"
                    },
                    "key": {
                        "type": "string",
                        "description": "Optional unique key; if not provided, a UUID will be generated"
                    },
                    "kind": {
                        "type": "string",
                        "enum": ["snippet", "explanation", "pattern", "example", "reference", "memory"],
                        "description": "Category type for the memory"
                    },
                    "language": {
                        "type": "string",
                        "description": "Programming language (e.g., 'python', 'rust', 'javascript')"
                    },
                    "path": {
                        "type": "string",
                        "description": "File path context for code-related entries"
                    },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Searchable tags for categorization"
                    },
                    "priority": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 10,
                        "description": "Importance ranking (1-10, higher = more important)"
                    },
                    "topic": {
                        "type": "string",
                        "description": "High-level topic classification"
                    },
                    "code": {
                        "type": "string",
                        "description": "Actual code content (for snippet kind)"
                    },
                    "author": {
                        "type": "string",
                        "description": "Author or source attribution"
                    }
                },
                "required": ["information"]
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let information = get_string_arg(&args, "information")?;
        let key = get_optional_string_arg(&args, "key");

        // Parse kind
        let kind = get_optional_string_arg(&args, "kind")
            .and_then(|k| k.parse().ok())
            .unwrap_or_default();

        // Parse tags
        let tags = args
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        // Parse priority
        let priority = args
            .get("priority")
            .and_then(|v| v.as_u64())
            .map(|p| p.min(10) as u8);

        let metadata = MemoryMetadata {
            kind,
            language: get_optional_string_arg(&args, "language"),
            path: get_optional_string_arg(&args, "path"),
            tags,
            priority,
            topic: get_optional_string_arg(&args, "topic"),
            code: get_optional_string_arg(&args, "code"),
            author: get_optional_string_arg(&args, "author"),
            extra: HashMap::new(),
        };

        match self
            .service
            .store_with_metadata(key, information, metadata)
            .await
        {
            Ok(entry) => {
                let response = serde_json::json!({
                    "success": true,
                    "id": entry.id,
                    "key": entry.key,
                    "message": format!("Stored memory: {} (id: {})", entry.key, entry.id)
                });
                Ok(success_result(serde_json::to_string_pretty(&response)?))
            }
            Err(e) => Ok(error_result(format!("Failed to store memory: {}", e))),
        }
    }
}

/// Memory find tool with hybrid search and filtering (m1rl0k/Context-Engine compatible).
pub struct MemoryFindTool {
    service: Arc<MemoryService>,
}

impl MemoryFindTool {
    pub fn new(service: Arc<MemoryService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for MemoryFindTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "memory_find".to_string(),
            description: "Search for memories using hybrid text matching and metadata filtering. \
                Returns results sorted by relevance with priority boosting."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query text"
                    },
                    "kind": {
                        "type": "string",
                        "enum": ["snippet", "explanation", "pattern", "example", "reference", "memory"],
                        "description": "Filter by memory kind"
                    },
                    "language": {
                        "type": "string",
                        "description": "Filter by programming language"
                    },
                    "topic": {
                        "type": "string",
                        "description": "Filter by topic"
                    },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Filter by tags (any match)"
                    },
                    "priority_min": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 10,
                        "description": "Minimum priority threshold"
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 100,
                        "description": "Maximum number of results (default: 10)"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let query = get_string_arg(&args, "query")?;

        // Parse kind filter
        let kind =
            get_optional_string_arg(&args, "kind").and_then(|k| k.parse::<MemoryKind>().ok());

        // Parse tags filter
        let tags = args.get("tags").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });

        // Parse priority_min
        let priority_min = args
            .get("priority_min")
            .and_then(|v| v.as_u64())
            .map(|p| p.min(10) as u8);

        // Parse limit
        let limit = args
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|l| l.min(100) as usize);

        let options = MemorySearchOptions {
            kind,
            language: get_optional_string_arg(&args, "language"),
            topic: get_optional_string_arg(&args, "topic"),
            tags,
            priority_min,
            limit,
        };

        let results = self.service.find(&query, options).await;

        let response = serde_json::json!({
            "query": query,
            "count": results.len(),
            "results": results
        });

        Ok(success_result(serde_json::to_string_pretty(&response)?))
    }
}
