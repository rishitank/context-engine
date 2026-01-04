//! Context Engine MCP Server - Rust Implementation
//!
//! A high-performance Model Context Protocol (MCP) server for AI-powered code
//! context retrieval, planning, and review.
//!
//! ## Skills Architecture
//!
//! This server implements the "Tool Search Tool" pattern for Agent Skills:
//! - Skills are loaded from the `skills/` directory
//! - MCP clients can discover skills via `list_skills` and `search_skills` tools
//! - Full skill instructions are loaded on-demand via `load_skill`
//! - Skills are also exposed as MCP prompts for native MCP client support

use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

use context_engine_rs::config::{Args, Config, Transport};
use context_engine_rs::error::Result;
use context_engine_rs::mcp::handler::McpHandler;
use context_engine_rs::mcp::prompts::PromptRegistry;
use context_engine_rs::mcp::server::McpServer;
use context_engine_rs::mcp::skills::SkillRegistry;
use context_engine_rs::mcp::transport::StdioTransport;
use context_engine_rs::service::{ContextService, MemoryService, PlanningService};
use context_engine_rs::tools;
use context_engine_rs::VERSION;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_writer(std::io::stderr)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    // Build configuration from args
    let config: Config = args.into();

    info!("Context Engine MCP Server v{}", VERSION);
    info!("Workspace: {:?}", config.workspace);
    info!("Transport: {:?}", config.transport);

    // Initialize services
    let context_service = Arc::new(ContextService::new(&config).await?);
    let memory_service = Arc::new(MemoryService::new(&config.workspace).await?);
    let planning_service = Arc::new(PlanningService::new(&config.workspace).await?);

    // Initialize the context index
    info!("Initializing codebase index...");
    context_service.initialize().await?;
    let status = context_service.status().await;
    info!("Index ready: {} files indexed", status.file_count);

    // Initialize skills registry
    let skills_dir = PathBuf::from("skills");
    let skill_registry = Arc::new(RwLock::new(SkillRegistry::new(skills_dir)));
    {
        let mut registry = skill_registry.write().await;
        if let Err(e) = registry.load_skills().await {
            warn!("Failed to load skills: {}", e);
        } else {
            let count = registry.list().len();
            info!("Loaded {} skills", count);
        }
    }

    // Create MCP handler and register tools
    let mut handler = McpHandler::new();
    tools::register_all_tools(
        &mut handler,
        context_service.clone(),
        memory_service.clone(),
        planning_service.clone(),
    );
    tools::register_skills_tools(&mut handler, skill_registry.clone());
    info!("Registered {} MCP tools", handler.tool_count());

    // Create prompt registry and register skills as prompts
    let mut prompts = PromptRegistry::new();
    {
        let registry = skill_registry.read().await;
        prompts.register_skills(&registry);
        info!(
            "Registered {} skills as prompts",
            registry.list().len()
        );
    }

    // Start the server based on transport mode
    match config.transport {
        Transport::Stdio => {
            info!("Starting stdio transport...");
            let server =
                McpServer::with_features(handler, prompts, context_service, "context-engine");
            let transport = StdioTransport::new();
            server.run(transport).await?;
        }
        Transport::Http => {
            info!("Starting HTTP transport on port {}...", config.port);
            let handler = Arc::new(handler);
            context_engine_rs::http::start_server(&config, handler).await?;
        }
    }

    Ok(())
}
