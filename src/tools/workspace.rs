//! Workspace analysis and statistics tools.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::process::Command;

use crate::error::Result;
use crate::mcp::handler::{error_result, get_string_arg, success_result, ToolHandler};
use crate::mcp::protocol::{Tool, ToolResult};
use crate::service::ContextService;

/// Get workspace statistics (file counts, language breakdown, etc.).
pub struct WorkspaceStatsTool {
    service: Arc<ContextService>,
}

impl WorkspaceStatsTool {
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for WorkspaceStatsTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "workspace_stats".to_string(),
            description: "Get workspace statistics including file counts by language, total lines of code, and directory structure overview.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "include_hidden": {
                        "type": "boolean",
                        "description": "Include hidden files/directories (default: false)"
                    }
                },
                "required": []
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let include_hidden = args
            .get("include_hidden")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let workspace = self.service.workspace_path();
        match collect_workspace_stats(workspace, include_hidden).await {
            Ok(stats) => Ok(success_result(
                serde_json::to_string_pretty(&stats).unwrap(),
            )),
            Err(e) => Ok(error_result(format!("Failed to collect stats: {}", e))),
        }
    }
}

#[derive(serde::Serialize)]
struct WorkspaceStats {
    total_files: usize,
    total_lines: usize,
    languages: HashMap<String, LanguageStats>,
    directories: usize,
}

#[derive(serde::Serialize, Default)]
struct LanguageStats {
    files: usize,
    lines: usize,
}

async fn collect_workspace_stats(root: &Path, include_hidden: bool) -> Result<WorkspaceStats> {
    let mut stats = WorkspaceStats {
        total_files: 0,
        total_lines: 0,
        languages: HashMap::new(),
        directories: 0,
    };

    collect_stats_recursive(root, &mut stats, include_hidden).await;
    Ok(stats)
}

fn collect_stats_recursive<'a>(
    path: &'a Path,
    stats: &'a mut WorkspaceStats,
    include_hidden: bool,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
    Box::pin(async move {
        let mut entries = match fs::read_dir(path).await {
            Ok(e) => e,
            Err(_) => return,
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Skip hidden files/dirs unless requested
            if !include_hidden && name_str.starts_with('.') {
                continue;
            }

            // Skip common non-code directories
            if matches!(
                name_str.as_ref(),
                "node_modules" | "target" | "dist" | "build" | ".git" | "__pycache__" | "venv"
            ) {
                continue;
            }

            let file_type = match entry.file_type().await {
                Ok(ft) => ft,
                Err(_) => continue,
            };
            let entry_path = entry.path();

            if file_type.is_dir() {
                stats.directories += 1;
                collect_stats_recursive(&entry_path, stats, include_hidden).await;
            } else if file_type.is_file() {
                if let Some(ext) = entry_path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    let lang = extension_to_language(&ext_str);

                    if lang != "binary" {
                        stats.total_files += 1;
                        let lines = count_lines(&entry_path).await.unwrap_or(0);
                        stats.total_lines += lines;

                        let lang_stats = stats.languages.entry(lang.to_string()).or_default();
                        lang_stats.files += 1;
                        lang_stats.lines += lines;
                    }
                }
            }
        }
    })
}

async fn count_lines(path: &Path) -> Result<usize> {
    let content = fs::read_to_string(path).await?;
    Ok(content.lines().count())
}

fn extension_to_language(ext: &str) -> &'static str {
    match ext {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "tsx" | "jsx" => "react",
        "go" => "go",
        "java" => "java",
        "rb" => "ruby",
        "c" | "h" => "c",
        "cpp" | "cc" | "hpp" => "cpp",
        "cs" => "csharp",
        "swift" => "swift",
        "kt" => "kotlin",
        "scala" => "scala",
        "php" => "php",
        "sh" | "bash" => "shell",
        "sql" => "sql",
        "html" => "html",
        "css" | "scss" | "sass" => "css",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "md" | "markdown" => "markdown",
        "xml" => "xml",
        _ => "binary",
    }
}

/// Get git status for the workspace.
pub struct GitStatusTool {
    service: Arc<ContextService>,
}

