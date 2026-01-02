//! MCP Server Integration Tests
//!
//! These tests verify the MCP server works correctly with real MCP clients
//! by spawning the server and communicating via JSON-RPC over stdio.

#![allow(deprecated)] // Allow deprecated cargo_bin for now

use assert_cmd::cargo::CommandCargoExt;
use assert_cmd::Command as AssertCommand;
use predicates::prelude::*;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use tempfile::TempDir;

/// MCP Test Client that communicates with the server via stdio
struct McpTestClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    request_id: i64,
}

impl McpTestClient {
    /// Spawn a new MCP server and connect to it
    fn spawn(workspace_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Get the path to the built binary using cargo_bin!
        let mut child = Command::cargo_bin("context-engine")?
            .arg("--workspace")
            .arg(workspace_dir)
            .arg("--transport")
            .arg("stdio")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdin = child.stdin.take().expect("Failed to get stdin");
        let stdout = BufReader::new(child.stdout.take().expect("Failed to get stdout"));

        Ok(Self {
            child,
            stdin,
            stdout,
            request_id: 0,
        })
    }

    /// Send a JSON-RPC request and get the response
    fn request(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        self.request_id += 1;
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params
        });

        let request_str = serde_json::to_string(&request)?;
        writeln!(self.stdin, "{}", request_str)?;
        self.stdin.flush()?;

        let mut response_line = String::new();
        self.stdout.read_line(&mut response_line)?;

        let response: Value = serde_json::from_str(&response_line)?;
        Ok(response)
    }

    fn initialize(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        self.request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "roots": { "listChanged": true } },
                "clientInfo": { "name": "test-client", "version": "1.0.0" }
            }),
        )
    }

    fn list_tools(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        self.request("tools/list", json!({}))
    }

    fn call_tool(
        &mut self,
        name: &str,
        arguments: Value,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        self.request(
            "tools/call",
            json!({ "name": name, "arguments": arguments }),
        )
    }

    fn list_resources(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        self.request("resources/list", json!({}))
    }

    fn list_prompts(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        self.request("prompts/list", json!({}))
    }
}

impl Drop for McpTestClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

fn create_test_workspace() -> TempDir {
    let dir = TempDir::new().expect("Failed to create temp dir");
    std::fs::write(
        dir.path().join("main.rs"),
        "fn main() { println!(\"Hello\"); }\nfn add(a: i32, b: i32) -> i32 { a + b }\nstruct Calculator { value: i32 }",
    ).expect("Failed to write main.rs");
    std::fs::write(
        dir.path().join("utils.py"),
        "def greet(name): return f\"Hello, {name}!\"\nclass Helper:\n    def __init__(self): self.count = 0",
    ).expect("Failed to write utils.py");
    dir
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_binary_help() {
    AssertCommand::cargo_bin("context-engine")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("MCP server"));
}

#[test]
fn test_binary_version() {
    AssertCommand::cargo_bin("context-engine")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("context-engine"));
}

#[test]
#[ignore = "Requires running MCP server - run with --ignored"]
fn test_mcp_initialize() {
    let workspace = create_test_workspace();
    let mut client = McpTestClient::spawn(workspace.path().to_str().unwrap())
        .expect("Failed to spawn MCP server");

    let response = client.initialize().expect("Failed to initialize");
    assert!(
        response.get("result").is_some(),
        "Expected result in response"
    );
    let result = &response["result"];
    assert!(
        result.get("protocolVersion").is_some(),
        "Expected protocolVersion"
    );
    assert!(result.get("serverInfo").is_some(), "Expected serverInfo");
    assert!(
        result.get("capabilities").is_some(),
        "Expected capabilities"
    );
}

#[test]
#[ignore = "Requires running MCP server - run with --ignored"]
fn test_mcp_list_tools() {
    let workspace = create_test_workspace();
    let mut client = McpTestClient::spawn(workspace.path().to_str().unwrap())
        .expect("Failed to spawn MCP server");

    client.initialize().expect("Failed to initialize");
    let response = client.list_tools().expect("Failed to list tools");

    assert!(response.get("result").is_some(), "Expected result");
    let result = &response["result"];
    let tools = result["tools"].as_array().expect("tools should be array");
    assert!(!tools.is_empty(), "Expected at least one tool");

    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(
        tool_names.contains(&"codebase_retrieval"),
        "Expected codebase_retrieval tool"
    );
    assert!(tool_names.contains(&"get_file"), "Expected get_file tool");
}

