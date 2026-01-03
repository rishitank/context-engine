//! Specialized search tools (m1rl0k/Context-Engine compatible).
//!
//! This module provides specialized search tools with preset file patterns:
//! - `search_tests_for`: Find test files related to a query
//! - `search_config_for`: Find configuration files related to a query
//! - `search_callers_for`: Find callers/usages of a symbol
//! - `search_importers_for`: Find files importing a module/symbol
//! - `info_request`: Simplified codebase retrieval with explanation mode
//! - `pattern_search`: Structural code pattern matching
//! - `context_search`: Context-aware semantic search

use async_trait::async_trait;
use glob::Pattern;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use walkdir::WalkDir;

use crate::error::Result;
use crate::mcp::handler::{get_optional_string_arg, get_string_arg, success_result, ToolHandler};
use crate::mcp::protocol::{Tool, ToolAnnotations, ToolResult};
use crate::service::ContextService;

/// Preset glob patterns for test files.
const TEST_GLOBS: &[&str] = &[
    "tests/**/*",
    "test/**/*",
    "**/*test*.*",
    "**/*_test.*",
    "**/*Test*.*",
    "**/*.test.*",
    "**/*.spec.*",
    "**/test_*.*",
    "**/__tests__/**/*",
];

/// Preset glob patterns for config files.
const CONFIG_GLOBS: &[&str] = &[
    "**/*.yaml",
    "**/*.yml",
    "**/*.json",
    "**/*.toml",
    "**/*.ini",
    "**/*.cfg",
    "**/*.conf",
    "**/*.config.*",
    "**/.env*",
    "**/config/**/*",
    "**/configs/**/*",
    "**/settings/**/*",
    "**/*config*.*",
    "**/*settings*.*",
];

// Helper function to search files matching patterns and containing query.
async fn search_files_with_patterns(
    workspace: &Path,
    query: &str,
    patterns: &[&str],
    limit: usize,
) -> Vec<serde_json::Value> {
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    // Compile glob patterns
    let compiled_patterns: Vec<Pattern> = patterns
        .iter()
        .filter_map(|p| Pattern::new(p).ok())
        .collect();

    for entry in WalkDir::new(workspace)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if results.len() >= limit {
            break;
        }

        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let relative_path = path.strip_prefix(workspace).unwrap_or(path);
        let path_str = relative_path.to_string_lossy();

        // Check if path matches any pattern
        let matches_pattern = compiled_patterns.iter().any(|p| p.matches(&path_str));
        if !matches_pattern {
            continue;
        }

        // Check if file contains query
        if let Ok(content) = fs::read_to_string(path).await {
            if content.to_lowercase().contains(&query_lower) {
                // Find matching lines
                let matching_lines: Vec<_> = content
                    .lines()
                    .enumerate()
                    .filter(|(_, line)| line.to_lowercase().contains(&query_lower))
                    .take(5)
                    .map(|(i, line)| {
                        serde_json::json!({
                            "line": i + 1,
                            "content": line.trim()
                        })
                    })
                    .collect();

                results.push(serde_json::json!({
                    "path": path_str,
                    "matches": matching_lines
                }));
            }
        }
    }

    results
}

/// Search tests for tool.
pub struct SearchTestsForTool {
    workspace: Arc<Path>,
}

impl SearchTestsForTool {
    pub fn new(workspace: &Path) -> Self {
        Self {
            workspace: Arc::from(workspace),
        }
    }
}

#[async_trait]
impl ToolHandler for SearchTestsForTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "search_tests_for".to_string(),
            description:
                "Search for test files related to a query. Uses preset test file patterns \
                (tests/**, *test*.*, *.spec.*, __tests__/**, etc.) to find relevant test files."
                    .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query (function name, class name, or keyword)"
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 50,
                        "description": "Maximum number of results (default: 10)"
                    }
                },
                "required": ["query"]
            }),
            annotations: Some(ToolAnnotations::read_only().with_title("Search Tests For")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let query = get_string_arg(&args, "query")?;
        let limit = args
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|l| l.min(50) as usize)
            .unwrap_or(10);

        let results = search_files_with_patterns(&self.workspace, &query, TEST_GLOBS, limit).await;

        let response = serde_json::json!({
            "query": query,
            "patterns": TEST_GLOBS,
            "count": results.len(),
            "results": results
        });

        Ok(success_result(serde_json::to_string_pretty(&response)?))
    }
}

/// Search config for tool.
pub struct SearchConfigForTool {
    workspace: Arc<Path>,
}

