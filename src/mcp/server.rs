//! MCP server implementation.

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::error::{Error, Result};
use crate::mcp::handler::McpHandler;
use crate::mcp::prompts::PromptRegistry;
use crate::mcp::protocol::*;
use crate::mcp::resources::ResourceRegistry;
use crate::mcp::transport::{Message, Transport};
use crate::service::ContextService;
use crate::VERSION;

/// MCP server.
pub struct McpServer {
    handler: Arc<McpHandler>,
    prompts: Arc<PromptRegistry>,
    resources: Option<Arc<ResourceRegistry>>,
    name: String,
    version: String,
    /// Workspace roots provided by the client.
    roots: Arc<RwLock<Vec<PathBuf>>>,
    /// Active request IDs for cancellation support.
    active_requests: Arc<RwLock<HashSet<RequestId>>>,
}

impl McpServer {
    /// Create a new MCP server.
    pub fn new(handler: McpHandler, name: impl Into<String>) -> Self {
        Self {
            handler: Arc::new(handler),
            prompts: Arc::new(PromptRegistry::new()),
            resources: None,
            name: name.into(),
            version: VERSION.to_string(),
            roots: Arc::new(RwLock::new(Vec::new())),
            active_requests: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Create a new MCP server with all features.
    pub fn with_features(
        handler: McpHandler,
        prompts: PromptRegistry,
        context_service: Arc<ContextService>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            handler: Arc::new(handler),
            prompts: Arc::new(prompts),
            resources: Some(Arc::new(ResourceRegistry::new(context_service))),
            name: name.into(),
            version: VERSION.to_string(),
            roots: Arc::new(RwLock::new(Vec::new())),
            active_requests: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Get the client-provided workspace roots.
    pub async fn roots(&self) -> Vec<PathBuf> {
        self.roots.read().await.clone()
    }

    /// Check if a request has been cancelled.
    pub async fn is_cancelled(&self, id: &RequestId) -> bool {
        !self.active_requests.read().await.contains(id)
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

        // Track active request for cancellation
        self.active_requests.write().await.insert(req.id.clone());

        let result = match req.method.as_str() {
            // Core
            "initialize" => self.handle_initialize(req.params).await,
            "ping" => Ok(serde_json::json!({})),
            // Tools
            "tools/list" => self.handle_list_tools().await,
            "tools/call" => self.handle_call_tool(req.params).await,
            // Prompts
            "prompts/list" => self.handle_list_prompts().await,
            "prompts/get" => self.handle_get_prompt(req.params).await,
            // Resources
            "resources/list" => self.handle_list_resources(req.params).await,
            "resources/read" => self.handle_read_resource(req.params).await,
            "resources/subscribe" => self.handle_subscribe_resource(req.params).await,
            "resources/unsubscribe" => self.handle_unsubscribe_resource(req.params).await,
            // Completions
            "completion/complete" => self.handle_completion(req.params).await,
            // Unknown
            _ => Err(Error::McpProtocol(format!(
                "Unknown method: {}",
                req.method
            ))),
        };

        // Remove from active requests
        self.active_requests.write().await.remove(&req.id);

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
                // Extract the request ID from params and cancel it
                if let Some(params) = notif.params {
                    #[derive(serde::Deserialize)]
                    struct CancelledParams {
                        #[serde(rename = "requestId")]
                        request_id: RequestId,
                    }
                    if let Ok(cancel) = serde_json::from_value::<CancelledParams>(params) {
                        info!("Cancelling request: {:?}", cancel.request_id);
                        self.active_requests
                            .write()
                            .await
                            .remove(&cancel.request_id);
                    }
                }
            }
            "notifications/roots/listChanged" => {
                info!("Client roots changed");
            }
            _ => {
                debug!("Unknown notification: {}", notif.method);
            }
        }
    }

    /// Handle initialize request.
    async fn handle_initialize(&self, params: Option<Value>) -> Result<Value> {
        // Extract roots from client if provided
        if let Some(ref params) = params {
            #[derive(serde::Deserialize)]
            struct InitParams {
                #[serde(default)]
                roots: Vec<RootInfo>,
            }
            #[derive(serde::Deserialize)]
            struct RootInfo {
                uri: String,
                #[serde(default)]
                name: Option<String>,
            }

            if let Ok(init) = serde_json::from_value::<InitParams>(params.clone()) {
                let mut roots = self.roots.write().await;
                for root in init.roots {
                    if let Some(path) = root.uri.strip_prefix("file://") {
                        roots.push(PathBuf::from(path));
                        info!("Added client root: {} ({:?})", path, root.name);
                    }
                }
            }
        }

        // Build capabilities based on what's configured
        let resources_cap = if self.resources.is_some() {
            Some(ResourcesCapability {
                subscribe: true,
                list_changed: true,
            })
        } else {
            None
        };

        let result = InitializeResult {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability { list_changed: true }),
                resources: resources_cap,
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

    /// Handle list resources request.
    async fn handle_list_resources(&self, params: Option<Value>) -> Result<Value> {
        let resources = self
            .resources
            .as_ref()
            .ok_or_else(|| Error::McpProtocol("Resources not enabled".to_string()))?;

        #[derive(serde::Deserialize, Default)]
        struct ListParams {
            cursor: Option<String>,
        }

        let list_params: ListParams = params
            .map(|v| serde_json::from_value(v).unwrap_or_default())
            .unwrap_or_default();

        let result = resources.list(list_params.cursor.as_deref()).await?;
        Ok(serde_json::to_value(result)?)
    }

    /// Handle read resource request.
    async fn handle_read_resource(&self, params: Option<Value>) -> Result<Value> {
        let resources = self
            .resources
            .as_ref()
            .ok_or_else(|| Error::McpProtocol("Resources not enabled".to_string()))?;

        #[derive(serde::Deserialize)]
        struct ReadParams {
            uri: String,
        }

        let read_params: ReadParams = params
            .ok_or_else(|| Error::InvalidToolArguments("Missing params".to_string()))
            .and_then(|v| {
                serde_json::from_value(v).map_err(|e| Error::InvalidToolArguments(e.to_string()))
            })?;

        let result = resources.read(&read_params.uri).await?;
        Ok(serde_json::to_value(result)?)
    }

    /// Handle subscribe to resource.
    async fn handle_subscribe_resource(&self, params: Option<Value>) -> Result<Value> {
        let resources = self
            .resources
            .as_ref()
            .ok_or_else(|| Error::McpProtocol("Resources not enabled".to_string()))?;

        #[derive(serde::Deserialize)]
        struct SubscribeParams {
            uri: String,
        }

        let sub_params: SubscribeParams = params
            .ok_or_else(|| Error::InvalidToolArguments("Missing params".to_string()))
            .and_then(|v| {
                serde_json::from_value(v).map_err(|e| Error::InvalidToolArguments(e.to_string()))
            })?;

        // Use a placeholder session ID for now
        resources.subscribe(&sub_params.uri, "default").await?;
        Ok(serde_json::json!({}))
    }

    /// Handle unsubscribe from resource.
    async fn handle_unsubscribe_resource(&self, params: Option<Value>) -> Result<Value> {
        let resources = self
            .resources
            .as_ref()
            .ok_or_else(|| Error::McpProtocol("Resources not enabled".to_string()))?;

        #[derive(serde::Deserialize)]
        struct UnsubscribeParams {
            uri: String,
        }

        let unsub_params: UnsubscribeParams = params
            .ok_or_else(|| Error::InvalidToolArguments("Missing params".to_string()))
            .and_then(|v| {
                serde_json::from_value(v).map_err(|e| Error::InvalidToolArguments(e.to_string()))
            })?;

        resources.unsubscribe(&unsub_params.uri, "default").await?;
        Ok(serde_json::json!({}))
    }

    /// Handle completion request.
    async fn handle_completion(&self, params: Option<Value>) -> Result<Value> {
        #[derive(serde::Deserialize)]
        struct CompletionParams {
            r#ref: CompletionRef,
            argument: CompletionArgument,
        }

        #[derive(serde::Deserialize)]
        #[allow(dead_code)]
        struct CompletionRef {
            r#type: String,
            #[serde(default)]
            uri: Option<String>,
            #[serde(default)]
            name: Option<String>,
        }

        #[derive(serde::Deserialize)]
        struct CompletionArgument {
            name: String,
            value: String,
        }

        let comp_params: CompletionParams = params
            .ok_or_else(|| Error::InvalidToolArguments("Missing params".to_string()))
            .and_then(|v| {
                serde_json::from_value(v).map_err(|e| Error::InvalidToolArguments(e.to_string()))
            })?;

        // Provide completions based on argument type
        let values = match comp_params.argument.name.as_str() {
            "path" | "file" | "uri" => {
                // File path completion
                self.complete_file_path(&comp_params.argument.value).await
            }
            "prompt" | "name" if comp_params.r#ref.r#type == "ref/prompt" => {
                // Prompt name completion
                self.prompts
                    .list()
                    .into_iter()
                    .filter(|p| p.name.starts_with(&comp_params.argument.value))
                    .map(|p| p.name)
                    .collect()
            }
            _ => Vec::new(),
        };

        Ok(serde_json::json!({
            "completion": {
                "values": values,
                "hasMore": false
            }
        }))
    }

    /// Complete file paths.
    async fn complete_file_path(&self, prefix: &str) -> Vec<String> {
        let roots = self.roots.read().await;
        let mut completions = Vec::new();

        // If we have resources, use that
        if let Some(ref resources) = self.resources {
            if let Ok(result) = resources.list(None).await {
                for resource in result.resources {
                    if resource.name.starts_with(prefix) {
                        completions.push(resource.name);
                    }
                }
            }
        }

        // Also check client-provided roots
        for root in roots.iter() {
            let search_path = root.join(prefix);
            if let Some(parent) = search_path.parent() {
                if let Ok(mut entries) = tokio::fs::read_dir(parent).await {
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let full = format!(
                            "{}{}",
                            prefix
                                .rsplit_once('/')
                                .map(|(p, _)| format!("{}/", p))
                                .unwrap_or_default(),
                            name
                        );
                        if full.starts_with(prefix) && !completions.contains(&full) {
                            completions.push(full);
                        }
                    }
                }
            }
        }

        completions.into_iter().take(20).collect()
    }
}
