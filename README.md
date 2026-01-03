# Context Engine MCP Server

A high-performance, memory-safe Model Context Protocol (MCP) server written in Rust for AI-powered codebase context retrieval.

## Overview

Context Engine provides semantic code search and AI-powered context retrieval for coding agents. It integrates with the Augment Code SDK to offer:

- **Semantic Code Search**: AI-powered codebase retrieval using embeddings
- **File Indexing**: Automatic workspace indexing with intelligent file filtering
- **MCP Protocol**: Full Model Context Protocol support (JSON-RPC over stdio/HTTP)
- **Code Review**: Multi-pass review pipeline with risk scoring and invariant checking
- **Planning**: AI-assisted task planning and step management
- **Memory**: Persistent memory storage for context across sessions

## Features

| Metric | Value |
|--------|-------|
| **Binary Size** | ~7 MB (optimized ARM64) |
| **Lines of Code** | ~10,500 Rust |
| **Unit Tests** | 201 tests |
| **Integration Tests** | 11 tests |
| **MCP Tools** | 73 tools |
| **Supported Languages** | 18+ (symbol detection) |
| **Startup Time** | <10ms |
| **Memory Usage** | ~20 MB idle |

## Installation

### Prerequisites

- Rust 1.83+ (with cargo)
- Augment API credentials (via `~/.augment/session.json` or environment variables)

### Build from Source

```bash
cargo build --release
```

The binary will be at `target/release/context-engine`.

## Usage

### Command Line

