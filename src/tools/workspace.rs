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
    /// Create a new WorkspaceStatsTool that uses the given ContextService.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// // `service` should be an initialized `ContextService` from the application.
    /// let service: Arc<ContextService> = Arc::new(/* ... */);
    /// let tool = WorkspaceStatsTool::new(service);
    /// ```
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for WorkspaceStatsTool {
    /// Returns the tool descriptor for the `workspace_stats` tool.
    ///
    /// The descriptor includes the tool's name, a short description of what it provides,
    /// and the JSON input schema (optionally accepts `include_hidden: bool`).
    ///
    /// # Examples
    ///
    /// ```
    /// let tool = WorkspaceStatsTool::new(service).definition();
    /// assert_eq!(tool.name, "workspace_stats");
    /// ```
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

    /// Execute the workspace statistics tool with the given arguments.
    ///
    /// The `args` map may include an optional `"include_hidden"` boolean; when `true` hidden files and
    /// directories are included in the statistics. On success this returns a `ToolResult` containing a
    /// pretty-printed JSON string of workspace statistics (total files, total lines, per-language
    /// breakdown, and directory count). On failure this returns an error `ToolResult` with a
    /// descriptive message.
    ///
    /// # Parameters
    ///
    /// - `args`: A map of input arguments; recognizes the optional `"include_hidden"` boolean.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use serde_json::json;
    ///
    /// // prepare args to include hidden files
    /// let mut args = HashMap::new();
    /// args.insert("include_hidden".to_string(), json!(true));
    ///
    /// // assume `tool` is an initialized `WorkspaceStatsTool`
    /// // let result = tool.execute(args).await.unwrap();
    /// // println!("{}", result);
    /// ```
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

/// Collects aggregated statistics for the workspace rooted at `root`.
///
/// Scans files and directories under `root` to compute total files, total lines,
/// a per-language breakdown (files and lines), and the number of directories encountered.
/// When `include_hidden` is `true`, hidden files and directories (those starting with a dot)
/// are included in the scan; otherwise they are skipped.
///
/// # Examples
///
/// ```no_run
/// # async fn example() -> anyhow::Result<()> {
/// use std::path::Path;
/// let stats = collect_workspace_stats(Path::new("."), false).await?;
/// // stats contains totals and per-language breakdowns
/// assert!(stats.total_files >= 0);
/// # Ok(()) }
/// ```
///
/// # Returns
///
/// A `WorkspaceStats` value containing totals for files and lines, a language map with
/// per-language file/line counts, and the number of directories scanned.
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

/// Recursively traverses a directory tree and accumulates workspace statistics into the provided `WorkspaceStats`.
///
/// This function walks `path` asynchronously, skipping hidden entries unless `include_hidden` is `true`,
/// and pruning common non-code directories (`node_modules`, `target`, `dist`, `build`, `.git`, `__pycache__`, `venv`).
/// For each regular file, it maps the file extension to a language (via `extension_to_language`), counts lines for
/// recognized source files, and updates `stats` in place: incrementing `total_files`, `total_lines`, per-language
/// `files` and `lines`, and `directories` for visited directories. I/O errors for directories or entries are ignored
/// (those entries are skipped).
///
/// # Parameters
///
/// - `path`: root directory to traverse.
/// - `stats`: mutable accumulator that will be updated with discovered statistics.
/// - `include_hidden`: when `true`, include files and directories whose names start with `.`.
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # use crate::tools::workspace::{collect_stats_recursive, WorkspaceStats};
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let mut stats = WorkspaceStats::default();
/// collect_stats_recursive(Path::new("."), &mut stats, false).await;
/// // `stats` now contains aggregated workspace metrics for the current directory.
/// # });
/// ```
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

/// Count the number of lines in a UTF-8 text file.
///
/// Reads the file at `path` as UTF-8 text and returns the number of newline-separated lines. I/O errors encountered while reading the file are propagated.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::path::Path;
/// use std::env::temp_dir;
/// use std::fs;
///
/// // create a temporary file with three lines
/// let tmp = temp_dir().join("workspace_count_lines_example.txt");
/// fs::write(&tmp, "line1\nline2\nline3\n")?;
///
/// let rt = tokio::runtime::Runtime::new().unwrap();
/// let count = rt.block_on(async { crate::count_lines(&Path::new(&tmp)) })?;
/// assert_eq!(count, 3);
///
/// fs::remove_file(&tmp)?;
/// # Ok(()) }
/// ```
async fn count_lines(path: &Path) -> Result<usize> {
    let content = fs::read_to_string(path).await?;
    Ok(content.lines().count())
}

/// Maps a file extension to a human-readable language label.
///
/// Returns the language label associated with `ext` (for example, `"rs"` -> `"rust"`).
/// Unknown or binary extensions return `"binary"`.
///
/// # Examples
///
/// ```
/// assert_eq!(extension_to_language("rs"), "rust");
/// assert_eq!(extension_to_language("py"), "python");
/// assert_eq!(extension_to_language("tsx"), "react");
/// assert_eq!(extension_to_language("unknown_ext"), "binary");
/// ```
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
    /// Create a new WorkspaceStatsTool that uses the given ContextService.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// // `service` should be an initialized `ContextService` from the application.
    /// let service: Arc<ContextService> = Arc::new(/* ... */);
    /// let tool = WorkspaceStatsTool::new(service);
    /// ```
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for GitStatusTool {
    /// Returns the Tool descriptor for the `git_status` tool.
    ///
    /// The descriptor includes the tool name, a short description of its purpose,
    /// and the JSON input schema (optional `include_diff` boolean).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Create the tool and get its definition:
    /// let tool = GitStatusTool::new(service_arc).definition();
    /// assert_eq!(tool.name, "git_status");
    /// ```
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

    /// Retrieve the workspace git status and optionally include the repository diff.
    ///
    /// Parses `git status --porcelain` in the workspace directory and categorizes files into
    /// `staged`, `unstaged`, and `untracked`. If `include_diff` is true in `args`, also captures
    /// the output of `git diff` and places it in the `diff` field.
    ///
    /// The `args` map may include:
    /// - `"include_diff"`: boolean (optional, defaults to `false`) — when `true`, the tool will try to
    ///   include the output of `git diff` in the returned result.
    ///
    /// # Returns
    ///
    /// Ok containing a `ToolResult` whose success payload is a pretty-printed JSON representation of
    /// the `GitStatus` structure:
    /// - `staged`: list of file paths with staged changes
    /// - `unstaged`: list of file paths with unstaged changes
    /// - `untracked`: list of untracked file paths
    /// - `diff`: optional diff string when requested and available
    ///
    /// If the git commands fail (for example, the workspace is not a git repository), the function
    /// returns an error `ToolResult`.
    ///
    /// # Examples
    ///
    /// ```
    /// // Example illustrating the JSON shape produced by the tool.
    /// use serde_json::json;
    ///
    /// let example = json!({
    ///     "staged": ["src/lib.rs"],
    ///     "unstaged": ["README.md"],
    ///     "untracked": ["tmp/new_file.txt"],
    ///     "diff": null
    /// });
    ///
    /// let pretty = serde_json::to_string_pretty(&example).unwrap();
    /// assert!(pretty.contains("\"staged\""));
    /// ```
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
    /// Create a new WorkspaceStatsTool that uses the given ContextService.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// // `service` should be an initialized `ContextService` from the application.
    /// let service: Arc<ContextService> = Arc::new(/* ... */);
    /// let tool = WorkspaceStatsTool::new(service);
    /// ```
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for ExtractSymbolsTool {
    /// Returns the tool descriptor for the extract_symbols tool.
    ///
    /// The descriptor includes the tool's name, a short description of its behavior,
    /// and the JSON input schema (requiring `file_path`) used to invoke the tool.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// // Construct a ContextService appropriately in your application.
    /// let service = Arc::new(ContextService::new());
    /// let tool = ExtractSymbolsTool::new(service).definition();
    /// assert_eq!(tool.name, "extract_symbols");
    /// assert!(tool.input_schema.get("required").and_then(|r| r.as_array()).is_some());
    /// ```
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

    /// Extracts symbols from a file inside the workspace and returns them as a JSON-formatted ToolResult.
    ///
    /// The method expects `args` to contain a `"file_path"` key with a path relative to the workspace root.
    /// It verifies the resolved path does not escape the workspace, reads the file, detects symbols based on the
    /// file extension, and returns a pretty-printed JSON object with the keys:
    /// - `file`: the supplied relative file path
    /// - `symbols`: an array of detected `Symbol` objects (name, kind, line, optional signature)
    ///
    /// # Parameters
    ///
    /// - `args`: A map of input arguments; must include `"file_path"` (string) pointing to a file within the workspace.
    ///
    /// # Returns
    ///
    /// A `ToolResult` containing a pretty-printed JSON object with the `file` and `symbols` fields. On failure the
    /// returned `ToolResult` contains an error message describing the problem (e.g., path resolution or read error).
    ///
    /// # Examples
    ///
    /// ```
    /// // Given file content, `extract_symbols_from_content` demonstrates the expected symbol extraction.
    /// let content = "pub struct Foo {}\n\npub fn bar() {}";
    /// let symbols = extract_symbols_from_content(content, "rs");
    /// assert!(symbols.iter().any(|s| s.kind == "struct" && s.name == "Foo"));
    /// assert!(symbols.iter().any(|s| s.kind == "function" && s.name == "bar"));
    /// ```
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

/// Extracts symbol definitions from the given source text for the specified file extension.
///
/// Scans the content line-by-line and returns a vector of detected `Symbol` entries
/// (each with name, kind, line number, and optional signature) appropriate for the
/// language indicated by `ext`.
///
/// # Examples
///
/// ```
/// let src = "pub struct Foo {}\nfn bar() {}\n";
/// let syms = extract_symbols_from_content(src, "rs");
/// assert_eq!(syms.len(), 2);
/// assert_eq!(syms[0].name, "Foo");
/// assert_eq!(syms[0].kind, "struct");
/// assert_eq!(syms[1].name, "bar");
/// assert_eq!(syms[1].kind, "function");
/// ```
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

/// Dispatches a single source line to the language-specific symbol detector based on the file extension.
///
/// Supported extensions: "rs" (Rust), "py" (Python), "ts", "tsx", "js", "jsx" (TypeScript/JavaScript), and "go" (Go).
///
/// # Returns
///
/// `Some(Symbol)` if the line contains a recognized symbol for the given language, `None` otherwise.
///
/// # Examples
///
/// ```
/// let line = "pub fn hello(name: &str) -> String {";
/// let sym = detect_symbol(line, "rs", 1);
/// assert!(sym.is_some());
/// ```
fn detect_symbol(line: &str, ext: &str, line_num: usize) -> Option<Symbol> {
    match ext {
        "rs" => detect_rust_symbol(line, line_num),
        "py" => detect_python_symbol(line, line_num),
        "ts" | "tsx" | "js" | "jsx" => detect_ts_symbol(line, line_num),
        "go" => detect_go_symbol(line, line_num),
        _ => None,
    }
}

/// Detects a top-level Rust symbol declared on a single source line.
///
/// Recognizes `function`, `struct`, `enum`, `trait`, and `impl` declarations. When a symbol is found,
/// returns a `Symbol` with its `name`, `kind`, `line`, and an optional `signature` (present for functions and impls).
///
/// # Examples
///
/// ```
/// let s = detect_rust_symbol("pub fn add(a: i32, b: i32) -> i32 {", 3).unwrap();
/// assert_eq!(s.name, "add");
/// assert_eq!(s.kind, "function");
/// assert_eq!(s.line, 3);
///
/// let st = detect_rust_symbol("struct Point {", 10).unwrap();
/// assert_eq!(st.name, "Point");
/// assert_eq!(st.kind, "struct");
/// ```
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

/// Detects a top-level Python symbol declaration on a single line.
///
/// Analyzes the provided line for `def`, `async def`, or `class` declarations and,
/// when found, returns a `Symbol` containing the identifier name, kind, source line
/// number, and an optional signature (the full line) for functions.
///
/// # Parameters
///
/// - `line`: the source line to inspect.
/// - `line_num`: the 1-based line number to record in the returned `Symbol`.
///
/// # Returns
///
/// `Some(Symbol)` when the line declares a Python function, async function, or class;
/// `None` otherwise.
///
/// # Examples
///
/// ```
/// let s = detect_python_symbol("def foo(bar):", 1).unwrap();
/// assert_eq!(s.name, "foo");
/// assert_eq!(s.kind, "function");
/// assert_eq!(s.line, 1);
/// assert_eq!(s.signature.unwrap(), "def foo(bar):".to_string());
/// ```
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

/// Detects a TypeScript/JavaScript symbol declaration on a single source line.
///
/// The function recognizes `function`, `class`, `interface`, and `type` declarations
/// (including `export` variants) and returns the corresponding symbol metadata.
///
/// # Returns
///
/// `Some(Symbol)` containing the detected symbol's name, kind, line number, and an
/// optional signature (present for `function`), or `None` if no supported declaration is found.
///
/// # Examples
///
/// ```
/// let line = "export class MyComponent {";
/// let sym = detect_ts_symbol(line, 10).unwrap();
/// assert_eq!(sym.name, "MyComponent");
/// assert_eq!(sym.kind, "class");
/// assert_eq!(sym.line, 10);
/// ```
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

/// Detects top-level Go symbols on a single source line.
///
/// Recognizes function declarations (including methods), `type ... struct` and `type ... interface`
/// declarations and returns a corresponding `Symbol`.
///
/// # Parameters
///
/// - `line`: the source code line to analyze.
/// - `line_num`: the 1-based line number where `line` appears.
///
/// # Returns
///
/// `Some(Symbol)` when a Go symbol is found, `None` otherwise.
///
/// # Examples
///
/// ```
/// let line_fn = "func Add(a int, b int) int {";
/// let sym = detect_go_symbol(line_fn, 10).unwrap();
/// assert_eq!(sym.name, "Add");
/// assert_eq!(sym.kind, "function");
/// assert_eq!(sym.line, 10);
///
/// let line_method = "func (r *Repo) Save(item Item) error {";
/// let sym = detect_go_symbol(line_method, 20).unwrap();
/// assert_eq!(sym.name, "Save");
/// assert_eq!(sym.kind, "function");
///
/// let line_struct = "type User struct {";
/// let sym = detect_go_symbol(line_struct, 30).unwrap();
/// assert_eq!(sym.name, "User");
/// assert_eq!(sym.kind, "struct");
///
/// let line_iface = "type Reader interface {";
/// let sym = detect_go_symbol(line_iface, 40).unwrap();
/// assert_eq!(sym.name, "Reader");
/// assert_eq!(sym.kind, "interface");
/// ```
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

/// Extracts the identifier immediately following a given prefix in a line.
///
/// The function finds the first occurrence of `prefix` in `line` and returns the contiguous
/// sequence of ASCII letters, digits, or underscores that follows it. If the prefix is not
/// present or no valid identifier follows, an empty string is returned.
///
/// # Examples
///
/// ```
/// assert_eq!(extract_name("pub fn hello_world()", "fn "), "hello_world");
/// assert_eq!(extract_name("impl<T> MyType<T> {", "impl "), "MyType");
/// assert_eq!(extract_name("let x = 1;", "const "), "");
/// ```
fn extract_name(line: &str, prefix: &str) -> String {
    line.split(prefix)
        .nth(1)
        .unwrap_or("")
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .next()
        .unwrap_or("")
        .to_string()
}

// ===== Git Tools =====

/// Git blame tool - show blame information for a file.
pub struct GitBlameTool {
    service: Arc<ContextService>,
}

impl GitBlameTool {
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for GitBlameTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "git_blame".to_string(),
            description:
                "Show git blame information for a file, revealing who last modified each line."
                    .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the file (relative to workspace)"
                    },
                    "start_line": {
                        "type": "integer",
                        "description": "Start line number (optional, 1-based)"
                    },
                    "end_line": {
                        "type": "integer",
                        "description": "End line number (optional, 1-based)"
                    }
                },
                "required": ["file_path"]
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let file_path = get_string_arg(&args, "file_path")?;
        let start_line = args
            .get("start_line")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        let end_line = args
            .get("end_line")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        let workspace = self.service.workspace_path();

        // Build git blame command
        let mut cmd = Command::new("git");
        cmd.current_dir(workspace);
        cmd.args(["blame", "--line-porcelain"]);

        if let (Some(start), Some(end)) = (start_line, end_line) {
            cmd.arg(format!("-L{},{}", start, end));
        } else if let Some(start) = start_line {
            cmd.arg(format!("-L{},", start));
        }

        cmd.arg(&file_path);

        let output = cmd.output().await;

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let blame_info = parse_git_blame_porcelain(&stdout);
                    Ok(success_result(
                        serde_json::to_string_pretty(&blame_info).unwrap(),
                    ))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Ok(error_result(format!("git blame failed: {}", stderr)))
                }
            }
            Err(e) => Ok(error_result(format!("Failed to run git: {}", e))),
        }
    }
}

