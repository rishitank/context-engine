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
| **Lines of Code** | ~8,800 Rust |
| **Unit Tests** | 107 tests |
| **MCP Tools** | 49 tools |
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

## MCP Tools (49 Total)

### Retrieval Tools (6)
| Tool | Description |
|------|-------------|
| `codebase_retrieval` | Semantic search across the codebase |
| `semantic_search` | Search for code patterns and text |
| `get_file` | Retrieve file contents with optional line range |
| `get_context_for_prompt` | Get comprehensive context bundle |
| `enhance_prompt` | AI-powered prompt enhancement |
| `tool_manifest` | Discover available capabilities |

### Index Tools (5)
| Tool | Description |
|------|-------------|
| `index_workspace` | Index files for semantic search |
| `index_status` | Check indexing status |
| `reindex_workspace` | Clear and rebuild index |
| `clear_index` | Remove index state |
| `refresh_index` | Refresh the codebase index |

### Memory Tools (4)
| Tool | Description |
|------|-------------|
| `store_memory` | Store persistent memories |
| `retrieve_memory` | Recall stored memories |
| `list_memory` | List all memories |
| `delete_memory` | Delete a memory |

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
cargo test
```

### Linting

```bash
cargo clippy
```

### Formatting

```bash
cargo fmt
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
