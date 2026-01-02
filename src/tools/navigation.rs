//! Code navigation tools for finding references and definitions.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncBufReadExt;

use crate::error::Result;
use crate::mcp::handler::{error_result, get_string_arg, success_result, ToolHandler};
use crate::mcp::protocol::{Tool, ToolResult};
use crate::service::ContextService;

/// Find all references to a symbol in the codebase.
pub struct FindReferencesTool {
    service: Arc<ContextService>,
}

impl FindReferencesTool {
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for FindReferencesTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "find_references".to_string(),
            description: "Find all references to a symbol (function, class, variable) in the codebase. Returns file paths and line numbers where the symbol is used.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "The symbol name to search for"
                    },
                    "file_pattern": {
                        "type": "string",
                        "description": "Optional glob pattern to filter files (e.g., '*.rs', 'src/**/*.ts')"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results to return (default: 50)"
                    }
                },
                "required": ["symbol"]
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let symbol = get_string_arg(&args, "symbol")?;
        let file_pattern = args.get("file_pattern").and_then(|v| v.as_str());
        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(50) as usize;

        let workspace = self.service.workspace();
        let references = find_symbol_in_files(workspace, &symbol, file_pattern, max_results).await;

        if references.is_empty() {
            return Ok(success_result(format!(
                "No references found for symbol: `{}`",
                symbol
            )));
        }

        let mut output = format!(
            "# References to `{}`\n\nFound {} references:\n\n",
            symbol,
            references.len()
        );

        for reference in references {
            output.push_str(&format!(
                "- **{}:{}**: `{}`\n",
                reference.file,
                reference.line,
                reference.context.trim()
            ));
        }

        Ok(success_result(output))
    }
}

/// Go to definition - find where a symbol is defined.
pub struct GoToDefinitionTool {
    service: Arc<ContextService>,
}

impl GoToDefinitionTool {
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for GoToDefinitionTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "go_to_definition".to_string(),
            description: "Find the definition of a symbol (function, class, struct, type). Returns the file and line where the symbol is defined.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "The symbol name to find the definition of"
                    },
                    "language": {
                        "type": "string",
                        "description": "Programming language hint (rust, python, typescript, etc.)"
                    }
                },
                "required": ["symbol"]
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let symbol = get_string_arg(&args, "symbol")?;
        let language = args.get("language").and_then(|v| v.as_str());

        let workspace = self.service.workspace();
        let definitions = find_definition(workspace, &symbol, language).await;

        if definitions.is_empty() {
            return Ok(success_result(format!(
                "No definition found for symbol: `{}`",
                symbol
            )));
        }

        let mut output = format!("# Definition of `{}`\n\n", symbol);

        for def in definitions {
            output.push_str(&format!("## {}\n\n", def.file));
            output.push_str(&format!("Line {}\n\n", def.line));
            output.push_str(&format!("```{}\n{}\n```\n\n", def.language, def.context));
        }

        Ok(success_result(output))
    }
}

/// Diff two files or show changes.
pub struct DiffFilesTool {
    service: Arc<ContextService>,
}

impl DiffFilesTool {
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for DiffFilesTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "diff_files".to_string(),
            description:
                "Compare two files and show the differences. Returns a unified diff format."
                    .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file1": {
                        "type": "string",
                        "description": "Path to the first file"
                    },
                    "file2": {
                        "type": "string",
                        "description": "Path to the second file"
                    },
                    "context_lines": {
                        "type": "integer",
                        "description": "Number of context lines around changes (default: 3)"
                    }
                },
                "required": ["file1", "file2"]
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let file1 = get_string_arg(&args, "file1")?;
        let file2 = get_string_arg(&args, "file2")?;
        let context = args
            .get("context_lines")
            .and_then(|v| v.as_u64())
            .unwrap_or(3) as usize;

        let workspace = self.service.workspace();
        let path1 = workspace.join(&file1);
        let path2 = workspace.join(&file2);

        let content1 = match fs::read_to_string(&path1).await {
            Ok(c) => c,
            Err(e) => return Ok(error_result(format!("Cannot read {}: {}", file1, e))),
        };

        let content2 = match fs::read_to_string(&path2).await {
            Ok(c) => c,
            Err(e) => return Ok(error_result(format!("Cannot read {}: {}", file2, e))),
        };

        let diff = generate_diff(&file1, &file2, &content1, &content2, context);

        if diff.is_empty() {
            Ok(success_result("Files are identical.".to_string()))
        } else {
            Ok(success_result(format!("```diff\n{}\n```", diff)))
        }
    }
}

// ===== Helper types and functions =====

struct Reference {
    file: String,
    line: usize,
    context: String,
}

struct Definition {
    file: String,
    line: usize,
    context: String,
    language: String,
}