#[derive(serde::Serialize)]
struct BlameEntry {
    commit: String,
    author: String,
    date: String,
    line_number: usize,
    content: String,
}

fn parse_git_blame_porcelain(output: &str) -> Vec<BlameEntry> {
    let mut entries = Vec::new();
    let mut current_commit = String::new();
    let mut current_author = String::new();
    let mut current_date = String::new();
    let mut current_line = 0usize;
    let mut in_entry = false;

    for line in output.lines() {
        if line.len() >= 40 && line.chars().take(40).all(|c| c.is_ascii_hexdigit()) {
            // New commit line: <sha> <orig_line> <final_line> [<num_lines>]
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                current_commit = parts[0][..8].to_string(); // Short SHA
                current_line = parts[2].parse().unwrap_or(0);
                in_entry = true;
            }
        } else if in_entry {
            if let Some(author) = line.strip_prefix("author ") {
                current_author = author.to_string();
            } else if let Some(time) = line.strip_prefix("author-time ") {
                // Convert Unix timestamp to date
                if let Ok(ts) = time.parse::<i64>() {
                    current_date = chrono::DateTime::from_timestamp(ts, 0)
                        .map(|dt| dt.format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|| time.to_string());
                }
            } else if let Some(content) = line.strip_prefix('\t') {
                // Content line
                entries.push(BlameEntry {
                    commit: current_commit.clone(),
                    author: current_author.clone(),
                    date: current_date.clone(),
                    line_number: current_line,
                    content: content.to_string(),
                });
                in_entry = false;
            }
        }
    }

    entries
}

