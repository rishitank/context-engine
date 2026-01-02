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

/// Log level for the MCP server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogLevel {
    Debug,
    #[default]
    Info,
    Notice,
    Warning,
    Error,
    Critical,
    Alert,
    Emergency,
}

impl LogLevel {
    /// Converts a case-insensitive string into the corresponding `LogLevel`, defaulting to `Info` for unknown values.
    ///
    /// # Returns
    /// The matching `LogLevel` variant; `Info` if the input is not recognized.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::mcp::server::LogLevel;
    ///
    /// assert_eq!(LogLevel::from_str("debug"), LogLevel::Debug);
    /// assert_eq!(LogLevel::from_str("Warn"), LogLevel::Warning);
    /// assert_eq!(LogLevel::from_str("unknown-level"), LogLevel::Info);
    /// ```
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "debug" => Self::Debug,
            "info" => Self::Info,
            "notice" => Self::Notice,
            "warning" | "warn" => Self::Warning,
            "error" => Self::Error,
            "critical" => Self::Critical,
            "alert" => Self::Alert,
            "emergency" => Self::Emergency,
            _ => Self::Info,
        }
    }

    /// Get the lowercase string name for the log level.
    ///
    /// The returned string is a static, lowercase identifier corresponding to the variant
    /// (for example, `"info"`, `"warning"`, or `"error"`).
    ///
    /// # Examples
    ///
    /// ```
    /// let lvl = LogLevel::Info;
    /// assert_eq!(lvl.as_str(), "info");
    /// ```
    fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Notice => "notice",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
            Self::Alert => "alert",
            Self::Emergency => "emergency",
        }
    }
}

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
    /// Explicitly cancelled request IDs.
    cancelled_requests: Arc<RwLock<HashSet<RequestId>>>,
    /// Current log level.
    log_level: Arc<RwLock<LogLevel>>,
}

impl McpServer {
    /// Creates a new MCP server with default features.
    ///
    /// The returned server uses an empty prompt registry, no resource registry (resources disabled),
    /// empty workspace roots, no active or cancelled requests, and the default log level and version.
    ///
    /// # Examples
    ///
    /// ```
    /// // create a handler appropriate for your setup
    /// let handler = /* create or obtain an McpHandler instance */ ;
    /// let _server = McpServer::new(handler, "my-server");
    /// ```
    pub fn new(handler: McpHandler, name: impl Into<String>) -> Self {
        Self {
            handler: Arc::new(handler),
            prompts: Arc::new(PromptRegistry::new()),
            resources: None,
            name: name.into(),
            version: VERSION.to_string(),
            roots: Arc::new(RwLock::new(Vec::new())),
            active_requests: Arc::new(RwLock::new(HashSet::new())),
            cancelled_requests: Arc::new(RwLock::new(HashSet::new())),
            log_level: Arc::new(RwLock::new(LogLevel::default())),
        }
    }

