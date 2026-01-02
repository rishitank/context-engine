//! Context Engine MCP Server - Rust Implementation
//!
//! A high-performance Model Context Protocol (MCP) server for AI-powered code
//! context retrieval, planning, and review. This is a complete Rust port of the
//! TypeScript implementation, designed for maximum performance and memory safety.
//!
//! # Architecture
//!
//! The server follows a 5-layer architecture:
//!
//! 1. **SDK Layer** (`sdk`) - API client for Augment backend (embeddings, search, LLM)
//! 2. **Service Layer** (`service`) - Business logic, caching, context bundling
//! 3. **MCP Layer** (`mcp`) - Protocol implementation, transport handling
//! 4. **Tools Layer** (`tools`) - 35+ MCP tools for retrieval, planning, review
//! 5. **Specialized Modules** - Reviewer pipeline, reactive review, HTTP server
//!
//! # Features
//!
//! - **Semantic Search**: Natural language code search via Augment API
//! - **Planning**: AI-powered task planning with DAG dependencies
//! - **Code Review**: Multi-pass review with risk scoring and invariants
//! - **Memory**: Persistent memory storage for agent context
//! - **Reactive Review**: Session-based PR reviews with parallel execution

pub mod config;
pub mod error;
pub mod http;
pub mod mcp;
pub mod metrics;
pub mod reactive;
pub mod reviewer;
pub mod sdk;
pub mod service;
pub mod tools;
pub mod types;
pub mod watcher;

pub use error::{Error, Result};

/// Server version matching the TypeScript implementation
pub const VERSION: &str = "1.9.0";

/// Maximum file size for indexing (1MB)
pub const MAX_FILE_SIZE: usize = 1024 * 1024;

/// Default token budget for context windows
pub const DEFAULT_TOKEN_BUDGET: usize = 8000;