/// Git log tool - show commit history.
pub struct GitLogTool {
    service: Arc<ContextService>,
}

impl GitLogTool {
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for GitLogTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "git_log".to_string(),
            description:
                "Show git commit history with optional filtering by file, author, or date range."
                    .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Filter commits affecting this file (optional)"
                    },
                    "author": {
                        "type": "string",
                        "description": "Filter by author name or email (optional)"
                    },
                    "since": {
                        "type": "string",
                        "description": "Show commits after this date (e.g., '2024-01-01', '1 week ago')"
                    },
                    "until": {
                        "type": "string",
                        "description": "Show commits before this date"
                    },
                    "max_count": {
                        "type": "integer",
                        "description": "Maximum number of commits to show (default: 20)"
                    },
                    "grep": {
                        "type": "string",
                        "description": "Filter commits by message pattern"
                    }
                },
                "required": []
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let file_path = args.get("file_path").and_then(|v| v.as_str());
        let author = args.get("author").and_then(|v| v.as_str());
        let since = args.get("since").and_then(|v| v.as_str());
        let until = args.get("until").and_then(|v| v.as_str());
        let grep = args.get("grep").and_then(|v| v.as_str());
        let max_count = args.get("max_count").and_then(|v| v.as_u64()).unwrap_or(20) as usize;

        let workspace = self.service.workspace_path();

        let mut cmd = Command::new("git");
        cmd.current_dir(workspace);
        cmd.args([
            "log",
            "--format=%H|%an|%ae|%aI|%s",
            &format!("-{}", max_count),
        ]);

        if let Some(author) = author {
            cmd.arg(format!("--author={}", author));
        }
        if let Some(since) = since {
            cmd.arg(format!("--since={}", since));
        }
        if let Some(until) = until {
            cmd.arg(format!("--until={}", until));
        }
        if let Some(grep) = grep {
            cmd.arg(format!("--grep={}", grep));
        }
        if let Some(file) = file_path {
            cmd.arg("--").arg(file);
        }

        let output = cmd.output().await;

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let commits: Vec<CommitInfo> = stdout
                        .lines()
                        .filter_map(|line| {
                            let parts: Vec<&str> = line.splitn(5, '|').collect();
                            if parts.len() == 5 {
                                Some(CommitInfo {
                                    sha: parts[0][..8].to_string(),
                                    full_sha: parts[0].to_string(),
                                    author_name: parts[1].to_string(),
                                    author_email: parts[2].to_string(),
                                    date: parts[3].to_string(),
                                    message: parts[4].to_string(),
                                })
                            } else {
                                None
                            }
                        })
                        .collect();

                    Ok(success_result(
                        serde_json::to_string_pretty(&commits).unwrap(),
                    ))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Ok(error_result(format!("git log failed: {}", stderr)))
                }
            }
            Err(e) => Ok(error_result(format!("Failed to run git: {}", e))),
        }
    }
}