impl GitStatusTool {
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for GitStatusTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "git_status".to_string(),
            description: "Get the current git status of the workspace including staged, unstaged, and untracked files.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "include_diff": {
                        "type": "boolean",
                        "description": "Include diff of changes (default: false)"
                    }
                },
                "required": []
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let include_diff = args
            .get("include_diff")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let workspace = self.service.workspace_path();

        // Get git status
        let status_output = Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .current_dir(workspace)
            .output()
            .await;

        let status = match status_output {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            _ => return Ok(error_result("Not a git repository or git command failed")),
        };

        // Parse status
        let mut result = GitStatus {
            staged: Vec::new(),
            unstaged: Vec::new(),
            untracked: Vec::new(),
            diff: None,
        };

        for line in status.lines() {
            if line.len() < 3 {
                continue;
            }
            let index_status = line.chars().next().unwrap_or(' ');
            let work_status = line.chars().nth(1).unwrap_or(' ');
            let file = line[3..].to_string();

            match (index_status, work_status) {
                ('?', '?') => result.untracked.push(file),
                (' ', _) => result.unstaged.push(file),
                (_, ' ') => result.staged.push(file),
                (_, _) => {
                    result.staged.push(file.clone());
                    result.unstaged.push(file);
                }
            }
        }

        // Get diff if requested
        if include_diff {
            let diff_output = Command::new("git")
                .arg("diff")
                .current_dir(workspace)
                .output()
                .await;

            if let Ok(output) = diff_output {
                if output.status.success() {
                    result.diff = Some(String::from_utf8_lossy(&output.stdout).to_string());
                }
            }
        }

        Ok(success_result(
            serde_json::to_string_pretty(&result).unwrap(),
        ))
    }
}

#[derive(serde::Serialize)]
struct GitStatus {
    staged: Vec<String>,
    unstaged: Vec<String>,
    untracked: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diff: Option<String>,
}

/// Extract symbols (functions, classes, structs) from a file.
pub struct ExtractSymbolsTool {
    service: Arc<ContextService>,
}

impl ExtractSymbolsTool {
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for ExtractSymbolsTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "extract_symbols".to_string(),
            description: "Extract function, class, struct, and other symbol definitions from a source file. Returns a structured list of symbols with their line numbers.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the source file"
                    }
                },
                "required": ["file_path"]
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let file_path = get_string_arg(&args, "file_path")?;

        let workspace = self.service.workspace_path();
        let full_path = workspace.join(&file_path);

        // Security: canonicalize and verify path stays within workspace
        let workspace_canonical = match workspace.canonicalize() {
            Ok(p) => p,
            Err(e) => return Ok(error_result(format!("Cannot resolve workspace: {}", e))),
        };
        let path_canonical = match full_path.canonicalize() {
            Ok(p) => p,
            Err(e) => return Ok(error_result(format!("Cannot resolve {}: {}", file_path, e))),
        };
        if !path_canonical.starts_with(&workspace_canonical) {
            return Ok(error_result(format!(
                "Path escapes workspace: {}",
                file_path
            )));
        }

        let content = match fs::read_to_string(&path_canonical).await {
            Ok(c) => c,
            Err(e) => return Ok(error_result(format!("Failed to read file: {}", e))),
        };

        let ext = full_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let symbols = extract_symbols_from_content(&content, ext);

        let result = serde_json::json!({
            "file": file_path,
            "symbols": symbols
        });
        Ok(success_result(
            serde_json::to_string_pretty(&result).unwrap(),
        ))
    }
}

#[derive(serde::Serialize)]
struct Symbol {
    name: String,
    kind: String,
    line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature: Option<String>,
}

fn extract_symbols_from_content(content: &str, ext: &str) -> Vec<Symbol> {
    let mut symbols = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if let Some(sym) = detect_symbol(trimmed, ext, i + 1) {
            symbols.push(sym);
        }
    }

    symbols
}

fn detect_symbol(line: &str, ext: &str, line_num: usize) -> Option<Symbol> {
    match ext {
        "rs" => detect_rust_symbol(line, line_num),
        "py" => detect_python_symbol(line, line_num),
        "ts" | "tsx" | "js" | "jsx" => detect_ts_symbol(line, line_num),
        "go" => detect_go_symbol(line, line_num),
        _ => None,
    }
}

