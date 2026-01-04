# AGENTS.md

This file provides guidance for AI coding agents working with the Context Engine MCP Server.

## Project Overview

Context Engine is a high-performance Model Context Protocol (MCP) server written in Rust that provides AI-powered code context retrieval, planning, and review capabilities.

## Skills

This project includes Agent Skills that provide workflow guidance for complex tasks.

| Skill | Description | When to Use |
|-------|-------------|-------------|
| `planning` | Task planning and execution workflow | Breaking down complex multi-step tasks |
| `code_review` | Comprehensive code review workflow | Reviewing PRs, analyzing risks, checking quality |
| `search_patterns` | Specialized search patterns | Finding tests, configs, callers, semantic search |

### Loading Skills

Skills are available via MCP tools:

```
# List all available skills
list_skills()

# Search for relevant skills
search_skills(query: "code review")

# Load full skill instructions
load_skill(id: "code_review")
```

## Architecture

```
src/
├── config/          # Configuration and CLI args
├── error.rs         # Error types
├── http/            # HTTP transport
├── mcp/             # MCP protocol implementation
│   ├── handler.rs   # Tool handler
│   ├── prompts.rs   # Prompt templates
│   ├── resources.rs # File resources
│   ├── server.rs    # MCP server
│   ├── skills.rs    # Agent Skills support
│   └── transport.rs # Transport layer
├── service/         # Business logic services
│   ├── context.rs   # Context/search service
│   ├── memory.rs    # Memory persistence
│   └── planning.rs  # Planning service
├── tools/           # MCP tool implementations
│   ├── retrieval.rs # Codebase search tools
│   ├── planning.rs  # Planning tools
│   ├── review.rs    # Code review tools
│   ├── skills.rs    # Skills discovery tools
│   └── ...
└── types/           # Shared type definitions
```

## Development Guidelines

### Building

```bash
cargo build
cargo test --lib
```

### Running

```bash
# Stdio transport (default)
cargo run -- --workspace /path/to/project

# HTTP transport
cargo run -- --workspace /path/to/project --transport http --port 3000
```

### Code Style

- Use `rustfmt` for formatting
- Use `clippy` for linting
- Follow Rust naming conventions
- Add doc comments for public APIs

### Testing

- Unit tests in the same file as the code
- Integration tests in `tests/` directory
- Run tests with `cargo test`

## MCP Tools

The server provides 72 MCP tools organized by category:

- **Retrieval** (7): `codebase_retrieval`, `search_code`, `get_file`, etc.
- **Index** (5): `index_workspace`, `index_status`, etc.
- **Memory** (6): `store_memory`, `retrieve_memory`, etc.
- **Planning** (20): `create_plan`, `add_step`, `complete_step`, etc.
- **Review** (14): `review_diff`, `analyze_risk`, etc.
- **Navigation** (3): `find_references`, `go_to_definition`, `diff_files`
- **Workspace** (7): `workspace_stats`, `git_status`, etc.
- **Specialized Search** (7): `search_tests_for`, `search_config_for`, etc.
- **Skills** (3): `list_skills`, `search_skills`, `load_skill`

## Key Patterns

### Tool Search Tool Pattern

For skills discovery, use the progressive disclosure pattern:

1. Call `list_skills()` or `search_skills(query)` to find relevant skills
2. Call `load_skill(id)` to get full instructions
3. Follow the skill instructions to complete the task

This reduces token overhead by loading skill content on-demand.

### Error Handling

All tools return a `ToolResult` with:
- `content`: Array of content items (text, images, resources)
- `isError`: Boolean indicating if the operation failed

### Async Operations

Long-running operations support progress notifications via MCP progress tokens.

## Contributing

1. Create a feature branch
2. Make changes with tests
3. Run `cargo test` and `cargo clippy`
4. Submit a PR

## License

MIT

