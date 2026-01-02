//! Memory tools for persistent storage.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::Result;
use crate::mcp::handler::{error_result, get_optional_string_arg, get_string_arg, success_result, ToolHandler};
use crate::mcp::protocol::{Tool, ToolResult};
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
            description: "Store a piece of information in persistent memory for later retrieval.".to_string(),
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

