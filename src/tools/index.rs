//! Index management tools.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::error::Result;
use crate::mcp::handler::{error_result, success_result, ToolHandler};
use crate::mcp::protocol::{Tool, ToolResult};
use crate::service::ContextService;

/// Index workspace tool.
pub struct IndexWorkspaceTool {
    service: Arc<ContextService>,
}

impl IndexWorkspaceTool {
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for IndexWorkspaceTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "index_workspace".to_string(),
            description: r#"Index the current workspace for semantic search.

This tool scans all source files in the workspace and builds a semantic index
that enables fast, meaning-based code search.

**When to use this tool:**
- First time using the context engine with a new project
- After making significant changes to the codebase
- When semantic_search or enhance_prompt returns no results

**What gets indexed (50+ file types):**
- TypeScript/JavaScript (.ts, .tsx, .js, .jsx, .mjs, .cjs)
- Python (.py, .pyi)
- Go (.go), Rust (.rs), Java (.java), Kotlin (.kt)
- C/C++ (.c, .cpp, .h, .hpp), Swift (.swift)
- Web (.vue, .svelte, .astro, .html, .css, .scss)
- Config (.json, .yaml, .yml, .toml, .xml)
- Documentation (.md, .txt)"#
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "force": {
                        "type": "boolean",
                        "description": "Force re-indexing even if an index already exists (default: false)"
                    },
                    "background": {
                        "type": "boolean",
                        "description": "Run indexing in a background worker thread (non-blocking)"
                    }
                },
                "required": []
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
        let background = args
            .get("background")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let start = Instant::now();

        if background {
            // Fire and forget
            let service = self.service.clone();
            tokio::spawn(async move {
                if let Err(e) = service.index_workspace().await {
                    tracing::error!("Background indexing failed: {}", e);
                }
            });

            let result = serde_json::json!({
                "success": true,
                "message": "Background indexing started"
            });
            return Ok(success_result(serde_json::to_string_pretty(&result)?));
        }

        // Clear if forced
        if force {
            self.service.clear().await;
        }

        match self.service.index_workspace().await {
            Ok(stats) => {
                let elapsed = start.elapsed().as_millis();
                let result = serde_json::json!({
                    "success": true,
                    "message": format!("Workspace indexed successfully in {}ms", elapsed),
                    "elapsed_ms": elapsed,
                    "indexed": stats.indexed,
                    "skipped": stats.skipped,
                    "errors": stats.errors
                });
                Ok(success_result(serde_json::to_string_pretty(&result)?))
            }
            Err(e) => Ok(error_result(format!("Failed to index workspace: {}", e))),
        }
    }
}

/// Index status tool.
pub struct IndexStatusTool {
    service: Arc<ContextService>,
}

impl IndexStatusTool {
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for IndexStatusTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "index_status".to_string(),
            description: "Get the current status of the codebase index, including file count and last indexed time.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn execute(&self, _args: HashMap<String, Value>) -> Result<ToolResult> {
        let status = self.service.status().await;
        let json = serde_json::to_string_pretty(&status)?;
        Ok(success_result(json))
    }
}

/// Reindex workspace tool.
pub struct ReindexWorkspaceTool {
    service: Arc<ContextService>,
}

impl ReindexWorkspaceTool {
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for ReindexWorkspaceTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "reindex_workspace".to_string(),
            description: "Clear current index state and rebuild it from scratch.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn execute(&self, _args: HashMap<String, Value>) -> Result<ToolResult> {
        let start = Instant::now();

        // Clear the index first
        self.service.clear().await;

        // Re-index
        match self.service.index_workspace().await {
            Ok(stats) => {
                let elapsed = start.elapsed().as_millis();
                let result = serde_json::json!({
                    "success": true,
                    "message": "Workspace reindexed successfully",
                    "elapsed_ms": elapsed,
                    "indexed": stats.indexed,
                    "skipped": stats.skipped,
                    "errors": stats.errors
                });
                Ok(success_result(serde_json::to_string_pretty(&result)?))
            }
            Err(e) => Ok(error_result(format!("Failed to reindex workspace: {}", e))),
        }
    }
}

/// Clear index tool.
pub struct ClearIndexTool {
    service: Arc<ContextService>,
}

impl ClearIndexTool {
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for ClearIndexTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "clear_index".to_string(),
            description: "Remove saved index state and clear caches without rebuilding."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn execute(&self, _args: HashMap<String, Value>) -> Result<ToolResult> {
        self.service.clear().await;

        let result = serde_json::json!({
            "success": true,
            "message": "Index cleared. Re-run index_workspace to rebuild."
        });
        Ok(success_result(serde_json::to_string_pretty(&result)?))
    }
}

/// Refresh index tool (legacy alias for index_workspace with force=true).
pub struct RefreshIndexTool {
    service: Arc<ContextService>,
}

impl RefreshIndexTool {
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for RefreshIndexTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "refresh_index".to_string(),
            description: "Refresh the codebase index by re-scanning all files. Use this after making significant changes to the codebase.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "force": {
                        "type": "boolean",
                        "description": "Force full re-index even if files haven't changed"
                    }
                },
                "required": []
            }),
        }
    }

    async fn execute(&self, _args: HashMap<String, Value>) -> Result<ToolResult> {
        // Initialize the service (which triggers indexing)
        match self.service.initialize().await {
            Ok(_) => {
                let status = self.service.status().await;
                let json = serde_json::to_string_pretty(&status)?;
                Ok(success_result(format!(
                    "Index refreshed successfully.\n{}",
                    json
                )))
            }
            Err(e) => Ok(error_result(format!("Failed to refresh index: {}", e))),
        }
    }
}
