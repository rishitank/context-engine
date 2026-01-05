//! MCP protocol types and message definitions.
//!
//! Based on the Model Context Protocol specification (2025-11-25).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// JSON-RPC version.
pub const JSONRPC_VERSION: &str = "2.0";

/// MCP protocol version - Updated to latest stable spec (2025-11-25).
pub const MCP_VERSION: &str = "2025-11-25";

// ===== JSON-RPC Base Types =====

/// A JSON-RPC request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: RequestId,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// A JSON-RPC response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: RequestId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// A JSON-RPC notification (no id, no response expected).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// A JSON-RPC error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Request ID (can be string or number).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
}

// ===== MCP-Specific Types =====

/// Server capabilities.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<LoggingCapability>,
}

/// Tools capability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolsCapability {
    #[serde(default)]
    pub list_changed: bool,
}

/// Resources capability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourcesCapability {
    #[serde(default)]
    pub subscribe: bool,
    #[serde(default)]
    pub list_changed: bool,
}

/// Prompts capability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptsCapability {
    #[serde(default)]
    pub list_changed: bool,
}

/// Logging capability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoggingCapability {}

/// Server info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// Initialize result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
}

/// Tool annotations - hints about tool behavior for AI clients.
///
/// These annotations help AI clients understand when and how to use tools automatically.
/// All properties are hints and not guaranteed to be accurate for untrusted servers.
///
/// Based on MCP spec 2025-11-25.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolAnnotations {
    /// Human-readable title for the tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// If true, the tool does not modify its environment.
    /// Default: false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only_hint: Option<bool>,

    /// If true, the tool may perform destructive updates to its environment.
    /// If false, the tool performs only additive updates.
    /// (Meaningful only when readOnlyHint == false)
    /// Default: true
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destructive_hint: Option<bool>,

    /// If true, calling the tool repeatedly with the same arguments
    /// will have no additional effect on its environment.
    /// (Meaningful only when readOnlyHint == false)
    /// Default: false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotent_hint: Option<bool>,

    /// If true, this tool may interact with an "open world" of external entities.
    /// If false, the tool's domain of interaction is closed.
    /// For example, the world of a web search tool is open, whereas that
    /// of a memory tool is not.
    /// Default: true
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_world_hint: Option<bool>,
}

impl ToolAnnotations {
    /// Create annotations for a read-only tool (does not modify environment).
    pub fn read_only() -> Self {
        Self {
            read_only_hint: Some(true),
            open_world_hint: Some(false),
            ..Default::default()
        }
    }

    /// Create annotations for a tool that modifies state but is not destructive.
    pub fn additive() -> Self {
        Self {
            read_only_hint: Some(false),
            destructive_hint: Some(false),
            open_world_hint: Some(false),
            ..Default::default()
        }
    }

    /// Create annotations for a tool that may perform destructive updates.
    pub fn destructive() -> Self {
        Self {
            read_only_hint: Some(false),
            destructive_hint: Some(true),
            open_world_hint: Some(false),
            ..Default::default()
        }
    }

    /// Create annotations for an idempotent tool (safe to call multiple times).
    pub fn idempotent() -> Self {
        Self {
            read_only_hint: Some(false),
            destructive_hint: Some(false),
            idempotent_hint: Some(true),
            open_world_hint: Some(false),
            ..Default::default()
        }
    }

    /// Set the human-readable title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Mark as interacting with external entities (open world).
    pub fn with_open_world(mut self) -> Self {
        self.open_world_hint = Some(true);
        self
    }
}

/// Tool definition.
///
/// Based on MCP spec 2025-11-25 with full support for annotations and output schema.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    /// Unique identifier for the tool.
    pub name: String,

    /// Human-readable description of the tool's functionality.
    pub description: String,

    /// JSON Schema defining expected parameters for the tool.
    #[serde(default)]
    pub input_schema: Value,

    /// Optional human-readable title for display purposes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Optional annotations describing tool behavior (hints for AI clients).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ToolAnnotations>,

    /// Optional JSON Schema defining the structure of the tool's output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,
}

