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
    // resources/list may return error if no resources are indexed yet, which is OK
    // The important thing is we get a valid JSON-RPC response
    assert!(
        response.get("result").is_some() || response.get("error").is_some(),
        "Expected valid JSON-RPC response, got: {}",
        response
    );
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
    // Check we get a valid JSON-RPC response (result or error)
    assert!(
        response.get("result").is_some() || response.get("error").is_some(),
        "Expected valid JSON-RPC response, got: {}",
        response
    );
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

// ============================================================================
// Skills Integration Tests
// ============================================================================

fn create_test_workspace_with_skills() -> TempDir {
    let dir = create_test_workspace();

    // Create skills directory with test skills
    let skills_dir = dir.path().join("skills");
    std::fs::create_dir_all(&skills_dir).expect("Failed to create skills dir");

    // Create a test skill
    let debug_dir = skills_dir.join("debugging");
    std::fs::create_dir_all(&debug_dir).expect("Failed to create debugging skill dir");
    std::fs::write(debug_dir.join("SKILL.md"), r#"---
name: Debugging
description: Systematic debugging workflow
category: troubleshooting
tags:
  - bugs
  - errors
  - fix
always_apply: false
---

# Debugging Workflow

1. Reproduce the issue
2. Identify the root cause
3. Fix the bug
4. Verify the fix
"#).expect("Failed to write debugging skill");

    // Create another test skill
    let test_dir = skills_dir.join("testing");
    std::fs::create_dir_all(&test_dir).expect("Failed to create testing skill dir");
    std::fs::write(test_dir.join("SKILL.md"), r#"---
name: Testing
description: Write comprehensive tests
category: quality
tags:
  - unit-tests
  - integration
always_apply: false
---

# Testing Workflow

Write good tests that cover edge cases.
"#).expect("Failed to write testing skill");

    dir
}

#[test]
#[ignore = "Requires running MCP server - run with --ignored"]
fn test_mcp_list_skills() {
    let workspace = create_test_workspace_with_skills();
    let mut client = McpTestClient::spawn(workspace.path().to_str().unwrap())
        .expect("Failed to spawn MCP server");

    client.initialize().expect("Failed to initialize");
    let response = client
        .call_tool("list_skills", json!({}))
        .expect("Failed to list skills");

    assert!(response.get("result").is_some(), "Expected result");
    let result = &response["result"];
    let content = result["content"].as_array().expect("content should be array");
    assert!(!content.is_empty(), "Expected content");

    // Parse the text content
    if let Some(text) = content[0]["text"].as_str() {
        let parsed: Value = serde_json::from_str(text).expect("Should parse as JSON");
        assert!(parsed["count"].as_i64().unwrap() >= 2, "Should have at least 2 skills");
        assert!(parsed["skills"].is_array(), "Should have skills array");
    }
}

#[test]
#[ignore = "Requires running MCP server - run with --ignored"]
fn test_mcp_search_skills() {
    let workspace = create_test_workspace_with_skills();
    let mut client = McpTestClient::spawn(workspace.path().to_str().unwrap())
        .expect("Failed to spawn MCP server");

    client.initialize().expect("Failed to initialize");
    let response = client
        .call_tool("search_skills", json!({ "query": "debug" }))
        .expect("Failed to search skills");

    assert!(response.get("result").is_some(), "Expected result");
    let result = &response["result"];
    let content = result["content"].as_array().expect("content should be array");

    if let Some(text) = content[0]["text"].as_str() {
        let parsed: Value = serde_json::from_str(text).expect("Should parse as JSON");
        assert!(parsed["count"].as_i64().unwrap() >= 1, "Should find at least 1 skill");

        let skills = parsed["skills"].as_array().unwrap();
        let ids: Vec<&str> = skills.iter().filter_map(|s| s["id"].as_str()).collect();
        assert!(ids.contains(&"debugging"), "Should find debugging skill");
    }
}

#[test]
#[ignore = "Requires running MCP server - run with --ignored"]
fn test_mcp_load_skill() {
    let workspace = create_test_workspace_with_skills();
    let mut client = McpTestClient::spawn(workspace.path().to_str().unwrap())
        .expect("Failed to spawn MCP server");

    client.initialize().expect("Failed to initialize");
    let response = client
        .call_tool("load_skill", json!({ "id": "debugging" }))
        .expect("Failed to load skill");

    assert!(response.get("result").is_some(), "Expected result");
    let result = &response["result"];
    let content = result["content"].as_array().expect("content should be array");

    if let Some(text) = content[0]["text"].as_str() {
        let parsed: Value = serde_json::from_str(text).expect("Should parse as JSON");
        assert_eq!(parsed["id"].as_str().unwrap(), "debugging");
        assert_eq!(parsed["name"].as_str().unwrap(), "Debugging");
        assert!(parsed["instructions"].as_str().unwrap().contains("Debugging Workflow"));
    }
}

#[test]
#[ignore = "Requires running MCP server - run with --ignored"]
fn test_mcp_load_skill_not_found() {
    let workspace = create_test_workspace_with_skills();
    let mut client = McpTestClient::spawn(workspace.path().to_str().unwrap())
        .expect("Failed to spawn MCP server");

    client.initialize().expect("Failed to initialize");
    let response = client
        .call_tool("load_skill", json!({ "id": "nonexistent_skill" }))
        .expect("Failed to load skill");

    assert!(response.get("result").is_some(), "Expected result");
    let result = &response["result"];
    let content = result["content"].as_array().expect("content should be array");

    if let Some(text) = content[0]["text"].as_str() {
        let parsed: Value = serde_json::from_str(text).expect("Should parse as JSON");
        assert!(parsed["error"].as_str().unwrap().contains("not found"));
        assert!(parsed["available_skills"].is_array());
    }
}

#[test]
#[ignore = "Requires running MCP server - run with --ignored"]
fn test_mcp_list_skills_filter_by_category() {
    let workspace = create_test_workspace_with_skills();
    let mut client = McpTestClient::spawn(workspace.path().to_str().unwrap())
        .expect("Failed to spawn MCP server");

    client.initialize().expect("Failed to initialize");
    let response = client
        .call_tool("list_skills", json!({ "category": "quality" }))
        .expect("Failed to list skills");

    assert!(response.get("result").is_some(), "Expected result");
    let result = &response["result"];
    let content = result["content"].as_array().expect("content should be array");

    if let Some(text) = content[0]["text"].as_str() {
        let parsed: Value = serde_json::from_str(text).expect("Should parse as JSON");
        let skills = parsed["skills"].as_array().unwrap();

        // All returned skills should have "quality" category
        for skill in skills {
            assert_eq!(skill["category"], "quality");
        }
    }
}

#[test]
#[ignore = "Requires running MCP server - run with --ignored"]
fn test_mcp_skill_prompts_available() {
    let workspace = create_test_workspace_with_skills();
    let mut client = McpTestClient::spawn(workspace.path().to_str().unwrap())
        .expect("Failed to spawn MCP server");

    client.initialize().expect("Failed to initialize");
    let response = client.list_prompts().expect("Failed to list prompts");

    assert!(response.get("result").is_some(), "Expected result");
    let result = &response["result"];
    let prompts = result["prompts"].as_array().expect("prompts should be array");

    let prompt_names: Vec<&str> = prompts.iter().filter_map(|p| p["name"].as_str()).collect();

    // Should have skill prompts
    assert!(
        prompt_names.iter().any(|n| n.starts_with("skill:")),
        "Expected skill prompts, got: {:?}",
        prompt_names
    );
}
