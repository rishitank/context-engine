//! Model Context Protocol (MCP) implementation.
//!
//! This module provides a complete implementation of the MCP protocol,
//! including JSON-RPC message handling, transport layers, and tool registration.
//!
//! # Architecture
//!
//! - `protocol` - Core MCP types and message definitions
//! - `server` - MCP server implementation
//! - `transport` - Transport layer (stdio, HTTP/SSE)
//! - `handler` - Request/notification handlers
//! - `prompts` - Prompt templates for common tasks
//! - `resources` - File resources for browsing codebase
//! - `progress` - Progress notifications for long-running operations

pub mod handler;
pub mod progress;
pub mod prompts;
pub mod protocol;
pub mod resources;
pub mod server;
pub mod transport;

pub use handler::McpHandler;
pub use progress::{ProgressManager, ProgressReporter, ProgressToken};
pub use prompts::PromptRegistry;
pub use protocol::*;
pub use resources::ResourceRegistry;
pub use server::McpServer;
pub use transport::{StdioTransport, Transport};