impl Tool {
    /// Create a new tool with the given name, description, and input schema.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
            title: None,
            annotations: None,
            output_schema: None,
        }
    }

    /// Set the human-readable title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the tool annotations.
    pub fn with_annotations(mut self, annotations: ToolAnnotations) -> Self {
        self.annotations = Some(annotations);
        self
    }

    /// Set the output schema.
    pub fn with_output_schema(mut self, schema: Value) -> Self {
        self.output_schema = Some(schema);
        self
    }

    /// Mark this tool as read-only (does not modify environment).
    pub fn read_only(mut self) -> Self {
        self.annotations = Some(ToolAnnotations::read_only());
        self
    }

    /// Mark this tool as additive (modifies state but not destructive).
    pub fn additive(mut self) -> Self {
        self.annotations = Some(ToolAnnotations::additive());
        self
    }

    /// Mark this tool as destructive (may perform destructive updates).
    pub fn destructive(mut self) -> Self {
        self.annotations = Some(ToolAnnotations::destructive());
        self
    }

    /// Mark this tool as idempotent (safe to call multiple times).
    pub fn idempotent(mut self) -> Self {
        self.annotations = Some(ToolAnnotations::idempotent());
        self
    }
}

/// Tool call result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: Vec<ContentBlock>,
    #[serde(default)]
    pub is_error: bool,
}

/// Content block in a tool result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ContentBlock {
    Text {
        text: String,
    },
    Image {
        data: String,
        mime_type: String,
    },
    Resource {
        uri: String,
        mime_type: Option<String>,
        text: Option<String>,
    },
}

/// List tools result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResult {
    pub tools: Vec<Tool>,
}

/// Call tool params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolParams {
    pub name: String,
    #[serde(default)]
    pub arguments: HashMap<String, Value>,
}

// ===== Error Codes =====

