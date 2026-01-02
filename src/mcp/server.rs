//! MCP server implementation.

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::error::{Error, Result};
use crate::mcp::handler::McpHandler;
use crate::mcp::prompts::PromptRegistry;
use crate::mcp::protocol::*;
use crate::mcp::transport::{Message, Transport};
use crate::VERSION;

/// MCP server.
pub struct McpServer {
    handler: Arc<McpHandler>,
    prompts: Arc<PromptRegistry>,
    name: String,
    version: String,
}

impl McpServer {
    /// Create a new MCP server.
    pub fn new(handler: McpHandler, name: impl Into<String>) -> Self {
        Self {
            handler: Arc::new(handler),
            prompts: Arc::new(PromptRegistry::new()),
            name: name.into(),
            version: VERSION.to_string(),
        }
    }

    /// Create a new MCP server with custom prompt registry.
    pub fn with_prompts(
        handler: McpHandler,
        prompts: PromptRegistry,
        name: impl Into<String>,
    ) -> Self {
        Self {
            handler: Arc::new(handler),
            prompts: Arc::new(prompts),
            name: name.into(),
            version: VERSION.to_string(),
        }
    }

    /// Run the server with the given transport.
    pub async fn run<T: Transport>(&self, mut transport: T) -> Result<()> {
        info!("Starting MCP server: {} v{}", self.name, self.version);

        let (mut incoming, outgoing) = transport.start().await?;

        while let Some(msg) = incoming.recv().await {
            match msg {
                Message::Request(req) => {
                    let response = self.handle_request(req).await;
                    if outgoing.send(Message::Response(response)).await.is_err() {
                        error!("Failed to send response");
                        break;
                    }
                }
                Message::Notification(notif) => {
                    self.handle_notification(notif).await;
                }
                Message::Response(_) => {
                    warn!("Received unexpected response");
                }
            }
        }

        transport.stop().await?;
        info!("MCP server stopped");
        Ok(())
    }

    /// Handle a JSON-RPC request.
    async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        debug!("Handling request: {} (id: {:?})", req.method, req.id);

        let result = match req.method.as_str() {
            "initialize" => self.handle_initialize(req.params).await,
            "tools/list" => self.handle_list_tools().await,
            "tools/call" => self.handle_call_tool(req.params).await,
            "prompts/list" => self.handle_list_prompts().await,
            "prompts/get" => self.handle_get_prompt(req.params).await,
            "ping" => Ok(serde_json::json!({})),
            _ => Err(Error::McpProtocol(format!(
                "Unknown method: {}",
                req.method
            ))),
        };

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: JSONRPC_VERSION.to_string(),
                id: req.id,
                result: Some(value),
                error: None,
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: JSONRPC_VERSION.to_string(),
                id: req.id,
                result: None,
                error: Some(JsonRpcError {
                    code: error_codes::INTERNAL_ERROR,
                    message: e.to_string(),
                    data: None,
                }),
            },
        }
    }

    /// Handle a notification.
    async fn handle_notification(&self, notif: JsonRpcNotification) {
        debug!("Handling notification: {}", notif.method);

        match notif.method.as_str() {
            "notifications/initialized" => {
                info!("Client initialized");
            }
            "notifications/cancelled" => {
                debug!("Request cancelled");
            }
            _ => {
                debug!("Unknown notification: {}", notif.method);
            }
        }
    }

    /// Handle initialize request.
    async fn handle_initialize(&self, _params: Option<Value>) -> Result<Value> {
        let result = InitializeResult {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability { list_changed: true }),
                resources: None,
                prompts: Some(PromptsCapability {
                    list_changed: false,
                }),
                logging: Some(LoggingCapability {}),
            },
            server_info: ServerInfo {
                name: self.name.clone(),
                version: self.version.clone(),
            },
        };

        Ok(serde_json::to_value(result)?)
    }

    /// Handle list tools request.
    async fn handle_list_tools(&self) -> Result<Value> {
        let tools = self.handler.list_tools();
        let result = ListToolsResult { tools };
        Ok(serde_json::to_value(result)?)
    }

    /// Handle call tool request.
    async fn handle_call_tool(&self, params: Option<Value>) -> Result<Value> {
        let params: CallToolParams = params
            .ok_or_else(|| Error::InvalidToolArguments("Missing params".to_string()))
            .and_then(|v| {
                serde_json::from_value(v).map_err(|e| Error::InvalidToolArguments(e.to_string()))
            })?;

        let handler = self
            .handler
            .get_tool(&params.name)
            .ok_or_else(|| Error::ToolNotFound(params.name.clone()))?;

        let result = handler.execute(params.arguments).await?;
        Ok(serde_json::to_value(result)?)
    }

    /// Handle list prompts request.
    async fn handle_list_prompts(&self) -> Result<Value> {
        use crate::mcp::prompts::ListPromptsResult;

        let prompts = self.prompts.list();
        let result = ListPromptsResult {
            prompts,
            next_cursor: None,
        };
        Ok(serde_json::to_value(result)?)
    }

    /// Handle get prompt request.
    async fn handle_get_prompt(&self, params: Option<Value>) -> Result<Value> {
        #[derive(serde::Deserialize)]
        struct GetPromptParams {
            name: String,
            #[serde(default)]
            arguments: HashMap<String, String>,
        }

        let params: GetPromptParams = params
            .ok_or_else(|| Error::InvalidToolArguments("Missing params".to_string()))
            .and_then(|v| {
                serde_json::from_value(v).map_err(|e| Error::InvalidToolArguments(e.to_string()))
            })?;

        let result = self
            .prompts
            .get(&params.name, &params.arguments)
            .ok_or_else(|| Error::McpProtocol(format!("Prompt not found: {}", params.name)))?;

        Ok(serde_json::to_value(result)?)
    }
}