fn detect_rust_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    if line.starts_with("pub fn ") || line.starts_with("fn ") {
        let name = extract_name(line, "fn ");
        return Some(Symbol {
            name,
            kind: "function".to_string(),
            line: line_num,
            signature: Some(line.to_string()),
        });
    }
    if line.starts_with("pub struct ") || line.starts_with("struct ") {
        let name = extract_name(line, "struct ");
        return Some(Symbol {
            name,
            kind: "struct".to_string(),
            line: line_num,
            signature: None,
        });
    }
    if line.starts_with("pub enum ") || line.starts_with("enum ") {
        let name = extract_name(line, "enum ");
        return Some(Symbol {
            name,
            kind: "enum".to_string(),
            line: line_num,
            signature: None,
        });
    }
    if line.starts_with("pub trait ") || line.starts_with("trait ") {
        let name = extract_name(line, "trait ");
        return Some(Symbol {
            name,
            kind: "trait".to_string(),
            line: line_num,
            signature: None,
        });
    }
    if line.starts_with("impl ") {
        let rest = line.strip_prefix("impl ").unwrap_or(line);
        // Skip generic parameters if present (e.g., impl<T> Foo<T>)
        let rest = if rest.starts_with('<') {
            rest.split_once('>')
                .map(|(_, after)| after.trim_start())
                .unwrap_or(rest)
        } else {
            rest
        };
        // Extract the type/trait name (first identifier before '<', ' ', '{', or 'for')
        let name = rest
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .next()
            .unwrap_or("")
            .to_string();
        return Some(Symbol {
            name,
            kind: "impl".to_string(),
            line: line_num,
            signature: Some(line.to_string()),
        });
    }
    None
}

fn detect_python_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    if line.starts_with("def ") {
        let name = extract_name(line, "def ");
        return Some(Symbol {
            name,
            kind: "function".to_string(),
            line: line_num,
            signature: Some(line.to_string()),
        });
    }
    if line.starts_with("class ") {
        let name = extract_name(line, "class ");
        return Some(Symbol {
            name,
            kind: "class".to_string(),
            line: line_num,
            signature: None,
        });
    }
    if line.starts_with("async def ") {
        let name = extract_name(line, "async def ");
        return Some(Symbol {
            name,
            kind: "async_function".to_string(),
            line: line_num,
            signature: Some(line.to_string()),
        });
    }
    None
}

fn detect_ts_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    // Function declarations
    if line.contains("function ") {
        let parts: Vec<&str> = line.split("function ").collect();
        if parts.len() > 1 {
            let name = parts[1].split('(').next().unwrap_or("").trim().to_string();
            if !name.is_empty() {
                return Some(Symbol {
                    name,
                    kind: "function".to_string(),
                    line: line_num,
                    signature: Some(line.to_string()),
                });
            }
        }
    }
    // Class declarations
    if line.starts_with("class ") || line.starts_with("export class ") {
        let name = if line.contains("export class ") {
            extract_name(line, "export class ")
        } else {
            extract_name(line, "class ")
        };
        return Some(Symbol {
            name,
            kind: "class".to_string(),
            line: line_num,
            signature: None,
        });
    }
    // Interface declarations
    if line.starts_with("interface ") || line.starts_with("export interface ") {
        let name = if line.contains("export interface ") {
            extract_name(line, "export interface ")
        } else {
            extract_name(line, "interface ")
        };
        return Some(Symbol {
            name,
            kind: "interface".to_string(),
            line: line_num,
            signature: None,
        });
    }
    // Type declarations
    if line.starts_with("type ") || line.starts_with("export type ") {
        let name = if line.contains("export type ") {
            extract_name(line, "export type ")
        } else {
            extract_name(line, "type ")
        };
        return Some(Symbol {
            name,
            kind: "type".to_string(),
            line: line_num,
            signature: None,
        });
    }
    None
}

