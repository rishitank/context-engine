//! Augment SDK - Rust implementation.
//!
//! This module provides a complete Rust port of the Auggie SDK,
//! reverse-engineered from the TypeScript implementation.
//!
//! # Architecture
//!
//! - `api_client` - HTTP client for Augment backend API
//! - `blob` - Blob naming and size calculations
//! - `credentials` - Authentication resolution
//! - `direct_context` - Main context management class
//! - `retry` - Retry logic with exponential backoff
//! - `types` - SDK-specific types

pub mod api_client;
pub mod blob;
pub mod credentials;
pub mod direct_context;
pub mod retry;
pub mod types;

pub use api_client::ApiClient;
pub use blob::BlobNameCalculator;
pub use credentials::{resolve_credentials, Credentials};
pub use direct_context::DirectContext;
pub use types::*;
