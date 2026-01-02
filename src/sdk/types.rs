//! SDK-specific types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Options for creating a DirectContext.
#[derive(Debug, Clone, Default)]
pub struct DirectContextOptions {
    /// API key (overrides env/session)
    pub api_key: Option<String>,
    /// API URL (overrides env/session)
    pub api_url: Option<String>,
    /// Enable debug logging
    pub debug: bool,
    /// Maximum file size in bytes (default: 1MB)
    pub max_file_size: Option<usize>,
}

/// Blob information for tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobInfo {
    /// Blob name (SHA256 hash)
    pub blob_name: String,
    /// File path
    pub path: String,
}

/// Blob entry for upload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobEntry {
    /// Blob name (SHA256 hash)
    pub blob_name: String,
    /// File path
    pub path: String,
    /// File content
    pub content: String,
}

/// Blobs state for API requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blobs {
    /// Checkpoint ID (null for first request)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint_id: Option<String>,
    /// Added blobs since last checkpoint
    pub added_blobs: Vec<BlobInfo>,
    /// Deleted blobs since last checkpoint
    pub deleted_blobs: Vec<String>,
}

/// Result of an indexing operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingResult {
    /// Number of files indexed
    pub indexed: usize,
    /// Number of files skipped
    pub skipped: usize,
    /// Error messages
    pub errors: Vec<String>,
}

/// State of a DirectContext (for persistence).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DirectContextState {
    /// Blob map: path -> blob_name
    pub blob_map: HashMap<String, String>,
    /// Client blob map: path -> blob_name (local tracking)
    pub client_blob_map: HashMap<String, String>,
    /// Current checkpoint ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint_id: Option<String>,
    /// Pending added blobs
    pub pending_added: Vec<BlobInfo>,
    /// Pending deleted blob names
    pub pending_deleted: Vec<String>,
}

// ===== API Request/Response Types =====

/// Request to find missing blobs.
#[derive(Debug, Clone, Serialize)]
pub struct FindMissingRequest {
    pub mem_object_names: Vec<String>,
}

/// Response from find-missing endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct FindMissingResponse {
    pub unknown_memory_names: Vec<String>,
    pub nonindexed_blob_names: Vec<String>,
}

/// Request to batch upload blobs.
#[derive(Debug, Clone, Serialize)]
pub struct BatchUploadRequest {
    pub blobs: Vec<BlobEntry>,
}

/// Response from batch-upload endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct BatchUploadResponse {
    pub blob_names: Vec<String>,
}

/// Request to checkpoint blobs.
#[derive(Debug, Clone, Serialize)]
pub struct CheckpointBlobsRequest {
    pub blobs: CheckpointBlobs,
}

/// Checkpoint blobs structure.
#[derive(Debug, Clone, Serialize)]
pub struct CheckpointBlobs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint_id: Option<String>,
    pub added_blobs: Vec<BlobInfo>,
    pub deleted_blobs: Vec<String>,
}

/// Response from checkpoint-blobs endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct CheckpointBlobsResponse {
    pub new_checkpoint_id: String,
}

/// Request for codebase retrieval.
#[derive(Debug, Clone, Serialize)]
pub struct CodebaseRetrievalRequest {
    pub information_request: String,
    pub blobs: Blobs,
    pub dialog: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_length: Option<usize>,
}

/// Response from codebase retrieval.
#[derive(Debug, Clone, Deserialize)]
pub struct CodebaseRetrievalResponse {
    pub formatted_retrieval: String,
}

/// Chat message for streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Chat node from SSE stream.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_node: Option<TextNode>,
}

/// Text node content.
#[derive(Debug, Clone, Deserialize)]
pub struct TextNode {
    pub content: String,
}

/// Chat stream response.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatStreamResponse {
    pub nodes: Vec<ChatNode>,
    pub chat_history: Vec<serde_json::Value>,
    pub conversation_id: String,
}

/// Request for chat stream.
#[derive(Debug, Clone, Serialize)]
pub struct ChatStreamRequest {
    pub prompt: String,
    pub blobs: Blobs,
    pub dialog: Vec<serde_json::Value>,
}

/// SSE event from chat stream.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatStreamEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done: Option<bool>,
}
