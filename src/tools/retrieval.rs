//! Retrieval tools for codebase search.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::error::Result;
use crate::mcp::handler::{
    error_result, get_optional_string_arg, get_string_arg, success_result, ToolHandler,
};
use crate::mcp::protocol::{Tool, ToolResult};
use crate::service::ContextService;

/// Get syntax highlighting language for a file extension.
fn get_language_for_extension(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        "ts" => "typescript",
        "tsx" => "tsx",
        "js" => "javascript",
        "jsx" => "jsx",
        "py" => "python",
        "rb" => "ruby",
        "go" => "go",
        "rs" => "rust",
        "java" => "java",
        "kt" => "kotlin",
        "cs" => "csharp",
        "cpp" | "cc" | "cxx" => "cpp",
        "c" | "h" => "c",
        "hpp" => "cpp",
        "swift" => "swift",
        "php" => "php",
        "sql" => "sql",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "md" => "markdown",
        "html" => "html",
        "css" => "css",
        "scss" => "scss",
        "vue" => "vue",
        "svelte" => "svelte",
        "toml" => "toml",
        "xml" => "xml",
        "sh" | "bash" => "bash",
        "ps1" => "powershell",
        _ => "",
    }
}

/// Format file size in human-readable format.
fn format_file_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} bytes", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Codebase retrieval tool - semantic search.
pub struct CodebaseRetrievalTool {
    service: Arc<ContextService>,
}

impl CodebaseRetrievalTool {
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for CodebaseRetrievalTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "codebase_retrieval".to_string(),
            description: "Search the codebase using natural language. Returns relevant code snippets and context based on semantic understanding of the query.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "information_request": {
                        "type": "string",
                        "description": "Natural language description of what you're looking for"
                    },
                    "max_tokens": {
                        "type": "integer",
                        "description": "Maximum tokens in the response (optional)"
                    }
                },
                "required": ["information_request"]
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let query = get_string_arg(&args, "information_request")?;
        let max_tokens = args
            .get("max_tokens")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        match self.service.search(&query, max_tokens).await {
            Ok(result) => Ok(success_result(result)),
            Err(e) => Ok(error_result(format!("Search failed: {}", e))),
        }
    }
}

/// Search code tool - keyword and semantic search.
pub struct SearchCodeTool {
    service: Arc<ContextService>,
}

impl SearchCodeTool {
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for SearchCodeTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "semantic_search".to_string(),
            description:
                "Search for code patterns, functions, classes, or specific text in the codebase."
                    .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query (can be natural language or code pattern)"
                    },
                    "file_pattern": {
                        "type": "string",
                        "description": "Optional glob pattern to filter files (e.g., '*.rs', 'src/**/*.ts')"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results to return"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let query = get_string_arg(&args, "query")?;
        let _file_pattern = get_optional_string_arg(&args, "file_pattern");
        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        // Use semantic search with optional token limit
        let max_tokens = max_results.map(|r| r * 500); // Rough estimate

        match self.service.search(&query, max_tokens).await {
            Ok(result) => Ok(success_result(result)),
            Err(e) => Ok(error_result(format!("Search failed: {}", e))),
        }
    }
}

/// Get file tool - retrieve file contents.
pub struct GetFileTool {
    service: Arc<ContextService>,
}

impl GetFileTool {
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for GetFileTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "get_file".to_string(),
            description: "Retrieve complete or partial contents of a file from the codebase."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path relative to workspace root"
                    },
                    "start_line": {
                        "type": "integer",
                        "description": "Optional: First line to include (1-based)"
                    },
                    "end_line": {
                        "type": "integer",
                        "description": "Optional: Last line to include (1-based)"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let path = get_string_arg(&args, "path")?;
        let start_line = args
            .get("start_line")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        let end_line = args
            .get("end_line")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        // Validate path
        if path.is_empty() {
            return Ok(error_result(
                "Invalid path: must be a non-empty string".to_string(),
            ));
        }
        if path.len() > 500 {
            return Ok(error_result(
                "Path too long: maximum 500 characters".to_string(),
            ));
        }

        // Read file
        let full_path = self.service.workspace_path().join(&path);
        let content = match tokio::fs::read_to_string(&full_path).await {
            Ok(c) => c,
            Err(e) => return Ok(error_result(format!("Failed to read file: {}", e))),
        };

        let all_lines: Vec<&str> = content.lines().collect();
        let total_lines = all_lines.len();
        let size = content.len();
        let ext = Path::new(&path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let language = get_language_for_extension(ext);
        let filename = Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&path);

        // Handle line range
        let (output_content, line_info) = if start_line.is_some() || end_line.is_some() {
            let start = start_line.unwrap_or(1).saturating_sub(1);
            let end = end_line.unwrap_or(total_lines);

            if start >= total_lines {
                return Ok(error_result(format!(
                    "start_line {} exceeds file length ({} lines)",
                    start + 1,
                    total_lines
                )));
            }

            let selected: Vec<&str> = all_lines[start..end.min(total_lines)].to_vec();
            let line_info = format!(
                "Lines {}-{} of {}",
                start + 1,
                end.min(total_lines),
                total_lines
            );
            (selected.join("\n"), line_info)
        } else {
            (content.clone(), format!("{} lines", total_lines))
        };

        // Format output
        let mut output = format!("# 📄 File: `{}`\n\n", filename);
        output.push_str("| Property | Value |\n");
        output.push_str("|----------|-------|\n");
        output.push_str(&format!("| **Path** | `{}` |\n", path));
        output.push_str(&format!("| **Lines** | {} |\n", line_info));
        output.push_str(&format!("| **Size** | {} |\n", format_file_size(size)));
        output.push_str(&format!("| **Type** | .{} |\n", ext));
        output.push_str("\n## Content\n\n");
        output.push_str(&format!("```{}\n", language));
        output.push_str(&output_content);
        if !output_content.ends_with('\n') {
            output.push('\n');
        }
        output.push_str("```\n");

        Ok(success_result(output))
    }
}