```bash
# Start MCP server (stdio transport - default)
./target/release/context-engine --workspace /path/to/project

# Start with HTTP transport
./target/release/context-engine --workspace /path/to/project --transport http --port 3000

# Enable metrics endpoint
./target/release/context-engine --workspace /path/to/project --metrics --metrics-port 9090

# Debug mode
./target/release/context-engine --workspace /path/to/project --debug
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `AUGMENT_API_TOKEN` | API authentication token |
| `AUGMENT_API_URL` | API base URL |
| `CONTEXT_ENGINE_DEBUG` | Enable debug logging |

### Configuration

Credentials are resolved in order:
1. Command-line options
2. Environment variables
3. Session file (`~/.augment/session.json`)

## MCP Tools (73 Total)

### Retrieval Tools (7)
| Tool | Description |
|------|-------------|
| `codebase_retrieval` | Semantic search across the codebase |
| `semantic_search` | Search for code patterns and text |
| `get_file` | Retrieve file contents with optional line range |
| `get_context_for_prompt` | Get comprehensive context bundle |
| `enhance_prompt` | AI-powered prompt enhancement with context injection |
| `bundle_prompt` | Bundle raw prompt with codebase context (no AI rewriting) |
| `tool_manifest` | Discover available capabilities |

### Index Tools (5)
| Tool | Description |
|------|-------------|
| `index_workspace` | Index files for semantic search |
| `index_status` | Check indexing status |
| `reindex_workspace` | Clear and rebuild index |
| `clear_index` | Remove index state |
| `refresh_index` | Refresh the codebase index |

### Memory Tools (6)
| Tool | Description |
|------|-------------|
| `store_memory` | Store persistent memories |
| `retrieve_memory` | Recall stored memories |
| `list_memory` | List all memories |
| `delete_memory` | Delete a memory |
| `memory_store` | Store with rich metadata (kind, language, tags, priority) |
| `memory_find` | Hybrid search with filtering |

### Planning Tools (20)
| Tool | Description |
|------|-------------|
| `create_plan` | Create AI-powered implementation plans |
| `get_plan` | Get plan details |
| `list_plans` | List all plans |
| `add_step` | Add a step to a plan |
| `update_step` | Update step status |
| `refine_plan` | Refine plan with AI |
| `visualize_plan` | Generate visual representation |
| `execute_plan` | Execute plan steps |
| `save_plan` | Save plan to storage |
| `load_plan` | Load plan from storage |
| `delete_plan` | Delete a plan |
| `start_step` | Mark step as in progress |
| `complete_step` | Mark step as completed |
| `fail_step` | Mark step as failed |
| `view_progress` | View plan progress |
| `view_history` | View execution history |
| `request_approval` | Create approval request |
| `respond_approval` | Respond to approval request |
| `compare_plan_versions` | Generate diff between versions |
| `rollback_plan` | Rollback to previous version |

### Review Tools (14)
| Tool | Description |
|------|-------------|
| `review_diff` | Review code changes with risk analysis |
| `analyze_risk` | Analyze risk level of changes |
| `review_changes` | Review code changes in files |
| `review_git_diff` | Review current git diff |
| `review_auto` | Automatically review recent changes |
| `check_invariants` | Check code invariants |
| `run_static_analysis` | Run static analysis |
| `scrub_secrets` | Scan for potential secrets |
| `validate_content` | Validate content against rules |
| `get_review_status` | Get review status |
| `reactive_review_pr` | Start parallelized PR review |
| `pause_review` | Pause a running review session |
| `resume_review` | Resume a paused review session |
| `get_review_telemetry` | Get detailed review metrics |

### Navigation Tools (3)
| Tool | Description |
|------|-------------|
| `find_references` | Find all references to a symbol |
| `go_to_definition` | Navigate to symbol definition |
| `diff_files` | Compare two files with unified diff |

### Workspace Tools (7)
| Tool | Description |
|------|-------------|
| `workspace_stats` | Get workspace statistics and metrics |
| `git_status` | Get current git status |
| `extract_symbols` | Extract symbols from a file |
| `git_blame` | Get git blame information |
| `git_log` | Get git commit history |
| `dependency_graph` | Generate dependency graph |
| `file_outline` | Get file structure outline |

### Specialized Search Tools (7)
| Tool | Description |
|------|-------------|
| `search_tests_for` | Find test files with preset patterns |
| `search_config_for` | Find config files (yaml/json/toml/ini/env) |
| `search_callers_for` | Find callers/usages of a symbol |
| `search_importers_for` | Find files importing a module |
| `info_request` | Simplified retrieval with explanation mode |
| `pattern_search` | Structural code pattern matching |
| `context_search` | Context-aware semantic search |

## Architecture

```
src/
├── main.rs              # Entry point with CLI
├── lib.rs               # Library exports
├── error.rs             # Error types
├── config/              # Configuration management
├── sdk/                 # Augment API client
│   ├── api_client.rs    # HTTP client
│   ├── blob.rs          # SHA256 blob naming
│   ├── credentials.rs   # Auth resolution
│   └── direct_context.rs # Context operations
├── service/             # Business logic layer
│   ├── context.rs       # Context service
│   ├── memory.rs        # Memory service
│   └── planning.rs      # Planning service
├── mcp/                 # MCP protocol layer
│   ├── server.rs        # MCP server
│   ├── handler.rs       # Request handler
│   ├── protocol.rs      # JSON-RPC types
│   └── transport.rs     # Stdio/HTTP transports
├── tools/               # MCP tool implementations
├── reviewer/            # Code review pipeline
├── reactive/            # Reactive review system
├── watcher/             # File system watcher
├── http/                # HTTP server (axum)
├── metrics/             # Prometheus metrics
└── types/               # Shared type definitions
```

## Docker Support

The Docker image is ~20 MB (Alpine-based with statically linked binary).

### Build Docker Image

```bash
docker build -t context-engine .
```

### Run with Docker

```bash
# HTTP mode
docker run -d \
  -v /path/to/project:/workspace:ro \
  -v ~/.augment:/home/context-engine/.augment:ro \
  -p 3000:3000 \
  -p 9090:9090 \
  context-engine \
  --workspace /workspace \
  --transport http \
  --metrics

# Stdio mode (for MCP integration)
docker run -i \
  -v /path/to/project:/workspace:ro \
  -v ~/.augment:/home/context-engine/.augment:ro \
  context-engine \
  --workspace /workspace
```

### Docker Compose

```bash
# Set your project path
export PROJECT_PATH=/path/to/project

# Start services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop
docker-compose down
```

## Development

### Running Tests

```bash
# Run all unit tests (170 tests)
cargo test --lib

# Run integration tests (basic CLI tests)
cargo test --test mcp_integration_test

# Run full integration tests including MCP protocol tests
cargo test --test mcp_integration_test -- --ignored

# Run all tests
cargo test --all-targets
```

### Test Categories

| Category | Count | Description |
|----------|-------|-------------|
| Unit Tests | 201 | Core functionality tests |
| Integration Tests | 11 | MCP protocol and CLI tests |

### Linting

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Formatting

```bash
cargo fmt
```

### Code Coverage

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --out Html
```

## MCP Client Configuration

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "context-engine": {
      "command": "/path/to/context-engine",
      "args": ["--workspace", "/path/to/your/project"]
    }
  }
}
```

### Cursor

Add to your MCP configuration:

```json
{
  "context-engine": {
    "command": "/path/to/context-engine",
    "args": ["--workspace", "."]
  }
}
```

## License

MIT License - See LICENSE file for details.