impl SearchConfigForTool {
    pub fn new(workspace: &Path) -> Self {
        Self {
            workspace: Arc::from(workspace),
        }
    }
}

#[async_trait]
impl ToolHandler for SearchConfigForTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "search_config_for".to_string(),
            description: "Search for configuration files related to a query. Uses preset config \
                patterns (*.yaml, *.json, *.toml, *.ini, .env*, config/**, etc.)."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query (setting name, config key, or keyword)"
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 50,
                        "description": "Maximum number of results (default: 10)"
                    }
                },
                "required": ["query"]
            }),
            annotations: Some(ToolAnnotations::read_only().with_title("Search Config For")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let query = get_string_arg(&args, "query")?;
        let limit = args
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|l| l.min(50) as usize)
            .unwrap_or(10);

        let results =
            search_files_with_patterns(&self.workspace, &query, CONFIG_GLOBS, limit).await;

        let response = serde_json::json!({
            "query": query,
            "patterns": CONFIG_GLOBS,
            "count": results.len(),
            "results": results
        });

        Ok(success_result(serde_json::to_string_pretty(&response)?))
    }
}

/// Search callers for tool.
pub struct SearchCallersForTool {
    workspace: Arc<Path>,
}

impl SearchCallersForTool {
    pub fn new(workspace: &Path) -> Self {
        Self {
            workspace: Arc::from(workspace),
        }
    }
}

#[async_trait]
impl ToolHandler for SearchCallersForTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "search_callers_for".to_string(),
            description: "Search for callers/usages of a symbol in the codebase. \
                Finds all locations where a function, method, or variable is called or referenced."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "The symbol name to find callers for"
                    },
                    "file_pattern": {
                        "type": "string",
                        "description": "Optional file pattern to limit search (e.g., '*.rs', '*.py')"
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 100,
                        "description": "Maximum number of results (default: 20)"
                    }
                },
                "required": ["symbol"]
            }),
            annotations: Some(ToolAnnotations::read_only().with_title("Search Callers For")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let symbol = get_string_arg(&args, "symbol")?;
        let file_pattern = get_optional_string_arg(&args, "file_pattern");
        let limit = args
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|l| l.min(100) as usize)
            .unwrap_or(20);

        let results =
            search_callers(&self.workspace, &symbol, file_pattern.as_deref(), limit).await;

        let response = serde_json::json!({
            "symbol": symbol,
            "file_pattern": file_pattern,
            "count": results.len(),
            "results": results
        });

        Ok(success_result(serde_json::to_string_pretty(&response)?))
    }
}

/// Helper function to search for callers of a symbol.
async fn search_callers(
    workspace: &Path,
    symbol: &str,
    file_pattern: Option<&str>,
    limit: usize,
) -> Vec<serde_json::Value> {
    let mut results = Vec::new();

    // Patterns to find function/method calls
    let call_patterns = [
        format!(r"{}[\s]*\(", symbol),   // function call: symbol(
        format!(r"\.{}[\s]*\(", symbol), // method call: .symbol(
        format!(r"::{}[\s]*\(", symbol), // Rust path call: ::symbol(
        format!(r"->{}[\s]*\(", symbol), // C/C++ pointer call: ->symbol(
    ];

    let file_glob = file_pattern.and_then(|p| Pattern::new(p).ok());

    for entry in WalkDir::new(workspace)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if results.len() >= limit {
            break;
        }

        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let relative_path = path.strip_prefix(workspace).unwrap_or(path);
        let path_str = relative_path.to_string_lossy();

        // Check file pattern filter
        if let Some(ref glob) = file_glob {
            if !glob.matches(&path_str) {
                continue;
            }
        }

        // Check if file contains caller
        if let Ok(content) = fs::read_to_string(path).await {
            let mut matching_lines = Vec::new();

            for (i, line) in content.lines().enumerate() {
                for pattern in &call_patterns {
                    if regex::Regex::new(pattern)
                        .map(|re| re.is_match(line))
                        .unwrap_or(false)
                    {
                        matching_lines.push(serde_json::json!({
                            "line": i + 1,
                            "content": line.trim()
                        }));
                        break;
                    }
                }
                if matching_lines.len() >= 10 {
                    break;
                }
            }

            if !matching_lines.is_empty() {
                results.push(serde_json::json!({
                    "path": path_str,
                    "calls": matching_lines
                }));
            }
        }
    }

    results
}

