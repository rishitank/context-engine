//! HTTP server for MCP over HTTP/SSE transport.
//!
//! Provides an alternative to stdio transport for web-based clients.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::config::Config;
use crate::error::Result;
use crate::mcp::handler::McpHandler;
use crate::mcp::protocol::*;

/// HTTP server state.
#[derive(Clone)]
pub struct HttpState {
    handler: Arc<McpHandler>,
    server_info: ServerInfo,
}

/// Start the HTTP server.
pub async fn start_server(config: &Config, handler: Arc<McpHandler>) -> Result<()> {
    let state = HttpState {
        handler,
        server_info: ServerInfo {
            name: "context-engine".to_string(),
            version: crate::VERSION.to_string(),
        },
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/mcp/initialize", post(initialize))
        .route("/mcp/tools/list", get(list_tools))
        .route("/mcp/tools/call", post(call_tool))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    info!("Starting HTTP server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check endpoint.
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": crate::VERSION
    }))
}

/// Initialize endpoint.
async fn initialize(State(state): State<HttpState>) -> impl IntoResponse {
    let result = InitializeResult {
        protocol_version: MCP_VERSION.to_string(),
        capabilities: ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: true }),
            resources: None,
            prompts: None,
            logging: Some(LoggingCapability {}),
        },
        server_info: state.server_info,
    };

    Json(result)
}

/// List tools endpoint.
async fn list_tools(State(state): State<HttpState>) -> impl IntoResponse {
    let tools = state.handler.list_tools();
    Json(ListToolsResult { tools })
}

/// Call tool request.
#[derive(Debug, Deserialize)]
struct CallToolRequest {
    name: String,
    #[serde(default)]
    arguments: std::collections::HashMap<String, serde_json::Value>,
}

/// Call tool endpoint.
async fn call_tool(
    State(state): State<HttpState>,
    Json(req): Json<CallToolRequest>,
) -> impl IntoResponse {
    let handler = match state.handler.get_tool(&req.name) {
        Some(h) => h,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": format!("Tool not found: {}", req.name)
                })),
            );
        }
    };

    match handler.execute(req.arguments).await {
        Ok(result) => (StatusCode::OK, Json(serde_json::to_value(result).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": e.to_string()
            })),
        ),
    }
}