#[derive(serde::Serialize)]
struct CommitInfo {
    sha: String,
    full_sha: String,
    author_name: String,
    author_email: String,
    date: String,
    message: String,
}

/// Dependency graph tool - analyze file/module dependencies.
pub struct DependencyGraphTool {
    service: Arc<ContextService>,
}

impl DependencyGraphTool {
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for DependencyGraphTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "dependency_graph".to_string(),
            description: "Analyze and visualize file/module dependencies. Returns import/use relationships between files.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Analyze dependencies for this specific file (optional)"
                    },
                    "direction": {
                        "type": "string",
                        "enum": ["imports", "imported_by", "both"],
                        "description": "Direction of dependencies: 'imports' (what this file imports), 'imported_by' (what imports this file), or 'both' (default: 'imports')"
                    },
                    "depth": {
                        "type": "integer",
                        "description": "Maximum depth for transitive dependencies (default: 1)"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["json", "mermaid"],
                        "description": "Output format: 'json' or 'mermaid' diagram (default: 'json')"
                    }
                },
                "required": []
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let file_path = args.get("file_path").and_then(|v| v.as_str());
        let direction = args
            .get("direction")
            .and_then(|v| v.as_str())
            .unwrap_or("imports");
        let depth = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
        let format = args
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("json");

        let workspace = self.service.workspace_path();

        if let Some(file) = file_path {
            // Analyze specific file
            let full_path = workspace.join(file);
            let content = match fs::read_to_string(&full_path).await {
                Ok(c) => c,
                Err(e) => return Ok(error_result(format!("Failed to read file: {}", e))),
            };

            let ext = full_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let imports = extract_imports(&content, ext);

            let result = DependencyResult {
                file: file.to_string(),
                imports: imports.clone(),
                imported_by: if direction == "imported_by" || direction == "both" {
                    find_importers(workspace, file, depth).await
                } else {
                    vec![]
                },
            };

            if format == "mermaid" {
                let mermaid = generate_mermaid_graph(&result);
                Ok(success_result(mermaid))
            } else {
                Ok(success_result(
                    serde_json::to_string_pretty(&result).unwrap(),
                ))
            }
        } else {
            // Analyze entire workspace (limited)
            let mut all_deps: HashMap<String, Vec<String>> = HashMap::new();
            let files = collect_source_files(workspace, 100).await;

            for file in files {
                if let Ok(content) = fs::read_to_string(&file).await {
                    let relative = file.strip_prefix(workspace).unwrap_or(&file);
                    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
                    let imports = extract_imports(&content, ext);
                    if !imports.is_empty() {
                        all_deps.insert(relative.to_string_lossy().to_string(), imports);
                    }
                }
            }

            if format == "mermaid" {
                let mermaid = generate_workspace_mermaid(&all_deps);
                Ok(success_result(mermaid))
            } else {
                Ok(success_result(
                    serde_json::to_string_pretty(&all_deps).unwrap(),
                ))
            }
        }
    }
}

