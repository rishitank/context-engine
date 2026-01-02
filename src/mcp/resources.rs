//! MCP Resources Support
//!
//! Expose indexed files as MCP resources that AI clients can browse and read.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

use crate::error::{Error, Result};
use crate::service::ContextService;

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
    /// Create a new resource registry.
    pub fn new(context_service: Arc<ContextService>) -> Self {
        Self {
            context_service,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// List available resources (files in workspace).
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

                let uri = format!("file://{}", path.display());
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

    /// Read a resource by URI.
    pub async fn read(&self, uri: &str) -> Result<ReadResourceResult> {
        // Parse file:// URI
        let path = if let Some(path) = uri.strip_prefix("file://") {
            PathBuf::from(path)
        } else {
            return Err(Error::InvalidToolArguments(format!(
                "Invalid URI scheme: {}",
                uri
            )));
        };

        // Security: ensure path is within workspace
        let workspace = self.context_service.workspace();
        let canonical = path
            .canonicalize()
            .map_err(|e| Error::InvalidToolArguments(format!("Cannot resolve path: {}", e)))?;

        if !canonical.starts_with(workspace) {
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

    /// Subscribe to resource changes.
    pub async fn subscribe(&self, uri: &str, session_id: &str) -> Result<()> {
        let mut subs = self.subscriptions.write().await;
        subs.entry(uri.to_string())
            .or_default()
            .push(session_id.to_string());
        Ok(())
    }

    /// Unsubscribe from resource changes.
    pub async fn unsubscribe(&self, uri: &str, session_id: &str) -> Result<()> {
        let mut subs = self.subscriptions.write().await;
        if let Some(sessions) = subs.get_mut(uri) {
            sessions.retain(|s| s != session_id);
        }
        Ok(())
    }

    /// Discover files in directory (with pagination).
    async fn discover_files(
        &self,
        dir: &std::path::Path,
        limit: usize,
        after: Option<&str>,
    ) -> Result<Vec<PathBuf>> {
        use tokio::fs::read_dir;

        let mut files = Vec::new();
        let mut stack = vec![dir.to_path_buf()];
        let mut past_cursor = after.is_none();

        while let Some(current) = stack.pop() {
            if files.len() >= limit {
                break;
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

                if path.is_dir() {
                    stack.push(path);
                } else if path.is_file() {
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

        Ok(files)
    }

    /// Check if a file should be ignored.
    fn should_ignore(name: &str) -> bool {
        matches!(
            name,
            "node_modules" | "target" | "dist" | "build" | "__pycache__" | ".git"
        ) || name.ends_with(".lock")
            || name.ends_with(".pyc")
    }

    /// Guess MIME type from file extension.
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
    }
}