    /// Create a McpServer configured with prompts and an initialized resources registry.
    ///
    /// The returned server wraps the provided handler and prompt registry in Arcs,
    /// constructs a ResourceRegistry from `context_service`, and initializes
    /// empty workspace roots, active/cancelled request tracking, and the default log level.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::sync::Arc;
    ///
    /// // Assume `handler`, `prompts`, and `context_service` are available.
    /// let server = McpServer::with_features(handler, prompts, Arc::new(context_service), "my-server");
    /// ```
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
            cancelled_requests: Arc::new(RwLock::new(HashSet::new())),
            log_level: Arc::new(RwLock::new(LogLevel::default())),
        }
    }

    /// Retrieve the server's current log level.
    ///
    /// # Returns
    ///
    /// `LogLevel` containing the server's active log level.
    ///
    /// # Examples
    ///
    /// ```
    /// # use futures::executor::block_on;
    /// # // `server` must be a `McpServer` instance
    /// # let server = todo!();
    /// let level = block_on(server.log_level());
    /// ```
    pub async fn log_level(&self) -> LogLevel {
        *self.log_level.read().await
    }

    /// Update the server's current logging level.
    ///
    /// This changes the level that the server uses for subsequent log messages.
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::mcp::server::{McpServer, LogLevel};
    /// # async fn doc_example(server: &McpServer) {
    /// server.set_log_level(LogLevel::Debug).await;
    /// # }
    /// ```
    pub async fn set_log_level(&self, level: LogLevel) {
        *self.log_level.write().await = level;
    }

    /// Retrieve the client-provided workspace roots.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Obtain an McpServer instance from your application context.
    /// let server: McpServer = unimplemented!();
    ///
    /// // Call the async method to get the current roots.
    /// let roots = futures::executor::block_on(server.roots());
    /// assert!(roots.iter().all(|p| p.is_absolute()));
    /// ```
    pub async fn roots(&self) -> Vec<PathBuf> {
        self.roots.read().await.clone()
    }

    /// Returns whether the given request ID has been explicitly cancelled.
    
    ///
    
    /// # Examples
    
    ///
    
    /// ```
    
    /// // Assuming `server: McpServer` and `id: RequestId` are available:
    
    /// // let cancelled = server.is_cancelled(&id).await;
    
    /// ```
    pub async fn is_cancelled(&self, id: &RequestId) -> bool {
        self.cancelled_requests.read().await.contains(id)
    }

    /// Marks the given request ID as cancelled so the server will treat it as cancelled on subsequent checks.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assuming `server` is an instance of `McpServer` and `req_id` is a `RequestId`:
    /// // server.cancel_request(&req_id).await;
    /// ```
    pub async fn cancel_request(&self, id: &RequestId) {
        self.cancelled_requests.write().await.insert(id.clone());
    }

    /// Remove a request from the server's active and cancelled tracking sets.
    ///
    /// This removes `id` from both `active_requests` and `cancelled_requests`, ensuring
    /// the server no longer treats the request as in-progress or cancelled.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use mcp::server::McpServer;
    /// # use mcp::RequestId;
    /// # async fn example(server: &McpServer, id: &RequestId) {
    /// server.complete_request(id).await;
    /// # }
    /// ```
    pub async fn complete_request(&self, id: &RequestId) {
        self.active_requests.write().await.remove(id);
        self.cancelled_requests.write().await.remove(id);
    }

    /// Run the server loop that processes incoming MCP messages on the provided transport.
    ///
    /// Starts the transport, receives messages until the transport ends or a send failure occurs,
    /// dispatches requests and notifications to the server handlers, stops the transport, and returns
    /// when the server has shut down.
    ///
    /// # Returns
    ///
    /// `Ok(())` on normal shutdown; an `Err` is returned if starting or stopping the transport fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::sync::Arc;
    /// # async fn _example(server: Arc<crate::mcp::server::McpServer>, transport: impl crate::transport::Transport) {
    /// server.run(transport).await.unwrap();
    /// # }
    /// ```
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

    /// Dispatches an incoming JSON-RPC request to the appropriate handler, tracks the request lifecycle for cancellation, and returns the corresponding JSON-RPC response.
    ///
    /// The request is registered as active while being processed; upon completion it is removed from active tracking. Known MCP methods are routed to their specific handlers; unknown methods produce a protocol error encoded in the response.
    ///
    /// # Returns
    ///
    /// `JsonRpcResponse` containing either a successful `result` value or an `error` describing the failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // `server` and `request` are assumed to be initialized appropriately.
    /// let resp = futures::executor::block_on(server.handle_request(request));
    /// assert_eq!(resp.jsonrpc, "2.0");
    /// ```
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
            // Logging
            "logging/setLevel" => self.handle_set_log_level(req.params).await,
            // Unknown
            _ => Err(Error::McpProtocol(format!(
                "Unknown method: {}",
                req.method
            ))),
        };

        // Clean up request tracking
        self.complete_request(&req.id).await;

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

    /// Process an incoming JSON-RPC notification and perform any side effects for known notification types.
    ///
    /// Known notifications handled:
    /// - "notifications/initialized": logs client initialization.
    /// - "notifications/cancelled": extracts a `requestId` from `params` and marks the request cancelled.
    /// - "notifications/roots/listChanged": logs that client workspace roots changed.
    /// Unknown notifications are ignored (logged at debug level).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use serde_json::json;
    ///
    /// // Build a cancelled notification with a `requestId` param.
    /// let notif = JsonRpcNotification {
    ///     jsonrpc: "2.0".into(),
    ///     method: "notifications/cancelled".into(),
    ///     params: Some(json!({ "requestId": "some-request-id" })),
    /// };
    ///
    /// // `server` is an instance of `McpServer`. Call will mark the request cancelled.
    /// // server.handle_notification(notif).await;
    /// ```
    async fn handle_notification(&self, notif: JsonRpcNotification) {
        debug!("Handling notification: {}", notif.method);

        match notif.method.as_str() {
            "notifications/initialized" => {
                info!("Client initialized");
            }
            "notifications/cancelled" => {
                // Extract the request ID from params and mark it as cancelled
                if let Some(params) = notif.params {
                    #[derive(serde::Deserialize)]
                    struct CancelledParams {
                        #[serde(rename = "requestId")]
                        request_id: RequestId,
                    }
                    if let Ok(cancel) = serde_json::from_value::<CancelledParams>(params) {
                        info!("Cancelling request: {:?}", cancel.request_id);
                        self.cancel_request(&cancel.request_id).await;
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

    /// Build and return the server's initialize result as JSON.
    ///
    /// If `params` includes client workspace roots with URIs beginning with `file://`,
    /// those paths are added to the server's tracked roots. The returned JSON contains
    /// the protocol version, server capabilities (including resources capability only
    /// if resources support is enabled), and server info (name and version).
    ///
    /// # Examples
    ///
    /// ```
    /// // Call on a server instance: returns an `InitializeResult` serialized as JSON.
    /// // let resp = server.handle_initialize(None).await.unwrap();
    /// // assert!(resp.get("protocol_version").is_some());
    /// ```
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

    /// Calls a named tool with the supplied parameters and returns the tool's result as JSON.
    ///
    /// Expects `params` to be a JSON-encoded `CallToolParams` object containing the tool `name` and `arguments`.
    ///
    /// # Returns
    ///
    /// The tool's execution result as a `serde_json::Value`.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidToolArguments` if `params` is missing or cannot be deserialized into `CallToolParams`,
    /// `Error::ToolNotFound` if no tool with the given name is registered, and propagates errors from the tool's
    /// execution or JSON serialization.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::json;
    ///
    /// // Example params: { "name": "echo", "arguments": ["hello"] }
    /// let params = Some(json!({ "name": "echo", "arguments": ["hello"] }));
    /// // let result = server.handle_call_tool(params).await.unwrap();
    /// ```
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

    /// List available prompts and return them as a JSON value.
    ///
    /// The returned JSON matches `ListPromptsResult` with the `prompts` field populated
    /// and `next_cursor` set to `null`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::mcp::prompts::ListPromptsResult;
    /// # tokio_test::block_on(async {
    /// // assume `server` is a constructed `McpServer`
    /// let json = server.handle_list_prompts().await.unwrap();
    /// let res: ListPromptsResult = serde_json::from_value(json).unwrap();
    /// assert!(res.next_cursor.is_none());
    /// # });
    /// ```
    async fn handle_list_prompts(&self) -> Result<Value> {
        use crate::mcp::prompts::ListPromptsResult;

        let prompts = self.prompts.list();
        let result = ListPromptsResult {
            prompts,
            next_cursor: None,
        };
        Ok(serde_json::to_value(result)?)
    }

    /// Fetches a prompt by name with optional arguments and returns it as JSON.
    ///
    /// Expects `params` to be a JSON object with a required `name` string and an optional
    /// `arguments` object mapping strings to strings. Returns the prompt result serialized
    /// to a `serde_json::Value`.
    ///
    /// Errors:
    /// - Returns `Error::InvalidToolArguments` if `params` is missing or cannot be deserialized.
    /// - Returns `Error::McpProtocol` if no prompt with the given name exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # use serde_json::json;
    /// # async fn _example(server: &crate::mcp::server::McpServer) {
    /// let params = json!({ "name": "welcome", "arguments": { "user": "Alex" } });
    /// let res = server.handle_get_prompt(Some(params)).await.unwrap();
    /// // `res` is a serde_json::Value containing the prompt result
    /// # }
    /// ```
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

    /// Lists available resources using an optional pagination cursor.
    ///
    /// If the server was built without resource support this returns an MCP protocol
    /// error indicating resources are not enabled. When resources are enabled, the
    /// optional `params` JSON may contain a `"cursor"` string used for paging; the
    /// function returns the serialized listing result from the resource registry.
    ///
    /// # Errors
    ///
    /// Returns `Error::McpProtocol("Resources not enabled")` if resources are not
    /// configured for the server, or propagates errors from the resource registry
    /// or JSON serialization.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct the optional params JSON with a cursor:
    /// let params = serde_json::json!({ "cursor": "page-2" });
    /// // Call: server.handle_list_resources(Some(params)).await
    /// ```
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

    /// Read a resource identified by a URI and return its serialized content as JSON.
    ///
    /// Returns an error if resources are not enabled, if required parameters are missing or malformed,
    /// or if the underlying resource read operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use serde_json::json;
    /// # use std::sync::Arc;
    /// # async fn _example(server: &crate::mcp::server::McpServer) {
    /// let params = json!({ "uri": "file:///path/to/resource" });
    /// let result = server.handle_read_resource(Some(params)).await;
    /// match result {
    ///     Ok(value) => {
    ///         // `value` is the JSON-serialized content returned by the resource registry.
    ///         println!("{}", value);
    ///     }
    ///     Err(e) => {
    ///         eprintln!("read failed: {:?}", e);
    ///     }
    /// }
    /// # }
    /// ```
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

    /// Subscribe the default session to a resource identified by URI.
    ///
    /// Returns an error if resources are not enabled for this server or if the required `params` are
    /// missing or cannot be deserialized.
    ///
    /// The request causes the server to call the configured ResourceRegistry's `subscribe` method for
    /// the provided URI using a placeholder session id ("default") and, on success, returns an empty
    /// JSON object.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use serde_json::json;
    /// # async fn example(server: &crate::mcp::McpServer) -> Result<(), Box<dyn std::error::Error>> {
    /// let params = json!({ "uri": "file:///path/to/resource" });
    /// let res = server.handle_subscribe_resource(Some(params)).await?;
    /// assert_eq!(res, json!({}));
    /// # Ok(()) }
    /// ```
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

    /// Provide completion suggestions for a completion request.
    ///
    /// Expects `params` to deserialize to `{ ref: { type, uri?, name? }, argument: { name, value } }`.
    /// For argument names "path", "file", or "uri" it returns filesystem/resource path completions;
    /// for argument name "prompt" when `ref.type == "ref/prompt"` it returns prompt-name completions.
    /// The response is a JSON object with a `completion` field containing `values` (an array of strings)
    /// and `hasMore` (a boolean).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use serde_json::json;
    ///
    /// // Example request params for completing prompt names starting with "ins"
    /// let params = json!({
    ///     "ref": { "type": "ref/prompt" },
    ///     "argument": { "name": "prompt", "value": "ins" }
    /// });
    ///
    /// // Expected shape of the response:
    /// let expected = json!({
    ///     "completion": {
    ///         "values": ["install", "instance"], // example values
    ///         "hasMore": false
    ///     }
    /// });
    /// ```
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

    /// Generates file-path completion candidates that start with the given prefix.
    ///
    /// The returned completions are sourced from the optional resource registry (if enabled)
    /// and from files/directories under client-provided workspace roots. Results are
    /// deduplicated and limited to at most 20 entries.
    ///
    /// # Returns
    ///
    /// A vector of completion strings that begin with `prefix`, up to 20 items.
    ///
    /// # Examples
    ///
    /// ```
    /// // `server` is an instance of `McpServer`.
    /// // This example assumes an async context (e.g., inside an async test).
    /// # async fn example(server: &crate::mcp::server::McpServer) {
    /// let completions = server.complete_file_path("src/").await;
    /// // completions contains candidates like "src/main.rs", "src/lib.rs", ...
    /// # }
    /// ```
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

    /// Set the server's log level from RPC parameters.
    ///
    /// Expects `params` to be a JSON object `{ "level": "<level>" }`. Parses the `level` string,
    /// updates the server's log level, logs the change, and returns an empty JSON object on success.
    /// If `params` is `None`, returns an MCP protocol error indicating the missing parameter.
    /// Unknown or unrecognized level strings map to the default level (Info).
    ///
    /// # Parameters
    ///
    /// - `params`: Optional JSON `Value` containing a `level` string specifying the desired log level.
    ///
    /// # Returns
    ///
    /// An empty JSON object `{}` on success.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn docs_example(server: &McpServer) {
    /// let res = server
    ///     .handle_set_log_level(Some(serde_json::json!({ "level": "debug" })))
    ///     .await
    ///     .unwrap();
    /// assert_eq!(res, serde_json::json!({}));
    /// # }
    /// ```
    async fn handle_set_log_level(&self, params: Option<Value>) -> Result<Value> {
        #[derive(serde::Deserialize)]
        struct SetLevelParams {
            level: String,
        }

        let level_str = if let Some(params) = params {
            let p: SetLevelParams = serde_json::from_value(params)?;
            p.level
        } else {
            return Err(Error::McpProtocol("Missing level parameter".to_string()));
        };

        let level = LogLevel::from_str(&level_str);
        self.set_log_level(level).await;

        info!("Log level set to: {}", level.as_str());
        Ok(serde_json::json!({}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("debug"), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("DEBUG"), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("info"), LogLevel::Info);
        assert_eq!(LogLevel::from_str("warning"), LogLevel::Warning);
        assert_eq!(LogLevel::from_str("warn"), LogLevel::Warning);
        assert_eq!(LogLevel::from_str("error"), LogLevel::Error);
        assert_eq!(LogLevel::from_str("critical"), LogLevel::Critical);
        assert_eq!(LogLevel::from_str("unknown"), LogLevel::Info); // Default
    }

    #[test]
    fn test_log_level_as_str() {
        assert_eq!(LogLevel::Debug.as_str(), "debug");
        assert_eq!(LogLevel::Info.as_str(), "info");
        assert_eq!(LogLevel::Warning.as_str(), "warning");
        assert_eq!(LogLevel::Error.as_str(), "error");
        assert_eq!(LogLevel::Emergency.as_str(), "emergency");
    }

    #[test]
    fn test_log_level_default() {
        assert_eq!(LogLevel::default(), LogLevel::Info);
    }
}