/// Get context for prompt tool.
pub struct GetContextTool {
    service: Arc<ContextService>,
}

impl GetContextTool {
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for GetContextTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "get_context_for_prompt".to_string(),
            description: "Get relevant codebase context optimized for prompt enhancement. Returns file summaries, code snippets, and related memories.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Description of what you need context for"
                    },
                    "max_files": {
                        "type": "integer",
                        "description": "Maximum number of files to include (default: 5, max: 20)"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum tokens for the entire context (default: 8000)"
                    },
                    "include_related": {
                        "type": "boolean",
                        "description": "Include related/imported files (default: true)"
                    },
                    "min_relevance": {
                        "type": "number",
                        "description": "Minimum relevance score (0-1) to include a file (default: 0.3)"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let query = get_string_arg(&args, "query")?;
        let max_files = args.get("max_files").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
        let token_budget = args
            .get("token_budget")
            .and_then(|v| v.as_u64())
            .unwrap_or(8000) as usize;

        // Perform semantic search
        match self.service.search(&query, Some(token_budget)).await {
            Ok(result) => {
                let mut output = format!("# 📚 Codebase Context\n\n");
                output.push_str(&format!("**Query:** \"{}\"\n\n", query));
                output.push_str(&format!(
                    "**Settings:** max_files={}, token_budget={}\n\n",
                    max_files, token_budget
                ));
                output.push_str("## Results\n\n");
                output.push_str(&result);
                Ok(success_result(output))
            }
            Err(e) => Ok(error_result(format!("Context retrieval failed: {}", e))),
        }
    }
}

/// Enhance prompt tool - AI-powered prompt enhancement.
pub struct EnhancePromptTool {
    service: Arc<ContextService>,
}

impl EnhancePromptTool {
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for EnhancePromptTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "enhance_prompt".to_string(),
            description: "Transform a simple prompt into a detailed, structured prompt with codebase context using AI-powered enhancement.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "prompt": {
                        "type": "string",
                        "description": "The simple prompt to enhance"
                    }
                },
                "required": ["prompt"]
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let prompt = get_string_arg(&args, "prompt")?;

        if prompt.len() > 10000 {
            return Ok(error_result(
                "Prompt too long: maximum 10000 characters".to_string(),
            ));
        }

        // Use AI to enhance the prompt with codebase context
        match self.service.enhance_prompt(&prompt).await {
            Ok(enhanced) => Ok(success_result(enhanced)),
            Err(e) => Ok(error_result(format!("Prompt enhancement failed: {}", e))),
        }
    }
}

/// Tool manifest tool - discover available capabilities.
pub struct ToolManifestTool;

impl ToolManifestTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ToolManifestTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for ToolManifestTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "tool_manifest".to_string(),
            description: "Discover available tools and capabilities exposed by the server."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn execute(&self, _args: HashMap<String, Value>) -> Result<ToolResult> {
        let manifest = serde_json::json!({
            "version": "1.9.0",
            "capabilities": [
                "semantic_search",
                "file_retrieval",
                "context_enhancement",
                "index_status",
                "lifecycle",
                "planning",
                "code_review",
                "memory"
            ],
            "tools": [
                "index_workspace",
                "codebase_retrieval",
                "semantic_search",
                "get_file",
                "get_context_for_prompt",
                "enhance_prompt",
                "index_status",
                "reindex_workspace",
                "clear_index",
                "tool_manifest",
                "create_plan",
                "refine_plan",
                "visualize_plan",
                "execute_plan",
                "save_plan",
                "load_plan",
                "list_plans",
                "delete_plan",
                "request_approval",
                "respond_approval",
                "start_step",
                "complete_step",
                "fail_step",
                "view_progress",
                "view_history",
                "compare_plan_versions",
                "rollback_plan",
                "review_changes",
                "review_git_diff",
                "review_diff",
                "check_invariants",
                "run_static_analysis",
                "reactive_review_pr",
                "get_review_status",
                "pause_review",
                "resume_review",
                "get_review_telemetry",
                "scrub_secrets",
                "validate_content",
                "add_memory",
                "list_memories"
            ]
        });

        Ok(success_result(serde_json::to_string_pretty(&manifest)?))
    }
}