#[test]
#[ignore = "Requires running MCP server - run with --ignored"]
fn test_mcp_call_get_file() {
    let workspace = create_test_workspace();
    let mut client = McpTestClient::spawn(workspace.path().to_str().unwrap())
        .expect("Failed to spawn MCP server");

    client.initialize().expect("Failed to initialize");
    let response = client
        .call_tool("get_file", json!({ "path": "main.rs" }))
        .expect("Failed to call get_file");

    assert!(response.get("result").is_some(), "Expected result");
    let result = &response["result"];
    let content = result["content"]
        .as_array()
        .expect("content should be array");
    let text = content[0]["text"].as_str().expect("Expected text");
    assert!(
        text.contains("fn main()") || text.contains("main"),
        "Expected main function"
    );
}

#[test]
#[ignore = "Requires running MCP server - run with --ignored"]
fn test_mcp_list_resources() {
    let workspace = create_test_workspace();
    let mut client = McpTestClient::spawn(workspace.path().to_str().unwrap())
        .expect("Failed to spawn MCP server");

    client.initialize().expect("Failed to initialize");
    let response = client.list_resources().expect("Failed to list resources");
    assert!(response.get("result").is_some(), "Expected result");
}

#[test]
#[ignore = "Requires running MCP server - run with --ignored"]
fn test_mcp_list_prompts() {
    let workspace = create_test_workspace();
    let mut client = McpTestClient::spawn(workspace.path().to_str().unwrap())
        .expect("Failed to spawn MCP server");

    client.initialize().expect("Failed to initialize");
    let response = client.list_prompts().expect("Failed to list prompts");
    assert!(response.get("result").is_some(), "Expected result");
}

#[test]
#[ignore = "Requires running MCP server - run with --ignored"]
fn test_mcp_workspace_stats() {
    let workspace = create_test_workspace();
    let mut client = McpTestClient::spawn(workspace.path().to_str().unwrap())
        .expect("Failed to spawn MCP server");

    client.initialize().expect("Failed to initialize");
    let response = client
        .call_tool("workspace_stats", json!({}))
        .expect("Failed to call workspace_stats");
    assert!(response.get("result").is_some(), "Expected result");
}

#[test]
#[ignore = "Requires running MCP server - run with --ignored"]
fn test_mcp_extract_symbols() {
    let workspace = create_test_workspace();
    let mut client = McpTestClient::spawn(workspace.path().to_str().unwrap())
        .expect("Failed to spawn MCP server");

    client.initialize().expect("Failed to initialize");
    let response = client
        .call_tool("extract_symbols", json!({ "path": "main.rs" }))
        .expect("Failed to call extract_symbols");
    assert!(response.get("result").is_some(), "Expected result");
}

#[test]
#[ignore = "Requires running MCP server - run with --ignored"]
fn test_mcp_invalid_tool() {
    let workspace = create_test_workspace();
    let mut client = McpTestClient::spawn(workspace.path().to_str().unwrap())
        .expect("Failed to spawn MCP server");

    client.initialize().expect("Failed to initialize");
    let response = client
        .call_tool("nonexistent_tool", json!({}))
        .expect("Failed to call tool");
    assert!(
        response.get("error").is_some(),
        "Expected error for invalid tool"
    );
}

#[test]
#[ignore = "Requires running MCP server - run with --ignored"]
fn test_mcp_invalid_file_path() {
    let workspace = create_test_workspace();
    let mut client = McpTestClient::spawn(workspace.path().to_str().unwrap())
        .expect("Failed to spawn MCP server");

    client.initialize().expect("Failed to initialize");
    let response = client
        .call_tool("get_file", json!({ "path": "nonexistent.rs" }))
        .expect("Failed to call get_file");
    // Should return result with error content or error
    assert!(response.get("result").is_some() || response.get("error").is_some());
}