#[derive(serde::Serialize)]
struct DependencyResult {
    file: String,
    imports: Vec<String>,
    imported_by: Vec<String>,
}

fn extract_imports(content: &str, ext: &str) -> Vec<String> {
    let mut imports = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        match ext {
            "rs" => {
                // Rust: use crate::..., mod ..., use super::...
                if let Some(rest) = trimmed.strip_prefix("use ") {
                    let module = rest.split(';').next().unwrap_or("").trim();
                    if !module.is_empty() {
                        imports.push(module.to_string());
                    }
                } else if let Some(rest) = trimmed.strip_prefix("mod ") {
                    let module = rest.split(';').next().unwrap_or("").trim();
                    if !module.is_empty() && !trimmed.contains('{') {
                        imports.push(format!("mod {}", module));
                    }
                }
            }
            "py" => {
                // Python: import ..., from ... import ...
                if let Some(rest) = trimmed.strip_prefix("import ") {
                    imports.push(rest.split('#').next().unwrap_or("").trim().to_string());
                } else if let Some(rest) = trimmed.strip_prefix("from ") {
                    let parts: Vec<&str> = rest.split(" import ").collect();
                    if !parts.is_empty() {
                        imports.push(parts[0].trim().to_string());
                    }
                }
            }
            "ts" | "tsx" | "js" | "jsx" => {
                // TypeScript/JavaScript: import ... from '...'
                if trimmed.contains("import ") && trimmed.contains(" from ") {
                    if let Some(start) = trimmed.find(" from ") {
                        let rest = &trimmed[start + 7..];
                        let module = rest
                            .trim_start_matches(['\'', '"'])
                            .split(['\'', '"'])
                            .next()
                            .unwrap_or("");
                        if !module.is_empty() {
                            imports.push(module.to_string());
                        }
                    }
                } else if let Some(rest) = trimmed.strip_prefix("require(") {
                    let module = rest
                        .trim_start_matches(['\'', '"'])
                        .split(['\'', '"', ')'])
                        .next()
                        .unwrap_or("");
                    if !module.is_empty() {
                        imports.push(module.to_string());
                    }
                }
            }
            "go" => {
                // Go: import "..." or import (...)
                if let Some(rest) = trimmed.strip_prefix("import ") {
                    let module = rest.trim_start_matches('"').split('"').next().unwrap_or("");
                    if !module.is_empty() {
                        imports.push(module.to_string());
                    }
                } else if trimmed.starts_with('"') && trimmed.ends_with('"') {
                    // Inside import block
                    let module = trimmed.trim_matches('"');
                    if !module.is_empty() {
                        imports.push(module.to_string());
                    }
                }
            }
            _ => {}
        }
    }

    imports
}

