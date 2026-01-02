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

pub mod handler;
pub mod protocol;
pub mod server;
pub mod transport;

pub use handler::McpHandler;
pub use protocol::*;
pub use server::McpServer;
pub use transport::{StdioTransport, Transport};