/// Helper function to search for files importing a module.
async fn search_importers(
    workspace: &Path,
    module: &str,
    file_pattern: Option<&str>,
    limit: usize,
) -> Vec<serde_json::Value> {
    let mut results = Vec::new();

    // Import patterns for different languages
    let import_patterns = [
        format!("import.*{}", module),                // Python, JS, TS
        format!("from.*{}.*import", module),          // Python
        format!("require.*['\"].*{}.*['\"]", module), // Node.js
        format!("use.*{}", module),                   // Rust
        format!("#include.*{}", module),              // C/C++
        format!("using.*{}", module),                 // C#
    ];

    let file_glob = file_pattern.and_then(|p| Pattern::new(p).ok());

    for entry in WalkDir::new(workspace)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if results.len() >= limit {
            break;
        }

        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let relative_path = path.strip_prefix(workspace).unwrap_or(path);
        let path_str = relative_path.to_string_lossy();

        // Check file pattern filter
        if let Some(ref glob) = file_glob {
            if !glob.matches(&path_str) {
                continue;
            }
        }

        // Check if file contains import statement
        if let Ok(content) = fs::read_to_string(path).await {
            let mut matching_lines = Vec::new();

            for (i, line) in content.lines().enumerate() {
                let line_lower = line.to_lowercase();
                for pattern in &import_patterns {
                    if regex::Regex::new(&pattern.to_lowercase())
                        .map(|re| re.is_match(&line_lower))
                        .unwrap_or(false)
                    {
                        matching_lines.push(serde_json::json!({
                            "line": i + 1,
                            "content": line.trim()
                        }));
                        break;
                    }
                }
                if matching_lines.len() >= 5 {
                    break;
                }
            }

            if !matching_lines.is_empty() {
                results.push(serde_json::json!({
                    "path": path_str,
                    "imports": matching_lines
                }));
            }
        }
    }

    results
}

/// Search importers for tool.
pub struct SearchImportersForTool {
    workspace: Arc<Path>,
}

impl SearchImportersForTool {
    pub fn new(workspace: &Path) -> Self {
        Self {
            workspace: Arc::from(workspace),
        }
    }
}

#[async_trait]
impl ToolHandler for SearchImportersForTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "search_importers_for".to_string(),
            description: "Search for files that import a specific module or symbol. \
                Finds import/require/use statements across the codebase."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "module": {
                        "type": "string",
                        "description": "The module or symbol name to find importers for"
                    },
                    "file_pattern": {
                        "type": "string",
                        "description": "Optional file pattern to limit search (e.g., '*.rs', '*.py')"
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 100,
                        "description": "Maximum number of results (default: 20)"
                    }
                },
                "required": ["module"]
            }),
            annotations: Some(ToolAnnotations::read_only().with_title("Search Importers For")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let module = get_string_arg(&args, "module")?;
        let file_pattern = get_optional_string_arg(&args, "file_pattern");
        let limit = args
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|l| l.min(100) as usize)
            .unwrap_or(20);

        let results =
            search_importers(&self.workspace, &module, file_pattern.as_deref(), limit).await;

        let response = serde_json::json!({
            "module": module,
            "file_pattern": file_pattern,
            "count": results.len(),
            "results": results
        });

        Ok(success_result(serde_json::to_string_pretty(&response)?))
    }
}

/// Info request tool - simplified codebase retrieval with explanation mode.
pub struct InfoRequestTool {
    context_service: Arc<ContextService>,
}

impl InfoRequestTool {
    pub fn new(context_service: Arc<ContextService>) -> Self {
        Self { context_service }
    }
}

#[async_trait]
impl ToolHandler for InfoRequestTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "info_request".to_string(),
            description: "Simplified codebase retrieval with explanation mode. \
                Searches for information and optionally provides explanations of relationships."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Natural language query about the codebase"
                    },
                    "explain": {
                        "type": "boolean",
                        "description": "Whether to include relationship explanations (default: false)"
                    },
                    "max_results": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 50,
                        "description": "Maximum number of results (default: 10)"
                    }
                },
                "required": ["query"]
            }),
            annotations: Some(ToolAnnotations::read_only().with_title("Info Request")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let query = get_string_arg(&args, "query")?;
        let explain = args
            .get("explain")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .map(|l| l.min(50) as usize)
            .unwrap_or(10);

        // Use context service to search
        let search_result = self
            .context_service
            .search(&query, Some(max_results * 100))
            .await?;

        let response = if explain {
            serde_json::json!({
                "query": query,
                "explanation": format!(
                    "Searched for '{}' in the codebase. Found relevant code snippets that may help answer your question.",
                    query
                ),
                "relationships": [
                    "The results are ordered by relevance to your query.",
                    "Code snippets may reference other files or symbols in the codebase.",
                    "Use search_callers_for or search_importers_for for deeper relationship analysis."
                ],
                "results": search_result
            })
        } else {
            serde_json::json!({
                "query": query,
                "results": search_result
            })
        };

        Ok(success_result(serde_json::to_string_pretty(&response)?))
    }
}