/// Standard JSON-RPC error codes.
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({"name": "test"})),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"tools/call\""));

        let parsed: JsonRpcRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.method, "tools/call");
    }

    #[test]
    fn test_jsonrpc_response_success() {
        let response = JsonRpcResponse {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: RequestId::Number(1),
            result: Some(json!({"success": true})),
            error: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_jsonrpc_response_error() {
        let response = JsonRpcResponse {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: RequestId::String("abc".to_string()),
            result: None,
            error: Some(JsonRpcError {
                code: error_codes::METHOD_NOT_FOUND,
                message: "Method not found".to_string(),
                data: None,
            }),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"error\""));
        assert!(json.contains("-32601"));
    }

    #[test]
    fn test_request_id_variants() {
        let id_num = RequestId::Number(42);
        let id_str = RequestId::String("request-1".to_string());

        assert_eq!(serde_json::to_string(&id_num).unwrap(), "42");
        assert_eq!(serde_json::to_string(&id_str).unwrap(), "\"request-1\"");
    }

    #[test]
    fn test_tool_definition() {
        let tool = Tool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                }
            }),
            ..Default::default()
        };

        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("\"name\":\"test_tool\""));
        assert!(json.contains("\"inputSchema\""));
    }

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult {
            content: vec![ContentBlock::Text {
                text: "Success".to_string(),
            }],
            is_error: false,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"type\":\"text\""));
        assert!(json.contains("\"is_error\":false"));
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult {
            content: vec![ContentBlock::Text {
                text: "Error occurred".to_string(),
            }],
            is_error: true,
        };

        assert!(result.is_error);
    }

    #[test]
    fn test_content_block_variants() {
        let text = ContentBlock::Text {
            text: "Hello".to_string(),
        };
        let image = ContentBlock::Image {
            data: "base64data".to_string(),
            mime_type: "image/png".to_string(),
        };
        let resource = ContentBlock::Resource {
            uri: "file://test.txt".to_string(),
            mime_type: Some("text/plain".to_string()),
            text: Some("content".to_string()),
        };

        let text_json = serde_json::to_string(&text).unwrap();
        assert!(text_json.contains("\"type\":\"text\""));

        let image_json = serde_json::to_string(&image).unwrap();
        assert!(image_json.contains("\"type\":\"image\""));

        let resource_json = serde_json::to_string(&resource).unwrap();
        assert!(resource_json.contains("\"type\":\"resource\""));
    }

    #[test]
    fn test_server_capabilities() {
        let caps = ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: true }),
            resources: None,
            prompts: None,
            logging: None,
        };

        let json = serde_json::to_string(&caps).unwrap();
        assert!(json.contains("\"tools\""));
        assert!(!json.contains("\"resources\""));
    }

    #[test]
    fn test_initialize_result() {
        let result = InitializeResult {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: ServerCapabilities::default(),
            server_info: ServerInfo {
                name: "context-engine".to_string(),
                version: "1.9.0".to_string(),
            },
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"protocolVersion\""));
        assert!(json.contains("\"serverInfo\""));
    }

    #[test]
    fn test_call_tool_params() {
        let params = CallToolParams {
            name: "semantic_search".to_string(),
            arguments: {
                let mut args = std::collections::HashMap::new();
                args.insert("query".to_string(), json!("test query"));
                args.insert("top_k".to_string(), json!(10));
                args
            },
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"name\":\"semantic_search\""));
        assert!(json.contains("\"arguments\""));
    }

    #[test]
    fn test_jsonrpc_notification() {
        let notification = JsonRpcNotification {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: "notifications/initialized".to_string(),
            params: None,
        };

        let json = serde_json::to_string(&notification).unwrap();
        assert!(!json.contains("\"id\""));
        assert!(json.contains("\"method\""));
    }

    #[test]
    fn test_tool_annotations_read_only() {
        let annotations = ToolAnnotations::read_only();
        assert_eq!(annotations.read_only_hint, Some(true));
        assert_eq!(annotations.open_world_hint, Some(false));
        assert_eq!(annotations.destructive_hint, None);
        assert_eq!(annotations.idempotent_hint, None);
    }

    #[test]
    fn test_tool_annotations_destructive() {
        let annotations = ToolAnnotations::destructive();
        assert_eq!(annotations.read_only_hint, Some(false));
        assert_eq!(annotations.destructive_hint, Some(true));
        assert_eq!(annotations.open_world_hint, Some(false));
    }

    #[test]
    fn test_tool_annotations_idempotent() {
        let annotations = ToolAnnotations::idempotent();
        assert_eq!(annotations.read_only_hint, Some(false));
        assert_eq!(annotations.destructive_hint, Some(false));
        assert_eq!(annotations.idempotent_hint, Some(true));
        assert_eq!(annotations.open_world_hint, Some(false));
    }

    #[test]
    fn test_tool_annotations_with_title() {
        let annotations = ToolAnnotations::read_only().with_title("Search Code");
        assert_eq!(annotations.title, Some("Search Code".to_string()));
        assert_eq!(annotations.read_only_hint, Some(true));
    }

    #[test]
    fn test_tool_annotations_serialization() {
        let annotations = ToolAnnotations {
            title: Some("Test Tool".to_string()),
            read_only_hint: Some(true),
            destructive_hint: None,
            idempotent_hint: None,
            open_world_hint: Some(false),
        };

        let json = serde_json::to_string(&annotations).unwrap();
        assert!(json.contains("\"title\":\"Test Tool\""));
        assert!(json.contains("\"readOnlyHint\":true"));
        assert!(json.contains("\"openWorldHint\":false"));
        // None fields should be skipped
        assert!(!json.contains("destructiveHint"));
        assert!(!json.contains("idempotentHint"));
    }

    #[test]
    fn test_tool_with_annotations() {
        let tool = Tool::new(
            "search_code",
            "Search the codebase",
            json!({"type": "object"}),
        )
        .with_title("Code Search")
        .with_annotations(ToolAnnotations::read_only());

        assert_eq!(tool.name, "search_code");
        assert_eq!(tool.title, Some("Code Search".to_string()));
        assert!(tool.annotations.is_some());
        let annotations = tool.annotations.unwrap();
        assert_eq!(annotations.read_only_hint, Some(true));
    }

    #[test]
    fn test_tool_read_only_shorthand() {
        let tool =
            Tool::new("get_file", "Get file contents", json!({"type": "object"})).read_only();

        assert!(tool.annotations.is_some());
        let annotations = tool.annotations.unwrap();
        assert_eq!(annotations.read_only_hint, Some(true));
    }

    #[test]
    fn test_tool_with_output_schema() {
        let tool = Tool::new("analyze", "Analyze code", json!({"type": "object"}))
            .with_output_schema(json!({
                "type": "object",
                "properties": {
                    "result": { "type": "string" }
                }
            }));

        assert!(tool.output_schema.is_some());
        let schema = tool.output_schema.unwrap();
        assert!(schema.get("properties").is_some());
    }

    #[test]
    fn test_mcp_version_is_latest() {
        // Verify we're using the latest MCP spec version
        assert_eq!(MCP_VERSION, "2025-11-25");
    }
}