async fn find_importers(workspace: &Path, target_file: &str, _depth: usize) -> Vec<String> {
    let mut importers = Vec::new();
    let files = collect_source_files(workspace, 200).await;

    // Extract the module name from the target file
    let target_module = Path::new(target_file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    for file in files {
        if let Ok(content) = fs::read_to_string(&file).await {
            let relative = file.strip_prefix(workspace).unwrap_or(&file);
            let relative_str = relative.to_string_lossy();

            // Skip the target file itself
            if relative_str == target_file {
                continue;
            }

            // Check if this file imports the target
            if content.contains(target_module) || content.contains(target_file) {
                importers.push(relative_str.to_string());
            }
        }
    }

    importers
}

async fn collect_source_files(dir: &Path, limit: usize) -> Vec<std::path::PathBuf> {
    use tokio::fs::read_dir;

    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        if files.len() >= limit {
            break;
        }

        if let Ok(mut entries) = read_dir(&current).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                // Skip hidden and common non-source directories
                if name.starts_with('.')
                    || matches!(
                        name.as_str(),
                        "node_modules" | "target" | "dist" | "build" | "__pycache__"
                    )
                {
                    continue;
                }

                if path.is_dir() {
                    stack.push(path);
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if matches!(ext, "rs" | "py" | "ts" | "tsx" | "js" | "jsx" | "go") {
                        files.push(path);
                        if files.len() >= limit {
                            break;
                        }
                    }
                }
            }
        }
    }

    files
}

