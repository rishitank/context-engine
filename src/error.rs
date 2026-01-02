//! Error types for the Context Engine MCP Server.

use thiserror::Error;

/// Result type alias for Context Engine operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the Context Engine.
#[derive(Error, Debug)]
pub enum Error {
    // ===== SDK Errors =====
    #[error("API error: {status} {status_text} - {message}")]
    Api {
        status: u16,
        status_text: String,
        message: String,
    },

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Credentials not found: {0}")]
    CredentialsNotFound(String),

    #[error("Blob too large: file exceeds maximum size of {max_size} bytes")]
    BlobTooLarge { max_size: usize },

    #[error("Index not initialized: add files to index first using add_to_index()")]
    IndexNotInitialized,

    #[error("Indexing timeout: backend did not finish indexing within {seconds} seconds")]
    IndexingTimeout { seconds: u64 },

    // ===== MCP Errors =====
    #[error("MCP protocol error: {0}")]
    McpProtocol(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Invalid tool arguments: {0}")]
    InvalidToolArguments(String),

    #[error("Tool execution failed: {0}")]
    ToolExecutionFailed(String),

    // ===== Service Errors =====
    #[error("Workspace not found: {0}")]
    WorkspaceNotFound(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("File too large: {path} ({size} bytes)")]
    FileTooLarge { path: String, size: usize },

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    // ===== Planning Errors =====
    #[error("Plan not found: {0}")]
    PlanNotFound(String),

    #[error("Plan already exists: {0}")]
    PlanAlreadyExists(String),

    #[error("Step not found: step {0}")]
    StepNotFound(u32),

    #[error("Step blocked by dependencies: {0:?}")]
    StepBlocked(Vec<u32>),

    #[error("Approval required: {0}")]
    ApprovalRequired(String),

    // ===== Review Errors =====
    #[error("Invalid diff format: {0}")]
    InvalidDiffFormat(String),

    #[error("Review session not found: {0}")]
    ReviewSessionNotFound(String),

    #[error("Invariant check failed: {0}")]
    InvariantCheckFailed(String),

    // ===== I/O Errors =====
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    // ===== HTTP Errors =====
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("HTTP server error: {0}")]
    HttpServer(String),

    // ===== Internal Errors =====
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Timeout: operation timed out after {seconds} seconds")]
    Timeout { seconds: u64 },

    #[error("Cancelled: operation was cancelled")]
    Cancelled,
}

impl Error {
    /// Create an API error from HTTP response details.
    pub fn api(status: u16, status_text: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Api {
            status,
            status_text: status_text.into(),
            message: message.into(),
        }
    }

    /// Check if this error is retriable (transient failures).
    pub fn is_retriable(&self) -> bool {
        match self {
            Self::Api { status, .. } => {
                *status == 499 || *status == 503 || (*status >= 500 && *status < 600)
            }
            Self::Http(e) => e.is_timeout() || e.is_connect(),
            Self::Timeout { .. } => true,
            _ => false,
        }
    }

