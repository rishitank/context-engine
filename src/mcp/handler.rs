//! MCP request and notification handlers.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::Result;
use crate::mcp::protocol::{Tool, ToolResult};

/// Handler for MCP tool calls.
#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// Get the tool definition.
    fn definition(&self) -> Tool;

    /// Execute the tool with the given arguments.
    async fn execute(&self, arguments: HashMap<String, Value>) -> Result<ToolResult>;
}

/// Registry of tool handlers.
pub struct McpHandler {
    tools: HashMap<String, Arc<dyn ToolHandler>>,
}

impl McpHandler {
    /// Create a new handler registry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool handler.
    pub fn register<T: ToolHandler + 'static>(&mut self, handler: T) {
        let tool = handler.definition();
        self.tools.insert(tool.name.clone(), Arc::new(handler));
    }

    /// Register a tool handler (Arc version).
    pub fn register_arc(&mut self, handler: Arc<dyn ToolHandler>) {
        let tool = handler.definition();
        self.tools.insert(tool.name.clone(), handler);
    }

    /// Get all registered tools.
    pub fn list_tools(&self) -> Vec<Tool> {
        self.tools.values().map(|h| h.definition()).collect()
    }

    /// Get a tool by name.
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn ToolHandler>> {
        self.tools.get(name).cloned()
    }

    /// Check if a tool exists.
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get the number of registered tools.
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }
}

impl Default for McpHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper macro for creating tool input schemas.
#[macro_export]
macro_rules! tool_schema {
    ($($json:tt)+) => {
        serde_json::json!({
            "type": "object",
            "properties": {
                $($json)+
            }
        })
    };
}

/// Helper to create a text content block.
pub fn text_content(text: impl Into<String>) -> crate::mcp::protocol::ContentBlock {
    crate::mcp::protocol::ContentBlock::Text { text: text.into() }
}

/// Helper to create a successful tool result.
pub fn success_result(text: impl Into<String>) -> ToolResult {
    ToolResult {
        content: vec![text_content(text)],
        is_error: false,
    }
}

/// Helper to create an error tool result.
pub fn error_result(text: impl Into<String>) -> ToolResult {
    ToolResult {
        content: vec![text_content(text)],
        is_error: true,
    }
}

/// Helper to extract a required string argument.
pub fn get_string_arg(args: &HashMap<String, Value>, name: &str) -> Result<String> {
    args.get(name)
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| {
            crate::error::Error::InvalidToolArguments(format!(
                "Missing required argument: {}",
                name
            ))
        })
}

/// Helper to extract an optional string argument.
pub fn get_optional_string_arg(args: &HashMap<String, Value>, name: &str) -> Option<String> {
    args.get(name).and_then(|v| v.as_str()).map(String::from)
}

/// Helper to extract a required integer argument.
pub fn get_int_arg(args: &HashMap<String, Value>, name: &str) -> Result<i64> {
    args.get(name).and_then(|v| v.as_i64()).ok_or_else(|| {
        crate::error::Error::InvalidToolArguments(format!("Missing required argument: {}", name))
    })
}

/// Helper to extract a required boolean argument.
pub fn get_bool_arg(args: &HashMap<String, Value>, name: &str, default: bool) -> bool {
    args.get(name).and_then(|v| v.as_bool()).unwrap_or(default)
}

/// Helper to extract a string array argument.
pub fn get_string_array_arg(args: &HashMap<String, Value>, name: &str) -> Vec<String> {
    args.get(name)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct TestTool {
        name: String,
    }

    #[async_trait]
    impl ToolHandler for TestTool {
        fn definition(&self) -> Tool {
            Tool {
                name: self.name.clone(),
                description: format!("Test tool: {}", self.name),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "input": { "type": "string" }
                    }
                }),
            }
        }

        async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
            let input = get_optional_string_arg(&args, "input").unwrap_or_default();
            Ok(success_result(format!(
                "Executed {} with: {}",
                self.name, input
            )))
        }
    }

    #[test]
    fn test_handler_registration() {
        let mut handler = McpHandler::new();
        handler.register(TestTool {
            name: "test_tool".to_string(),
        });

        assert_eq!(handler.tool_count(), 1);
        assert!(handler.has_tool("test_tool"));
        assert!(!handler.has_tool("nonexistent"));
    }

    #[test]
    fn test_handler_list_tools() {
        let mut handler = McpHandler::new();
        handler.register(TestTool {
            name: "tool_a".to_string(),
        });
        handler.register(TestTool {
            name: "tool_b".to_string(),
        });

        let tools = handler.list_tools();
        assert_eq!(tools.len(), 2);

        let names: Vec<_> = tools.iter().map(|t| &t.name).collect();
        assert!(names.contains(&&"tool_a".to_string()));
        assert!(names.contains(&&"tool_b".to_string()));
    }

    #[tokio::test]
    async fn test_tool_execution() {
        let mut handler = McpHandler::new();
        handler.register(TestTool {
            name: "echo".to_string(),
        });

        let tool = handler.get_tool("echo").unwrap();
        let mut args = HashMap::new();
        args.insert("input".to_string(), json!("hello"));

        let result = tool.execute(args).await.unwrap();
        assert!(!result.is_error);

        if let crate::mcp::protocol::ContentBlock::Text { text } = &result.content[0] {
            assert!(text.contains("Executed echo with: hello"));
        } else {
            panic!("Expected text content");
        }
    }

    #[test]
    fn test_get_string_arg() {
        let mut args = HashMap::new();
        args.insert("name".to_string(), json!("value"));

        assert_eq!(get_string_arg(&args, "name").unwrap(), "value");
        assert!(get_string_arg(&args, "missing").is_err());
    }

    #[test]
    fn test_get_optional_string_arg() {
        let mut args = HashMap::new();
        args.insert("name".to_string(), json!("value"));

        assert_eq!(
            get_optional_string_arg(&args, "name"),
            Some("value".to_string())
        );
        assert_eq!(get_optional_string_arg(&args, "missing"), None);
    }

    #[test]
    fn test_get_int_arg() {
        let mut args = HashMap::new();
        args.insert("count".to_string(), json!(42));

        assert_eq!(get_int_arg(&args, "count").unwrap(), 42);
        assert!(get_int_arg(&args, "missing").is_err());
    }

    #[test]
    fn test_get_bool_arg() {
        let mut args = HashMap::new();
        args.insert("flag".to_string(), json!(true));

        assert!(get_bool_arg(&args, "flag", false));
        assert!(!get_bool_arg(&args, "missing", false));
        assert!(get_bool_arg(&args, "missing", true));
    }

    #[test]
    fn test_get_string_array_arg() {
        let mut args = HashMap::new();
        args.insert("items".to_string(), json!(["a", "b", "c"]));

        let items = get_string_array_arg(&args, "items");
        assert_eq!(items, vec!["a", "b", "c"]);

        assert!(get_string_array_arg(&args, "missing").is_empty());
    }

    #[test]
    fn test_success_result() {
        let result = success_result("Success!");
        assert!(!result.is_error);
        assert_eq!(result.content.len(), 1);
    }

    #[test]
    fn test_error_result() {
        let result = error_result("Error!");
        assert!(result.is_error);
        assert_eq!(result.content.len(), 1);
    }
}