fn generate_mermaid_graph(result: &DependencyResult) -> String {
    let mut mermaid = String::from("```mermaid\ngraph LR\n");
    let file_id = sanitize_mermaid_id(&result.file);

    for import in &result.imports {
        let import_id = sanitize_mermaid_id(import);
        mermaid.push_str(&format!(
            "    {}[\"{}\"] --> {}[\"{}\"]\n",
            file_id, result.file, import_id, import
        ));
    }

    for importer in &result.imported_by {
        let importer_id = sanitize_mermaid_id(importer);
        mermaid.push_str(&format!(
            "    {}[\"{}\"] --> {}[\"{}\"]\n",
            importer_id, importer, file_id, result.file
        ));
    }

    mermaid.push_str("```");
    mermaid
}

fn generate_workspace_mermaid(deps: &HashMap<String, Vec<String>>) -> String {
    let mut mermaid = String::from("```mermaid\ngraph LR\n");

    for (file, imports) in deps {
        let file_id = sanitize_mermaid_id(file);
        for import in imports {
            let import_id = sanitize_mermaid_id(import);
            mermaid.push_str(&format!("    {} --> {}\n", file_id, import_id));
        }
    }

    mermaid.push_str("```");
    mermaid
}

fn sanitize_mermaid_id(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

/// File outline tool - get structured outline of a file.
pub struct FileOutlineTool {
    service: Arc<ContextService>,
}

impl FileOutlineTool {
    pub fn new(service: Arc<ContextService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for FileOutlineTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "file_outline".to_string(),
            description: "Get a structured outline of a file showing all symbols (functions, classes, structs, etc.) with their line numbers.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the file (relative to workspace)"
                    },
                    "include_private": {
                        "type": "boolean",
                        "description": "Include private/internal symbols (default: true)"
                    }
                },
                "required": ["file_path"]
            }),
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let file_path = get_string_arg(&args, "file_path")?;
        let include_private = args
            .get("include_private")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let workspace = self.service.workspace_path();
        let full_path = workspace.join(&file_path);

        // Security check
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

        let ext = path_canonical
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let mut symbols = extract_symbols_from_content(&content, ext);

        // Filter private symbols if requested
        if !include_private {
            symbols.retain(|s| {
                s.signature
                    .as_ref()
                    .map(|sig| sig.contains("pub "))
                    .unwrap_or(true)
            });
        }

        // Group by kind
        let mut grouped: HashMap<String, Vec<&Symbol>> = HashMap::new();
        for sym in &symbols {
            grouped.entry(sym.kind.clone()).or_default().push(sym);
        }

        let outline = FileOutline {
            file: file_path,
            language: extension_to_language(ext).to_string(),
            total_lines: content.lines().count(),
            symbols: symbols.len(),
            outline: grouped
                .into_iter()
                .map(|(kind, syms)| OutlineSection {
                    kind,
                    count: syms.len(),
                    items: syms
                        .into_iter()
                        .map(|s| OutlineItem {
                            name: s.name.clone(),
                            line: s.line,
                            signature: s.signature.clone(),
                        })
                        .collect(),
                })
                .collect(),
        };

        Ok(success_result(
            serde_json::to_string_pretty(&outline).unwrap(),
        ))
    }
}