/// Find symbol references in files.
async fn find_symbol_in_files(
    workspace: &Path,
    symbol: &str,
    file_pattern: Option<&str>,
    max_results: usize,
) -> Vec<Reference> {
    let mut references = Vec::new();
    let mut stack = vec![workspace.to_path_buf()];

    while let Some(dir) = stack.pop() {
        if references.len() >= max_results {
            break;
        }

        let mut entries = match fs::read_dir(&dir).await {
            Ok(e) => e,
            Err(_) => continue,
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            if references.len() >= max_results {
                break;
            }

            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();

            // Skip hidden and common ignore patterns
            if name.starts_with('.')
                || matches!(name.as_ref(), "node_modules" | "target" | "dist" | "build")
            {
                continue;
            }

            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                // Check file pattern if provided
                if let Some(pattern) = file_pattern {
                    if !matches_pattern(&name, pattern) {
                        continue;
                    }
                }

                // Search file for symbol
                if let Ok(file) = fs::File::open(&path).await {
                    let reader = tokio::io::BufReader::new(file);
                    let mut lines = reader.lines();
                    let mut line_num = 0;

                    while let Ok(Some(line)) = lines.next_line().await {
                        line_num += 1;
                        if line.contains(symbol) {
                            let rel_path = path
                                .strip_prefix(workspace)
                                .unwrap_or(&path)
                                .to_string_lossy()
                                .to_string();

                            references.push(Reference {
                                file: rel_path,
                                line: line_num,
                                context: line,
                            });

                            if references.len() >= max_results {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    references
}

/// Find symbol definition.
async fn find_definition(
    workspace: &Path,
    symbol: &str,
    language: Option<&str>,
) -> Vec<Definition> {
    let mut definitions = Vec::new();

    // Build definition patterns based on language
    let patterns = get_definition_patterns(symbol, language);

    let mut stack = vec![workspace.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let mut entries = match fs::read_dir(&dir).await {
            Ok(e) => e,
            Err(_) => continue,
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();

            if name.starts_with('.')
                || matches!(name.as_ref(), "node_modules" | "target" | "dist" | "build")
            {
                continue;
            }

            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let file_lang = get_language(ext);

                // Skip if language hint provided and doesn't match
                if let Some(lang) = language {
                    if !file_lang.contains(lang) && lang != file_lang {
                        continue;
                    }
                }

                if let Ok(content) = fs::read_to_string(&path).await {
                    for (line_num, line) in content.lines().enumerate() {
                        for pattern in &patterns {
                            if line.contains(pattern) {
                                let rel_path = path
                                    .strip_prefix(workspace)
                                    .unwrap_or(&path)
                                    .to_string_lossy()
                                    .to_string();

                                // Get a few lines of context
                                let start = line_num.saturating_sub(1);
                                let context: String = content
                                    .lines()
                                    .skip(start)
                                    .take(5)
                                    .collect::<Vec<_>>()
                                    .join("\n");

                                definitions.push(Definition {
                                    file: rel_path,
                                    line: line_num + 1,
                                    context,
                                    language: file_lang.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    definitions
}

/// Get definition patterns for a symbol.
fn get_definition_patterns(symbol: &str, language: Option<&str>) -> Vec<String> {
    let mut patterns = Vec::new();

    match language {
        Some("rust" | "rs") => {
            patterns.push(format!("fn {}(", symbol));
            patterns.push(format!("struct {} ", symbol));
            patterns.push(format!("struct {}", symbol));
            patterns.push(format!("enum {} ", symbol));
            patterns.push(format!("trait {} ", symbol));
            patterns.push(format!("type {} ", symbol));
            patterns.push(format!("const {}", symbol));
            patterns.push(format!("static {}", symbol));
        }
        Some("python" | "py") => {
            patterns.push(format!("def {}(", symbol));
            patterns.push(format!("class {}:", symbol));
            patterns.push(format!("class {}(", symbol));
        }
        Some("typescript" | "javascript" | "ts" | "js") => {
            patterns.push(format!("function {}(", symbol));
            patterns.push(format!("const {} =", symbol));
            patterns.push(format!("let {} =", symbol));
            patterns.push(format!("class {} ", symbol));
            patterns.push(format!("interface {} ", symbol));
            patterns.push(format!("type {} =", symbol));
        }
        _ => {
            // Generic patterns
            patterns.push(format!("fn {}(", symbol));
            patterns.push(format!("function {}(", symbol));
            patterns.push(format!("def {}(", symbol));
            patterns.push(format!("class {} ", symbol));
            patterns.push(format!("struct {} ", symbol));
            patterns.push(format!("interface {} ", symbol));
        }
    }

    patterns
}

/// Get language from file extension.
fn get_language(ext: &str) -> &'static str {
    match ext {
        "rs" => "rust",
        "py" => "python",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "go" => "go",
        "java" => "java",
        "rb" => "ruby",
        "c" | "h" => "c",
        "cpp" | "hpp" | "cc" => "cpp",
        _ => "text",
    }
}

/// Simple pattern matching.
fn matches_pattern(name: &str, pattern: &str) -> bool {
    if let Some(ext) = pattern.strip_prefix("*.") {
        name.ends_with(&format!(".{}", ext))
    } else {
        name.contains(pattern)
    }
}

/// Generate a simple unified diff.
fn generate_diff(
    name1: &str,
    name2: &str,
    content1: &str,
    content2: &str,
    context: usize,
) -> String {
    let lines1: Vec<&str> = content1.lines().collect();
    let lines2: Vec<&str> = content2.lines().collect();

    if lines1 == lines2 {
        return String::new();
    }

    let mut output = format!("--- {}\n+++ {}\n", name1, name2);

    // Simple line-by-line comparison
    let max_len = lines1.len().max(lines2.len());
    let mut i = 0;

    while i < max_len {
        let l1 = lines1.get(i).copied();
        let l2 = lines2.get(i).copied();

        if l1 != l2 {
            // Found a difference - output hunk
            let start = i.saturating_sub(context);
            let end = (i + context + 1).min(max_len);

            output.push_str(&format!(
                "@@ -{},{} +{},{} @@\n",
                start + 1,
                end - start,
                start + 1,
                end - start
            ));

            for j in start..end {
                let l1 = lines1.get(j).copied().unwrap_or("");
                let l2 = lines2.get(j).copied().unwrap_or("");

                if l1 == l2 {
                    output.push_str(&format!(" {}\n", l1));
                } else {
                    if j < lines1.len() {
                        output.push_str(&format!("-{}\n", l1));
                    }
                    if j < lines2.len() {
                        output.push_str(&format!("+{}\n", l2));
                    }
                }
            }

            i = end;
        } else {
            i += 1;
        }
    }

    output
}
