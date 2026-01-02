//! Search and context retrieval types.

use serde::{Deserialize, Serialize};

/// A file with path and contents (source-agnostic).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    /// Relative path (e.g., "src/main.rs")
    pub path: String,
    /// File contents as string
    pub contents: String,
}

/// A single code chunk from retrieval results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// File path relative to workspace root
    pub path: String,
    /// Starting line number (1-based)
    pub start_line: u32,
    /// Ending line number (1-based, inclusive)
    pub end_line: u32,
    /// The code content
    pub contents: String,
}

/// Result from a codebase search query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// File path
    pub path: String,
    /// Content snippet
    pub content: String,
    /// Relevance score (0-1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    /// Line range (e.g., "10-25")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<String>,
    /// Normalized relevance score (0-1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relevance_score: Option<f64>,
    /// Type of match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_type: Option<MatchType>,
    /// When the result was retrieved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieved_at: Option<String>,
    /// Chunk identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_id: Option<String>,
}

/// Type of search match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchType {
    Semantic,
    Keyword,
    Hybrid,
}

/// Index status information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStatus {
    /// Workspace path
    pub workspace: String,
    /// Current status
    pub status: IndexState,
    /// Last indexed timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_indexed: Option<String>,
    /// Number of indexed files
    pub file_count: usize,
    /// Whether index is stale
    pub is_stale: bool,
    /// Last error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

/// Indexing state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IndexState {
    #[default]
    Idle,
    Indexing,
    Error,
}

/// Result of an indexing operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexResult {
    /// Number of files indexed
    pub indexed: usize,
    /// Number of files skipped
    pub skipped: usize,
    /// Error messages
    pub errors: Vec<String>,
    /// Duration in milliseconds
    pub duration: u64,
}

/// File watcher status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherStatus {
    /// Whether watcher is enabled
    pub enabled: bool,
    /// Number of directories being watched
    pub watching: usize,
    /// Number of pending changes
    pub pending_changes: usize,
    /// Last flush timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_flush: Option<String>,
}

/// Information about a code snippet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetInfo {
    /// Snippet text
    pub text: String,
    /// Line range
    pub lines: String,
    /// Relevance score (0-1)
    pub relevance: f64,
    /// Estimated token count
    pub token_count: usize,
    /// Type of code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_type: Option<String>,
}

/// Context for a single file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContext {
    /// File path
    pub path: String,
    /// File extension
    pub extension: String,
    /// Summary of file contents
    pub summary: String,
    /// Relevance score (0-1)
    pub relevance: f64,
    /// Estimated token count
    pub token_count: usize,
    /// Code snippets
    pub snippets: Vec<SnippetInfo>,
    /// Related files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_files: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_serialization() {
        let file = File {
            path: "src/main.rs".to_string(),
            contents: "fn main() {}".to_string(),
        };

        let json = serde_json::to_string(&file).unwrap();
        let parsed: File = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.path, file.path);
        assert_eq!(parsed.contents, file.contents);
    }

    #[test]
    fn test_chunk_serialization() {
        let chunk = Chunk {
            path: "src/lib.rs".to_string(),
            start_line: 10,
            end_line: 20,
            contents: "pub fn hello() {}".to_string(),
        };

        let json = serde_json::to_string(&chunk).unwrap();
        let parsed: Chunk = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.path, chunk.path);
        assert_eq!(parsed.start_line, 10);
        assert_eq!(parsed.end_line, 20);
    }

    #[test]
    fn test_search_result_optional_fields() {
        let result = SearchResult {
            path: "test.rs".to_string(),
            content: "test content".to_string(),
            score: Some(0.95),
            lines: Some("1-10".to_string()),
            relevance_score: None,
            match_type: Some(MatchType::Semantic),
            retrieved_at: None,
            chunk_id: None,
        };

        let json = serde_json::to_string(&result).unwrap();

        // Optional None fields should not appear in JSON
        assert!(!json.contains("relevance_score"));
        assert!(!json.contains("retrieved_at"));

        // Optional Some fields should appear
        assert!(json.contains("score"));
        assert!(json.contains("lines"));
    }

    #[test]
    fn test_index_state_default() {
        let state = IndexState::default();
        assert!(matches!(state, IndexState::Idle));
    }

    #[test]
    fn test_index_status_serialization() {
        let status = IndexStatus {
            workspace: "/path/to/workspace".to_string(),
            status: IndexState::Indexing,
            file_count: 100,
            last_indexed: Some("2024-01-01T00:00:00Z".to_string()),
            is_stale: false,
            last_error: None,
        };

        let json = serde_json::to_string(&status).unwrap();
        let parsed: IndexStatus = serde_json::from_str(&json).unwrap();

        assert!(matches!(parsed.status, IndexState::Indexing));
        assert_eq!(parsed.file_count, 100);
    }

    #[test]
    fn test_index_result() {
        let result = IndexResult {
            indexed: 50,
            skipped: 10,
            errors: vec!["error1".to_string()],
            duration: 1000,
        };

        assert_eq!(result.indexed, 50);
        assert_eq!(result.skipped, 10);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.duration, 1000);
    }
}