/// Pattern search tool - structural code pattern matching.
pub struct PatternSearchTool {
    workspace: Arc<Path>,
}

impl PatternSearchTool {
    pub fn new(workspace: &Path) -> Self {
        Self {
            workspace: Arc::from(workspace),
        }
    }
}

#[async_trait]
impl ToolHandler for PatternSearchTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "pattern_search".to_string(),
            description: "Search for structural code patterns across the codebase. \
                Finds code matching specific patterns like function definitions, class declarations, etc."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Regex pattern to search for"
                    },
                    "pattern_type": {
                        "type": "string",
                        "enum": ["function", "class", "import", "variable", "custom"],
                        "description": "Type of pattern to search for (provides preset patterns)"
                    },
                    "language": {
                        "type": "string",
                        "description": "Filter by programming language (e.g., 'rust', 'python', 'typescript')"
                    },
                    "file_pattern": {
                        "type": "string",
                        "description": "File pattern to limit search (e.g., '*.rs', '*.py')"
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 100,
                        "description": "Maximum number of results (default: 20)"
                    }
                },
                "required": []
            }),
            annotations: Some(ToolAnnotations::read_only().with_title("Pattern Search")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let custom_pattern = get_optional_string_arg(&args, "pattern");
        let pattern_type = get_optional_string_arg(&args, "pattern_type");
        let language = get_optional_string_arg(&args, "language");
        let file_pattern = get_optional_string_arg(&args, "file_pattern");
        let limit = args
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|l| l.min(100) as usize)
            .unwrap_or(20);

        // Determine the pattern to use
        let pattern = if let Some(ref custom) = custom_pattern {
            custom.clone()
        } else {
            match pattern_type.as_deref() {
                Some("function") => get_function_pattern(language.as_deref()),
                Some("class") => get_class_pattern(language.as_deref()),
                Some("import") => get_import_pattern(language.as_deref()),
                Some("variable") => get_variable_pattern(language.as_deref()),
                _ => r"\w+".to_string(), // Default: match any word
            }
        };

        let results =
            search_with_pattern(&self.workspace, &pattern, file_pattern.as_deref(), limit).await;

        let response = serde_json::json!({
            "pattern": pattern,
            "pattern_type": pattern_type,
            "language": language,
            "file_pattern": file_pattern,
            "count": results.len(),
            "results": results
        });

        Ok(success_result(serde_json::to_string_pretty(&response)?))
    }
}

/// Context search tool - context-aware semantic search.
pub struct ContextSearchTool {
    context_service: Arc<ContextService>,
}

impl ContextSearchTool {
    pub fn new(context_service: Arc<ContextService>) -> Self {
        Self { context_service }
    }
}

#[async_trait]
impl ToolHandler for ContextSearchTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "context_search".to_string(),
            description: "Context-aware semantic search that understands code relationships. \
                Searches with awareness of file context, symbol relationships, and code structure."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Natural language query"
                    },
                    "context_file": {
                        "type": "string",
                        "description": "Optional file path to use as context anchor"
                    },
                    "include_related": {
                        "type": "boolean",
                        "description": "Include related files and symbols (default: true)"
                    },
                    "max_tokens": {
                        "type": "integer",
                        "minimum": 100,
                        "maximum": 50000,
                        "description": "Maximum tokens in response (default: 4000)"
                    }
                },
                "required": ["query"]
            }),
            annotations: Some(ToolAnnotations::read_only().with_title("Context Search")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let query = get_string_arg(&args, "query")?;
        let context_file = get_optional_string_arg(&args, "context_file");
        let include_related = args
            .get("include_related")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let max_tokens = args
            .get("max_tokens")
            .and_then(|v| v.as_u64())
            .map(|l| l.min(50000) as usize)
            .unwrap_or(4000);

        // Build enhanced query with context
        let enhanced_query = if let Some(ref file) = context_file {
            format!("{} (in context of {})", query, file)
        } else {
            query.clone()
        };

        // Search with context service
        let search_result = self
            .context_service
            .search(&enhanced_query, Some(max_tokens))
            .await?;

        let response = serde_json::json!({
            "query": query,
            "context_file": context_file,
            "include_related": include_related,
            "max_tokens": max_tokens,
            "results": search_result
        });

        Ok(success_result(serde_json::to_string_pretty(&response)?))
    }
}

