# Testing Guide

This document describes the testing strategy and how to run tests for Context Engine.

## Test Overview

| Category | Count | Location | Description |
|----------|-------|----------|-------------|
| Unit Tests | 170 | `src/**/*.rs` | Core functionality tests |
| Integration Tests | 11 | `tests/` | MCP protocol and CLI tests |

## Running Tests

### Quick Start

```bash
# Run all unit tests
cargo test --lib

# Run integration tests (basic CLI tests only)
cargo test --test mcp_integration_test

# Run all tests including ignored integration tests
cargo test --test mcp_integration_test -- --ignored

# Run everything
cargo test --all-targets
```

### Unit Tests

Unit tests are embedded within source files using `#[cfg(test)]` modules.

```bash
# Run all unit tests
cargo test --lib

# Run tests for a specific module
cargo test --lib tools::language

# Run a specific test
cargo test --lib test_detect_rust_symbol

# Run with output
cargo test --lib -- --nocapture
```

### Integration Tests

Integration tests are in the `tests/` directory and test the MCP server as a whole.

```bash
# Run basic integration tests (CLI help/version)
cargo test --test mcp_integration_test

# Run full MCP protocol tests (spawns server, sends JSON-RPC)
cargo test --test mcp_integration_test -- --ignored

# Run a specific integration test
cargo test --test mcp_integration_test test_mcp_initialize -- --ignored
```

## Test Categories

### 1. Language Detection Tests (`src/tools/language.rs`)

Tests for multi-language symbol detection:

- `test_extension_to_language` - File extension mapping
- `test_detect_rust_symbol` - Rust symbol detection
- `test_detect_python_symbol` - Python symbol detection
- `test_detect_typescript_symbol` - TypeScript/JavaScript detection
- `test_detect_go_symbol` - Go symbol detection
- `test_detect_kotlin_symbol` - Kotlin symbol detection

### 2. Planning Service Tests (`src/service/planning.rs`)

- `test_create_plan` - Plan creation
- `test_add_step` - Step addition
- `test_update_step_status` - Status updates
- `test_plan_history` - History tracking

### 3. Review Type Tests (`src/types/review.rs`)

- `test_severity_ordering` - Severity comparison
- `test_finding_serialization` - JSON serialization
- `test_change_type_serialization` - Enum handling
- `test_diff_hunk` - Diff parsing

### 4. Search Type Tests (`src/types/search.rs`)

- `test_index_status_serialization` - Status serialization
- `test_search_result_optional_fields` - Optional field handling
- `test_chunk_serialization` - Chunk formatting

### 5. MCP Integration Tests (`tests/mcp_integration_test.rs`)

- `test_binary_help` - CLI --help flag
- `test_binary_version` - CLI --version flag
- `test_mcp_initialize` - MCP handshake
- `test_mcp_list_tools` - Tool listing
- `test_mcp_call_get_file` - File retrieval
- `test_mcp_list_resources` - Resource listing
- `test_mcp_list_prompts` - Prompt listing
- `test_mcp_workspace_stats` - Workspace stats tool
- `test_mcp_extract_symbols` - Symbol extraction
- `test_mcp_invalid_tool` - Error handling
- `test_mcp_invalid_file_path` - Invalid path handling

## Test Dependencies

```toml
[dev-dependencies]
tempfile = "3"              # Temporary directories
tokio-test = "0.4"          # Async test utilities
testcontainers = "0.26"     # Docker-based testing
testcontainers-modules = "0.14"
assert_cmd = "2"            # CLI testing
predicates = "3"            # Assertion predicates
```

## Writing New Tests

### Unit Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name() {
        // Arrange
        let input = "test input";

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected_output);
    }

    #[tokio::test]
    async fn test_async_feature() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### Integration Test Template

```rust
#[test]
#[ignore = "Requires running MCP server"]
fn test_mcp_feature() {
    let workspace = create_test_workspace();
    let mut client = McpTestClient::spawn(workspace.path().to_str().unwrap())
        .expect("Failed to spawn MCP server");

    client.initialize().expect("Failed to initialize");
    let response = client.call_tool("tool_name", json!({ "arg": "value" }))
        .expect("Failed to call tool");

    assert!(response.get("result").is_some());
}
```

## Code Coverage

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin --out Html --output-dir coverage

# Generate LCOV for CI
cargo tarpaulin --out Lcov
```

## Continuous Integration

Tests run automatically on every push and PR via GitHub Actions:

1. **Build** - Compile with `--release`
2. **Clippy** - Lint with `-D warnings`
3. **Format** - Check with `cargo fmt --check`
4. **Unit Tests** - Run `cargo test --lib`
5. **Integration Tests** - Run basic CLI tests

## Troubleshooting

### Tests hang or timeout

Integration tests spawn a real MCP server. If tests hang:

```bash
# Kill any orphaned processes
pkill -f context-engine

# Run with timeout
timeout 60 cargo test --test mcp_integration_test
```

### Docker tests fail

For testcontainers-based tests, ensure Docker is running:

```bash
docker info
```