#[derive(serde::Serialize)]
struct FileOutline {
    file: String,
    language: String,
    total_lines: usize,
    symbols: usize,
    outline: Vec<OutlineSection>,
}

#[derive(serde::Serialize)]
struct OutlineSection {
    kind: String,
    count: usize,
    items: Vec<OutlineItem>,
}

#[derive(serde::Serialize)]
struct OutlineItem {
    name: String,
    line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature: Option<String>,
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

    // Tests for new tools

    #[test]
    fn test_extract_imports_rust() {
        let code = r#"
use std::collections::HashMap;
use crate::error::Result;
mod handler;
"#;
        let imports = extract_imports(code, "rs");
        assert!(imports.contains(&"std::collections::HashMap".to_string()));
        assert!(imports.contains(&"crate::error::Result".to_string()));
        assert!(imports.contains(&"mod handler".to_string()));
    }

    #[test]
    fn test_extract_imports_python() {
        let code = r#"
import os
from pathlib import Path
import json
"#;
        let imports = extract_imports(code, "py");
        assert!(imports.contains(&"os".to_string()));
        assert!(imports.contains(&"pathlib".to_string()));
        assert!(imports.contains(&"json".to_string()));
    }

    #[test]
    fn test_extract_imports_typescript() {
        let code = r#"
import { useState } from 'react';
import axios from 'axios';
require('lodash');
"#;
        let imports = extract_imports(code, "ts");
        assert!(imports.contains(&"react".to_string()));
        assert!(imports.contains(&"axios".to_string()));
        assert!(imports.contains(&"lodash".to_string()));
    }

    #[test]
    fn test_extract_imports_go() {
        let code = r#"
import "fmt"
import (
    "os"
    "path/filepath"
)
"#;
        let imports = extract_imports(code, "go");
        assert!(imports.contains(&"fmt".to_string()));
        assert!(imports.contains(&"os".to_string()));
        assert!(imports.contains(&"path/filepath".to_string()));
    }

    #[test]
    fn test_sanitize_mermaid_id() {
        assert_eq!(sanitize_mermaid_id("src/main.rs"), "src_main_rs");
        assert_eq!(sanitize_mermaid_id("foo-bar"), "foo_bar");
        assert_eq!(sanitize_mermaid_id("test123"), "test123");
    }

    #[test]
    fn test_generate_mermaid_graph() {
        let result = DependencyResult {
            file: "main.rs".to_string(),
            imports: vec!["lib.rs".to_string()],
            imported_by: vec![],
        };
        let mermaid = generate_mermaid_graph(&result);
        assert!(mermaid.contains("```mermaid"));
        assert!(mermaid.contains("graph LR"));
        assert!(mermaid.contains("main_rs"));
        assert!(mermaid.contains("lib_rs"));
    }

    #[test]
    fn test_parse_git_blame_porcelain() {
        // Minimal porcelain format
        let output = "abc123def456789012345678901234567890abcd 1 1 1\n\
author John Doe\n\
author-time 1704067200\n\
\tHello World\n";
        let entries = parse_git_blame_porcelain(output);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].commit, "abc123de");
        assert_eq!(entries[0].author, "John Doe");
        assert_eq!(entries[0].content, "Hello World");
    }

    #[test]
    fn test_parse_git_blame_empty() {
        let entries = parse_git_blame_porcelain("");
        assert!(entries.is_empty());
    }
}