// Helper functions for pattern generation

fn get_function_pattern(language: Option<&str>) -> String {
    match language {
        Some("rust") => r"(pub\s+)?(async\s+)?fn\s+\w+".to_string(),
        Some("python") => r"(async\s+)?def\s+\w+".to_string(),
        Some("typescript") | Some("javascript") => {
            r"(async\s+)?function\s+\w+|const\s+\w+\s*=\s*(async\s+)?\(".to_string()
        }
        Some("go") => r"func\s+(\(\w+\s+\*?\w+\)\s+)?\w+".to_string(),
        Some("java") | Some("kotlin") => {
            r"(public|private|protected)?\s*(static)?\s*\w+\s+\w+\s*\(".to_string()
        }
        _ => r"(fn|def|function|func)\s+\w+".to_string(),
    }
}

fn get_class_pattern(language: Option<&str>) -> String {
    match language {
        Some("rust") => r"(pub\s+)?(struct|enum|trait|impl)\s+\w+".to_string(),
        Some("python") => r"class\s+\w+".to_string(),
        Some("typescript") | Some("javascript") => r"class\s+\w+".to_string(),
        Some("go") => r"type\s+\w+\s+struct".to_string(),
        Some("java") | Some("kotlin") => {
            r"(public|private)?\s*(abstract)?\s*class\s+\w+".to_string()
        }
        _ => r"(class|struct|enum|trait|interface)\s+\w+".to_string(),
    }
}

fn get_import_pattern(language: Option<&str>) -> String {
    match language {
        Some("rust") => r"use\s+[\w:]+".to_string(),
        Some("python") => r"(from\s+\w+\s+)?import\s+\w+".to_string(),
        Some("typescript") | Some("javascript") => r"import\s+.*from|require\s*\(".to_string(),
        Some("go") => r#"import\s+(\(|"[\w/]+")"#.to_string(),
        Some("java") => r"import\s+[\w.]+".to_string(),
        _ => r"(import|use|require|include)\s+".to_string(),
    }
}

fn get_variable_pattern(language: Option<&str>) -> String {
    match language {
        Some("rust") => r"(let|const|static)\s+(mut\s+)?\w+".to_string(),
        Some("python") => r"\w+\s*=\s*".to_string(),
        Some("typescript") | Some("javascript") => r"(let|const|var)\s+\w+".to_string(),
        Some("go") => r"(var|const)\s+\w+|:=".to_string(),
        Some("java") | Some("kotlin") => r"(final\s+)?\w+\s+\w+\s*=".to_string(),
        _ => r"(let|const|var|val)\s+\w+".to_string(),
    }
}

