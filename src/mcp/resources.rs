//! MCP Resources Support
//!
//! Expose indexed files as MCP resources that AI clients can browse and read.

use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

use crate::error::{Error, Result};
use crate::service::ContextService;

/// Decode a percent-encoded file:// URI path to a PathBuf.
///
/// Handles percent-encoded characters like `%20` (space) and properly converts
/// the decoded string to a filesystem path.
fn decode_file_uri(uri: &str) -> Option<PathBuf> {
    uri.strip_prefix("file://").map(|path| {
        let decoded = percent_decode_str(path).decode_utf8_lossy();
        PathBuf::from(decoded.as_ref())
    })
}

/// A resource exposed by the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Resource contents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceContents {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>, // base64 encoded
}

/// Result of resources/list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResourcesResult {
    pub resources: Vec<Resource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Result of resources/read.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceResult {
    pub contents: Vec<ResourceContents>,
}

/// Resource registry and manager.
pub struct ResourceRegistry {
    context_service: Arc<ContextService>,
    subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>, // uri -> session_ids
}

impl ResourceRegistry {
    /// Creates a new ResourceRegistry backed by the given workspace context.
    ///
    /// # Parameters
    ///
    /// - `context_service`: shared workspace context used to resolve the workspace root and related operations.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// # use crate::mcp::resources::ResourceRegistry;
    /// # use crate::context::ContextService;
    /// // Construct or obtain an Arc<ContextService> from your application.
    /// let ctx: Arc<ContextService> = Arc::new(ContextService::default());
    /// let registry = ResourceRegistry::new(ctx);
    /// ```
    pub fn new(context_service: Arc<ContextService>) -> Self {
        Self {
            context_service,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Lists workspace files as `Resource` entries with optional cursor-based pagination.
    ///
    /// The `cursor` parameter, if provided, is a resource name to start listing after; results include up to 100 entries.
    ///
    /// # Returns
    ///
    /// `ListResourcesResult` containing the discovered resources and an optional `next_cursor` string to continue pagination.
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// // `registry` would be constructed with a real `ContextService` in production.
    /// // let registry = ResourceRegistry::new(context_service);
    /// // let result = registry.list(None).await.unwrap();
    /// // assert!(result.resources.len() <= 100);
    /// # });
    /// ```
    pub async fn list(&self, cursor: Option<&str>) -> Result<ListResourcesResult> {
        let workspace = self.context_service.workspace();
        let files = self.discover_files(workspace, 100, cursor).await?;

        let resources: Vec<Resource> = files
            .iter()
            .map(|path| {
                let relative = path
                    .strip_prefix(workspace)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();

                // Construct proper file:// URI (handle Windows paths)
                let uri = Self::path_to_file_uri(path);
                let mime_type = Self::guess_mime_type(path);

                Resource {
                    uri,
                    name: relative.clone(),
                    description: Some(format!("File: {}", relative)),
                    mime_type,
                }
            })
            .collect();

        // Simple pagination - if we got max results, there might be more
        let next_cursor = if resources.len() >= 100 {
            resources.last().map(|r| r.name.clone())
        } else {
            None
        };

        Ok(ListResourcesResult {
            resources,
            next_cursor,
        })
    }

    /// Reads a resource identified by a `file://` URI from the workspace and returns its contents.
    ///
    /// # Arguments
    ///
    /// * `uri` - A `file://` URI pointing to a file located inside the workspace.
    ///
    /// # Returns
    ///
    /// A `ReadResourceResult` containing a single `ResourceContents` entry with the provided `uri`, the inferred `mime_type` (if any), and `text` set to the file's UTF-8 contents.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidToolArguments` when:
    /// - the URI does not start with `file://`,
    /// - the workspace or target path cannot be canonicalized,
    /// - the resolved path is outside the workspace, or
    /// - the file cannot be read.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn example_usage(registry: &crate::mcp::resources::ResourceRegistry) -> anyhow::Result<()> {
    /// let result = registry.read("file:///path/to/workspace/file.txt").await?;
    /// assert_eq!(result.contents.len(), 1);
    /// let content = &result.contents[0];
    /// assert_eq!(content.uri, "file:///path/to/workspace/file.txt");
    /// # Ok(()) }
    /// ```
    pub async fn read(&self, uri: &str) -> Result<ReadResourceResult> {
        // Parse and decode file:// URI (handles percent-encoded characters like %20)
        let path = decode_file_uri(uri)
            .ok_or_else(|| Error::InvalidToolArguments(format!("Invalid URI scheme: {}", uri)))?;

        // Security: canonicalize both workspace and path, then verify path is within workspace
        let workspace = self.context_service.workspace();
        let workspace_canonical = workspace
            .canonicalize()
            .map_err(|e| Error::InvalidToolArguments(format!("Cannot resolve workspace: {}", e)))?;
        let canonical = path
            .canonicalize()
            .map_err(|e| Error::InvalidToolArguments(format!("Cannot resolve path: {}", e)))?;

        if !canonical.starts_with(&workspace_canonical) {
            return Err(Error::InvalidToolArguments(
                "Access denied: path outside workspace".to_string(),
            ));
        }

        // Read file
        let content = fs::read_to_string(&canonical)
            .await
            .map_err(|e| Error::InvalidToolArguments(format!("Cannot read file: {}", e)))?;

        let mime_type = Self::guess_mime_type(&canonical);

        Ok(ReadResourceResult {
            contents: vec![ResourceContents {
                uri: uri.to_string(),
                mime_type,
                text: Some(content),
                blob: None,
            }],
        })
    }

    /// Registers a session to receive change notifications for the given resource URI.
    ///
    /// The session ID will be recorded in the registry's in-memory subscription map for the specified URI.
    ///
    /// # Parameters
    ///
    /// - `uri`: The resource URI to subscribe to (e.g., a `file://` URI).
    /// - `session_id`: The identifier of the session to register for notifications.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use tokio::runtime::Runtime;
    /// # async fn _example(registry: &crate::mcp::resources::ResourceRegistry) {
    /// registry.subscribe("file:///path/to/file", "session-123").await.unwrap();
    /// # }
    /// ```
    pub async fn subscribe(&self, uri: &str, session_id: &str) -> Result<()> {
        let mut subs = self.subscriptions.write().await;
        let sessions = subs.entry(uri.to_string()).or_default();
        // Prevent duplicate subscriptions from the same session
        if !sessions.contains(&session_id.to_string()) {
            sessions.push(session_id.to_string());
        }
        Ok(())
    }

    /// Remove a session's subscription for the given resource URI.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::sync::Arc;
    /// # use tokio::runtime::Runtime;
    /// # // Assume `registry` is an initialized `ResourceRegistry`.
    /// # let rt = Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// let registry = /* ResourceRegistry instance */ unimplemented!();
    /// registry.unsubscribe("file:///path/to/file", "session-123").await.unwrap();
    /// # });
    /// ```
    pub async fn unsubscribe(&self, uri: &str, session_id: &str) -> Result<()> {
        let mut subs = self.subscriptions.write().await;
        if let Some(sessions) = subs.get_mut(uri) {
            sessions.retain(|s| s != session_id);
        }
        Ok(())
    }

    /// Maximum recursion depth for file discovery to prevent excessive traversal.
    const MAX_DISCOVERY_DEPTH: usize = 20;

    /// Discover files in directory (with pagination and depth limit).
    async fn discover_files(
        &self,
        dir: &std::path::Path,
        limit: usize,
        after: Option<&str>,
    ) -> Result<Vec<PathBuf>> {
        use tokio::fs::read_dir;

        let mut files = Vec::new();
        // Stack contains (path, depth) tuples
        let mut stack = vec![(dir.to_path_buf(), 0usize)];
        let mut past_cursor = after.is_none();

        while let Some((current, depth)) = stack.pop() {
            if files.len() >= limit {
                break;
            }

            // Skip if we've exceeded the maximum depth
            if depth > Self::MAX_DISCOVERY_DEPTH {
                continue;
            }

            let mut entries = match read_dir(&current).await {
                Ok(e) => e,
                Err(_) => continue,
            };

            while let Ok(Some(entry)) = entries.next_entry().await {
                if files.len() >= limit {
                    break;
                }

                let path = entry.path();
                let name = path.file_name().unwrap_or_default().to_string_lossy();

                // Skip hidden files and common ignore patterns
                if name.starts_with('.') || Self::should_ignore(&name) {
                    continue;
                }

                // Use async file_type() instead of blocking is_dir()/is_file()
                let file_type = match entry.file_type().await {
                    Ok(ft) => ft,
                    Err(_) => continue,
                };

                if file_type.is_dir() {
                    stack.push((path, depth + 1));
                } else if file_type.is_file() {
                    let relative = path
                        .strip_prefix(dir)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();

                    // Handle cursor pagination
                    if !past_cursor {
                        if Some(relative.as_str()) == after {
                            past_cursor = true;
                        }
                        continue;
                    }

                    files.push(path);
                }
            }
        }

        // Sort files for deterministic pagination order
        files.sort();

        Ok(files)
    }

    /// Returns whether a file or directory name matches common ignore patterns used when discovering files.
    ///
    /// Matches directory names: "node_modules", "target", "dist", "build", "__pycache__", ".git",
    /// and files whose names end with `.lock` or `.pyc`.
    ///
    /// # Examples
    ///
    /// ```
    /// assert!(should_ignore("node_modules"));
    /// assert!(should_ignore("Cargo.lock"));
    /// assert!(should_ignore("__pycache__"));
    /// assert!(!should_ignore("src"));
    /// ```
    fn should_ignore(name: &str) -> bool {
        matches!(
            name,
            "node_modules" | "target" | "dist" | "build" | "__pycache__" | ".git"
        ) || name.ends_with(".lock")
            || name.ends_with(".pyc")
    }

    /// Convert a filesystem path to a file:// URI.
    ///
    /// On Windows this replaces backslashes with forward slashes and prefixes
    /// absolute drive paths (e.g., `C:/path`) with `file:///`. On other platforms
    /// the path is prefixed with `file://`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// let uri = path_to_file_uri(Path::new("/some/path"));
    /// assert!(uri.starts_with("file://"));
    /// ```
    fn path_to_file_uri(path: &std::path::Path) -> String {
        let path_str = path.to_string_lossy();

        // On Windows, paths like C:\foo\bar need to become file:///C:/foo/bar
        #[cfg(windows)]
        {
            let normalized = path_str.replace('\\', "/");
            if normalized.chars().nth(1) == Some(':') {
                // Absolute Windows path like C:/foo
                format!("file:///{}", normalized)
            } else {
                format!("file://{}", normalized)
            }
        }

        #[cfg(not(windows))]
        {
            format!("file://{}", path_str)
        }
    }

    /// Infer a MIME type string for a file path based on its extension.
    ///
    /// Returns `Some` with a guessed MIME type for known extensions, `Some("text/plain")` for unknown extensions,
    /// and `None` if the path has no extension or the extension is not valid UTF-8.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// assert_eq!(guess_mime_type(Path::new("main.rs")), Some("text/x-rust".to_string()));
    /// assert_eq!(guess_mime_type(Path::new("data.unknown")), Some("text/plain".to_string()));
    /// assert_eq!(guess_mime_type(Path::new("no_extension")), None);
    /// ```
    fn guess_mime_type(path: &std::path::Path) -> Option<String> {
        let ext = path.extension()?.to_str()?;
        let mime = match ext {
            "rs" => "text/x-rust",
            "py" => "text/x-python",
            "js" => "text/javascript",
            "ts" => "text/typescript",
            "tsx" | "jsx" => "text/javascript",
            "json" => "application/json",
            "yaml" | "yml" => "text/yaml",
            "toml" => "text/x-toml",
            "md" => "text/markdown",
            "html" => "text/html",
            "css" => "text/css",
            "sh" | "bash" => "text/x-shellscript",
            "sql" => "text/x-sql",
            "go" => "text/x-go",
            "java" => "text/x-java",
            "c" | "h" => "text/x-c",
            "cpp" | "hpp" | "cc" => "text/x-c++",
            "rb" => "text/x-ruby",
            "php" => "text/x-php",
            "swift" => "text/x-swift",
            "kt" => "text/x-kotlin",
            "scala" => "text/x-scala",
            "txt" => "text/plain",
            "xml" => "application/xml",
            _ => "text/plain",
        };
        Some(mime.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guess_mime_type() {
        assert_eq!(
            ResourceRegistry::guess_mime_type(std::path::Path::new("test.rs")),
            Some("text/x-rust".to_string())
        );
        assert_eq!(
            ResourceRegistry::guess_mime_type(std::path::Path::new("test.py")),
            Some("text/x-python".to_string())
        );
        assert_eq!(
            ResourceRegistry::guess_mime_type(std::path::Path::new("test.ts")),
            Some("text/typescript".to_string())
        );
        assert_eq!(
            ResourceRegistry::guess_mime_type(std::path::Path::new("test.json")),
            Some("application/json".to_string())
        );
        assert_eq!(
            ResourceRegistry::guess_mime_type(std::path::Path::new("test.unknown")),
            Some("text/plain".to_string())
        );
    }

    #[test]
    fn test_resource_serialization() {
        let resource = Resource {
            uri: "file:///test/file.rs".to_string(),
            name: "file.rs".to_string(),
            description: Some("A test file".to_string()),
            mime_type: Some("text/x-rust".to_string()),
        };

        let json = serde_json::to_string(&resource).unwrap();
        assert!(json.contains("\"uri\":\"file:///test/file.rs\""));
        assert!(json.contains("\"name\":\"file.rs\""));
        assert!(json.contains("\"mimeType\":\"text/x-rust\""));

        let parsed: Resource = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.uri, resource.uri);
        assert_eq!(parsed.name, resource.name);
    }

    #[test]
    fn test_resource_contents_serialization() {
        let contents = ResourceContents {
            uri: "file:///test/file.rs".to_string(),
            mime_type: Some("text/x-rust".to_string()),
            text: Some("fn main() {}".to_string()),
            blob: None,
        };

        let json = serde_json::to_string(&contents).unwrap();
        assert!(json.contains("\"text\":\"fn main() {}\""));
        assert!(!json.contains("\"blob\"")); // blob should be skipped when None
    }

    #[test]
    fn test_list_resources_result_serialization() {
        let result = ListResourcesResult {
            resources: vec![Resource {
                uri: "file:///test.rs".to_string(),
                name: "test.rs".to_string(),
                description: None,
                mime_type: None,
            }],
            next_cursor: Some("cursor123".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"nextCursor\":\"cursor123\""));

        let result_no_cursor = ListResourcesResult {
            resources: vec![],
            next_cursor: None,
        };
        let json2 = serde_json::to_string(&result_no_cursor).unwrap();
        assert!(!json2.contains("nextCursor"));
    }

    #[test]
    fn test_read_resource_result_serialization() {
        let result = ReadResourceResult {
            contents: vec![ResourceContents {
                uri: "file:///test.rs".to_string(),
                mime_type: Some("text/x-rust".to_string()),
                text: Some("code".to_string()),
                blob: None,
            }],
        };

        let json = serde_json::to_string(&result).unwrap();
        let parsed: ReadResourceResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.contents.len(), 1);
        assert_eq!(parsed.contents[0].text, Some("code".to_string()));
    }
}