    /// Check if this error is a chat-specific retriable error.
    pub fn is_chat_retriable(&self) -> bool {
        match self {
            Self::Api { status, .. } => {
                *status == 429
                    || *status == 499
                    || *status == 503
                    || *status == 529
                    || (*status >= 500 && *status < 600)
            }
            _ => self.is_retriable(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let api_err = Error::api(404, "Not Found", "Resource not found");
        assert_eq!(
            api_err.to_string(),
            "API error: 404 Not Found - Resource not found"
        );

        let auth_err = Error::Auth("Invalid token".to_string());
        assert_eq!(auth_err.to_string(), "Authentication error: Invalid token");

        let creds_err = Error::CredentialsNotFound("~/.augment/session.json".to_string());
        assert_eq!(
            creds_err.to_string(),
            "Credentials not found: ~/.augment/session.json"
        );
    }

    #[test]
    fn test_error_is_retriable() {
        // API errors
        assert!(Error::api(500, "Internal Server Error", "").is_retriable());
        assert!(Error::api(503, "Service Unavailable", "").is_retriable());
        assert!(Error::api(499, "Client Closed Request", "").is_retriable());
        assert!(!Error::api(400, "Bad Request", "").is_retriable());
        assert!(!Error::api(404, "Not Found", "").is_retriable());

        // Timeout errors
        assert!(Error::Timeout { seconds: 30 }.is_retriable());

        // Non-retriable errors
        assert!(!Error::Auth("invalid".to_string()).is_retriable());
        assert!(!Error::ToolNotFound("test".to_string()).is_retriable());
    }

    #[test]
    fn test_error_is_chat_retriable() {
        // Rate limiting
        assert!(Error::api(429, "Too Many Requests", "").is_chat_retriable());
        // Overloaded
        assert!(Error::api(529, "Site Overloaded", "").is_chat_retriable());
        // Standard retriable
        assert!(Error::api(503, "Service Unavailable", "").is_chat_retriable());
        // Non-retriable
        assert!(!Error::api(400, "Bad Request", "").is_chat_retriable());
    }

    #[test]
    fn test_api_error_constructor() {
        let err = Error::api(500, "Internal Server Error", "Something went wrong");
        match err {
            Error::Api {
                status,
                status_text,
                message,
            } => {
                assert_eq!(status, 500);
                assert_eq!(status_text, "Internal Server Error");
                assert_eq!(message, "Something went wrong");
            }
            _ => panic!("Expected Api error"),
        }
    }

    #[test]
    fn test_plan_errors() {
        let not_found = Error::PlanNotFound("plan-123".to_string());
        assert_eq!(not_found.to_string(), "Plan not found: plan-123");

        let step_not_found = Error::StepNotFound(5);
        assert_eq!(step_not_found.to_string(), "Step not found: step 5");

        let blocked = Error::StepBlocked(vec![1, 2, 3]);
        assert!(blocked.to_string().contains("[1, 2, 3]"));
    }

    #[test]
    fn test_review_errors() {
        let invalid_diff = Error::InvalidDiffFormat("missing header".to_string());
        assert_eq!(
            invalid_diff.to_string(),
            "Invalid diff format: missing header"
        );

        let session_not_found = Error::ReviewSessionNotFound("session-456".to_string());
        assert_eq!(
            session_not_found.to_string(),
            "Review session not found: session-456"
        );
    }

    #[test]
    fn test_file_errors() {
        let file_not_found = Error::FileNotFound("/path/to/file.rs".to_string());
        assert_eq!(
            file_not_found.to_string(),
            "File not found: /path/to/file.rs"
        );

        let file_too_large = Error::FileTooLarge {
            path: "large.bin".to_string(),
            size: 10_000_000,
        };
        assert!(file_too_large.to_string().contains("10000000 bytes"));
    }

    #[test]
    fn test_mcp_errors() {
        let tool_not_found = Error::ToolNotFound("unknown_tool".to_string());
        assert_eq!(tool_not_found.to_string(), "Tool not found: unknown_tool");

        let invalid_args =
            Error::InvalidToolArguments("missing required field 'query'".to_string());
        assert!(invalid_args.to_string().contains("missing required field"));
    }

    #[test]
    fn test_blob_error() {
        let blob_too_large = Error::BlobTooLarge {
            max_size: 5_000_000,
        };
        assert!(blob_too_large.to_string().contains("5000000 bytes"));
    }

    #[test]
    fn test_timeout_and_cancelled() {
        let timeout = Error::Timeout { seconds: 60 };
        assert_eq!(
            timeout.to_string(),
            "Timeout: operation timed out after 60 seconds"
        );

        let cancelled = Error::Cancelled;
        assert_eq!(cancelled.to_string(), "Cancelled: operation was cancelled");
    }
}