async fn search_with_pattern(
    workspace: &Path,
    pattern: &str,
    file_pattern: Option<&str>,
    limit: usize,
) -> Vec<serde_json::Value> {
    let mut results = Vec::new();
    let file_glob = file_pattern.and_then(|p| Pattern::new(p).ok());
    let regex = match regex::Regex::new(pattern) {
        Ok(r) => r,
        Err(_) => return results,
    };

    for entry in WalkDir::new(workspace)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if results.len() >= limit {
            break;
        }

        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let relative_path = path.strip_prefix(workspace).unwrap_or(path);
        let path_str = relative_path.to_string_lossy();

        // Check file pattern filter
        if let Some(ref glob) = file_glob {
            if !glob.matches(&path_str) {
                continue;
            }
        }

        if let Ok(content) = fs::read_to_string(path).await {
            let mut matching_lines = Vec::new();

            for (i, line) in content.lines().enumerate() {
                if regex.is_match(line) {
                    matching_lines.push(serde_json::json!({
                        "line": i + 1,
                        "content": line.trim()
                    }));
                }
                if matching_lines.len() >= 10 {
                    break;
                }
            }

            if !matching_lines.is_empty() {
                results.push(serde_json::json!({
                    "path": path_str,
                    "matches": matching_lines
                }));
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_function_pattern_rust() {
        let pattern = get_function_pattern(Some("rust"));
        assert!(pattern.contains("fn"));
        assert!(pattern.contains("async"));
    }

    #[test]
    fn test_get_function_pattern_python() {
        let pattern = get_function_pattern(Some("python"));
        assert!(pattern.contains("def"));
    }

    #[test]
    fn test_get_function_pattern_typescript() {
        let pattern = get_function_pattern(Some("typescript"));
        assert!(pattern.contains("function"));
    }

    #[test]
    fn test_get_function_pattern_default() {
        let pattern = get_function_pattern(None);
        assert!(pattern.contains("fn"));
        assert!(pattern.contains("def"));
        assert!(pattern.contains("function"));
    }

    #[test]
    fn test_get_class_pattern_rust() {
        let pattern = get_class_pattern(Some("rust"));
        assert!(pattern.contains("struct"));
        assert!(pattern.contains("enum"));
        assert!(pattern.contains("trait"));
    }

    #[test]
    fn test_get_class_pattern_python() {
        let pattern = get_class_pattern(Some("python"));
        assert!(pattern.contains("class"));
    }

    #[test]
    fn test_get_class_pattern_default() {
        let pattern = get_class_pattern(None);
        assert!(pattern.contains("class"));
        assert!(pattern.contains("struct"));
    }

    #[test]
    fn test_get_import_pattern_rust() {
        let pattern = get_import_pattern(Some("rust"));
        assert!(pattern.contains("use"));
    }

    #[test]
    fn test_get_import_pattern_python() {
        let pattern = get_import_pattern(Some("python"));
        assert!(pattern.contains("import"));
        assert!(pattern.contains("from"));
    }

    #[test]
    fn test_get_import_pattern_typescript() {
        let pattern = get_import_pattern(Some("typescript"));
        assert!(pattern.contains("import"));
        assert!(pattern.contains("require"));
    }

    #[test]
    fn test_get_variable_pattern_rust() {
        let pattern = get_variable_pattern(Some("rust"));
        assert!(pattern.contains("let"));
        assert!(pattern.contains("const"));
        assert!(pattern.contains("mut"));
    }

    #[test]
    fn test_get_variable_pattern_python() {
        let pattern = get_variable_pattern(Some("python"));
        assert!(pattern.contains("="));
    }

    #[test]
    fn test_get_variable_pattern_typescript() {
        let pattern = get_variable_pattern(Some("typescript"));
        assert!(pattern.contains("let"));
        assert!(pattern.contains("const"));
        assert!(pattern.contains("var"));
    }

    #[test]
    fn test_test_globs_coverage() {
        // Verify TEST_GLOBS covers common test file patterns
        assert!(TEST_GLOBS.iter().any(|g| g.contains("test")));
        assert!(TEST_GLOBS.iter().any(|g| g.contains("spec")));
        assert!(TEST_GLOBS.iter().any(|g| g.contains("__tests__")));
    }

    #[test]
    fn test_config_globs_coverage() {
        // Verify CONFIG_GLOBS covers common config file patterns
        assert!(CONFIG_GLOBS.iter().any(|g| g.contains("yaml")));
        assert!(CONFIG_GLOBS.iter().any(|g| g.contains("json")));
        assert!(CONFIG_GLOBS.iter().any(|g| g.contains("toml")));
        assert!(CONFIG_GLOBS.iter().any(|g| g.contains("env")));
        assert!(CONFIG_GLOBS.iter().any(|g| g.contains("config")));
    }

    #[test]
    fn test_pattern_regex_validity() {
        // Verify all generated patterns are valid regex
        for lang in &[
            Some("rust"),
            Some("python"),
            Some("typescript"),
            Some("javascript"),
            Some("go"),
            Some("java"),
            Some("kotlin"),
            None,
        ] {
            let pattern = get_function_pattern(*lang);
            assert!(
                regex::Regex::new(&pattern).is_ok(),
                "Invalid function pattern for {:?}: {}",
                lang,
                pattern
            );

            let pattern = get_class_pattern(*lang);
            assert!(
                regex::Regex::new(&pattern).is_ok(),
                "Invalid class pattern for {:?}: {}",
                lang,
                pattern
            );

            let pattern = get_import_pattern(*lang);
            assert!(
                regex::Regex::new(&pattern).is_ok(),
                "Invalid import pattern for {:?}: {}",
                lang,
                pattern
            );

            let pattern = get_variable_pattern(*lang);
            assert!(
                regex::Regex::new(&pattern).is_ok(),
                "Invalid variable pattern for {:?}: {}",
                lang,
                pattern
            );
        }
    }
}