fn detect_go_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    if line.starts_with("func ") {
        let rest = line.strip_prefix("func ").unwrap_or(line);
        let name = if rest.starts_with('(') {
            // Method: func (r *Receiver) MethodName(...)
            rest.split(')')
                .nth(1)
                .and_then(|s| s.trim().split('(').next())
        } else {
            // Function: func FuncName(...)
            rest.split('(').next()
        };
        if let Some(name) = name {
            return Some(Symbol {
                name: name.trim().to_string(),
                kind: "function".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }
    if line.starts_with("type ") && line.contains(" struct") {
        let name = extract_name(line, "type ");
        return Some(Symbol {
            name,
            kind: "struct".to_string(),
            line: line_num,
            signature: None,
        });
    }
    if line.starts_with("type ") && line.contains(" interface") {
        let name = extract_name(line, "type ");
        return Some(Symbol {
            name,
            kind: "interface".to_string(),
            line: line_num,
            signature: None,
        });
    }
    None
}

fn extract_name(line: &str, prefix: &str) -> String {
    line.split(prefix)
        .nth(1)
        .unwrap_or("")
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .next()
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_to_language() {
        assert_eq!(extension_to_language("rs"), "rust");
        assert_eq!(extension_to_language("py"), "python");
        assert_eq!(extension_to_language("ts"), "typescript");
        assert_eq!(extension_to_language("go"), "go");
        assert_eq!(extension_to_language("unknown"), "binary");
    }

    #[test]
    fn test_detect_rust_symbol_function() {
        let sym = detect_rust_symbol("pub fn hello_world() -> Result<()> {", 1);
        assert!(sym.is_some());
        let sym = sym.unwrap();
        assert_eq!(sym.name, "hello_world");
        assert_eq!(sym.kind, "function");
    }

    #[test]
    fn test_detect_rust_symbol_struct() {
        let sym = detect_rust_symbol("pub struct MyStruct {", 10);
        assert!(sym.is_some());
        let sym = sym.unwrap();
        assert_eq!(sym.name, "MyStruct");
        assert_eq!(sym.kind, "struct");
        assert_eq!(sym.line, 10);
    }

    #[test]
    fn test_detect_rust_symbol_enum() {
        let sym = detect_rust_symbol("enum Color { Red, Green, Blue }", 5);
        assert!(sym.is_some());
        let sym = sym.unwrap();
        assert_eq!(sym.name, "Color");
        assert_eq!(sym.kind, "enum");
    }

    #[test]
    fn test_detect_rust_symbol_trait() {
        let sym = detect_rust_symbol("pub trait Handler {", 15);
        assert!(sym.is_some());
        let sym = sym.unwrap();
        assert_eq!(sym.name, "Handler");
        assert_eq!(sym.kind, "trait");
    }

    #[test]
    fn test_detect_python_symbol_function() {
        let sym = detect_python_symbol("def process_data(data: dict) -> list:", 1);
        assert!(sym.is_some());
        let sym = sym.unwrap();
        assert_eq!(sym.name, "process_data");
        assert_eq!(sym.kind, "function");
    }

    #[test]
    fn test_detect_python_symbol_class() {
        let sym = detect_python_symbol("class MyClass:", 1);
        assert!(sym.is_some());
        let sym = sym.unwrap();
        assert_eq!(sym.name, "MyClass");
        assert_eq!(sym.kind, "class");
    }

    #[test]
    fn test_detect_python_symbol_async() {
        let sym = detect_python_symbol("async def fetch_data():", 1);
        assert!(sym.is_some());
        let sym = sym.unwrap();
        assert_eq!(sym.name, "fetch_data");
        assert_eq!(sym.kind, "async_function");
    }

    #[test]
    fn test_detect_ts_symbol_function() {
        let sym = detect_ts_symbol("function processData(data: any): void {", 1);
        assert!(sym.is_some());
        let sym = sym.unwrap();
        assert_eq!(sym.name, "processData");
        assert_eq!(sym.kind, "function");
    }

    #[test]
    fn test_detect_ts_symbol_class() {
        let sym = detect_ts_symbol("export class UserService {", 1);
        assert!(sym.is_some());
        let sym = sym.unwrap();
        assert_eq!(sym.name, "UserService");
        assert_eq!(sym.kind, "class");
    }

    #[test]
    fn test_detect_ts_symbol_interface() {
        let sym = detect_ts_symbol("interface UserData {", 1);
        assert!(sym.is_some());
        let sym = sym.unwrap();
        assert_eq!(sym.name, "UserData");
        assert_eq!(sym.kind, "interface");
    }

    #[test]
    fn test_detect_go_symbol_function() {
        let sym = detect_go_symbol(
            "func HandleRequest(w http.ResponseWriter, r *http.Request) {",
            1,
        );
        assert!(sym.is_some());
        let sym = sym.unwrap();
        assert_eq!(sym.name, "HandleRequest");
        assert_eq!(sym.kind, "function");
    }

    #[test]
    fn test_detect_go_symbol_struct() {
        let sym = detect_go_symbol("type Config struct {", 1);
        assert!(sym.is_some());
        let sym = sym.unwrap();
        assert_eq!(sym.name, "Config");
        assert_eq!(sym.kind, "struct");
    }

    #[test]
    fn test_extract_symbols_from_content() {
        let rust_code = r#"
pub struct Server {
    port: u16,
}

impl Server {
    pub fn new(port: u16) -> Self {
        Self { port }
    }
}
"#;
        let symbols = extract_symbols_from_content(rust_code, "rs");
        assert!(!symbols.is_empty());
        assert!(symbols
            .iter()
            .any(|s| s.name == "Server" && s.kind == "struct"));
        assert!(symbols
            .iter()
            .any(|s| s.name == "new" && s.kind == "function"));
    }

    #[test]
    fn test_extract_name() {
        assert_eq!(extract_name("fn hello() {", "fn "), "hello");
        assert_eq!(extract_name("struct MyStruct {", "struct "), "MyStruct");
        assert_eq!(extract_name("def process():", "def "), "process");
    }
}
