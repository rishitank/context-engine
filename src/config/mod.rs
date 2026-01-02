//! Configuration management for the Context Engine.

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Command-line arguments for the Context Engine server.
#[derive(Parser, Debug, Clone)]
#[command(name = "context-engine")]
#[command(author = "Context Engine Team")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "High-performance MCP server for AI-powered code context retrieval")]
pub struct Args {
    /// Workspace root directory
    #[arg(short, long, env = "CONTEXT_ENGINE_WORKSPACE")]
    pub workspace: Option<PathBuf>,

    /// Transport mode: stdio or http
    #[arg(short, long, default_value = "stdio", env = "CONTEXT_ENGINE_TRANSPORT")]
    pub transport: Transport,

    /// HTTP port (only for http transport)
    #[arg(short, long, default_value = "3000", env = "CONTEXT_ENGINE_PORT")]
    pub port: u16,

    /// Enable debug logging
    #[arg(short, long, env = "CONTEXT_ENGINE_DEBUG")]
    pub debug: bool,

    /// Enable file watcher
    #[arg(long, default_value = "true", env = "CONTEXT_ENGINE_WATCH")]
    pub watch: bool,

    /// Augment API key (overrides session file)
    #[arg(long, env = "AUGMENT_API_TOKEN")]
    pub api_key: Option<String>,

    /// Augment API URL (overrides session file)
    #[arg(long, env = "AUGMENT_API_URL")]
    pub api_url: Option<String>,

    /// Maximum file size for indexing (bytes)
    #[arg(long, default_value = "1048576", env = "CONTEXT_ENGINE_MAX_FILE_SIZE")]
    pub max_file_size: usize,

    /// Token budget for context windows
    #[arg(long, default_value = "8000", env = "CONTEXT_ENGINE_TOKEN_BUDGET")]
    pub token_budget: usize,

    /// Enable metrics collection
    #[arg(long, env = "CONTEXT_ENGINE_METRICS")]
    pub metrics: bool,

    /// Metrics port
    #[arg(long, default_value = "9090", env = "CONTEXT_ENGINE_METRICS_PORT")]
    pub metrics_port: u16,
}

/// Transport mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
    #[default]
    Stdio,
    Http,
}

/// Server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Workspace root directory
    pub workspace: PathBuf,
    /// Transport mode
    pub transport: Transport,
    /// HTTP port
    pub port: u16,
    /// Debug mode
    pub debug: bool,
    /// File watcher enabled
    pub watch: bool,
    /// API key
    pub api_key: Option<String>,
    /// API URL
    pub api_url: Option<String>,
    /// Maximum file size
    pub max_file_size: usize,
    /// Token budget
    pub token_budget: usize,
    /// Metrics enabled
    pub metrics: bool,
    /// Metrics port
    pub metrics_port: u16,
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        Self {
            workspace: args.workspace.unwrap_or_else(|| {
                std::env::current_dir().expect("Failed to get current directory")
            }),
            transport: args.transport,
            port: args.port,
            debug: args.debug,
            watch: args.watch,
            api_key: args.api_key,
            api_url: args.api_url,
            max_file_size: args.max_file_size,
            token_budget: args.token_budget,
            metrics: args.metrics,
            metrics_port: args.metrics_port,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            workspace: std::env::current_dir().expect("Failed to get current directory"),
            transport: Transport::Stdio,
            port: 3000,
            debug: false,
            watch: true,
            api_key: None,
            api_url: None,
            max_file_size: 1024 * 1024,
            token_budget: 8000,
            metrics: false,
            metrics_port: 9090,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_default() {
        assert_eq!(Transport::default(), Transport::Stdio);
    }

    #[test]
    fn test_transport_serialization() {
        let transports = [
            (Transport::Stdio, "\"stdio\""),
            (Transport::Http, "\"http\""),
        ];

        for (transport, expected) in &transports {
            let json = serde_json::to_string(transport).unwrap();
            assert_eq!(json, *expected);
        }
    }

    #[test]
    fn test_transport_deserialization() {
        let stdio: Transport = serde_json::from_str("\"stdio\"").unwrap();
        assert_eq!(stdio, Transport::Stdio);

        let http: Transport = serde_json::from_str("\"http\"").unwrap();
        assert_eq!(http, Transport::Http);
    }

    #[test]
    fn test_config_default_values() {
        let config = Config::default();

        assert_eq!(config.transport, Transport::Stdio);
        assert_eq!(config.port, 3000);
        assert!(!config.debug);
        assert!(config.watch);
        assert!(config.api_key.is_none());
        assert!(config.api_url.is_none());
        assert_eq!(config.max_file_size, 1024 * 1024);
        assert_eq!(config.token_budget, 8000);
        assert!(!config.metrics);
        assert_eq!(config.metrics_port, 9090);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            transport: Transport::Http,
            port: 8080,
            debug: true,
            ..Config::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"transport\":\"http\""));
        assert!(json.contains("\"port\":8080"));
        assert!(json.contains("\"debug\":true"));
    }

    #[test]
    fn test_config_deserialization() {
        let json = r#"{
            "workspace": "/tmp/test",
            "transport": "http",
            "port": 8080,
            "debug": true,
            "watch": false,
            "api_key": "test-key",
            "api_url": "https://api.example.com",
            "max_file_size": 2097152,
            "token_budget": 16000,
            "metrics": true,
            "metrics_port": 9091
        }"#;

        let config: Config = serde_json::from_str(json).unwrap();

        assert_eq!(config.transport, Transport::Http);
        assert_eq!(config.port, 8080);
        assert!(config.debug);
        assert!(!config.watch);
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.max_file_size, 2097152);
        assert!(config.metrics);
    }

    #[test]
    fn test_args_to_config() {
        let args = Args {
            workspace: Some(PathBuf::from("/test/workspace")),
            transport: Transport::Http,
            port: 4000,
            debug: true,
            watch: false,
            api_key: Some("key123".to_string()),
            api_url: Some("https://api.test.com".to_string()),
            max_file_size: 500000,
            token_budget: 4000,
            metrics: true,
            metrics_port: 9095,
        };

        let config: Config = args.into();

        assert_eq!(config.workspace, PathBuf::from("/test/workspace"));
        assert_eq!(config.transport, Transport::Http);
        assert_eq!(config.port, 4000);
        assert!(config.debug);
        assert!(!config.watch);
        assert_eq!(config.api_key, Some("key123".to_string()));
    }
}
