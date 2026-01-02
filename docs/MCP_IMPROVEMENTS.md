# MCP Server Improvement Roadmap

This document outlines potential improvements to make the Context Engine MCP Server more powerful and fully utilize the MCP specification.

## Current Implementation Status

### ✅ Fully Implemented
- **Tools** - All 49 tools for retrieval, indexing, memory, planning, and review
- **JSON-RPC 2.0** - Full request/response/notification handling
- **Stdio Transport** - Standard input/output for MCP clients
- **HTTP Transport** - Axum-based HTTP server with SSE
- **Logging Capability** - Structured logging support
- **Tools List Changed** - Dynamic tool list notifications

### 🔶 Partially Implemented
- **Resources** - Capability declared but not actively used
- **Prompts** - Capability declared but no prompts defined

### ❌ Not Yet Implemented
- **Resource Subscriptions** - Subscribe to file/resource changes
- **Prompt Templates** - Pre-defined prompt templates with arguments
- **Completions API** - Autocomplete suggestions for prompts/resources
- **Progress Notifications** - Long-running operation progress
- **Cancellation** - Cancel in-progress operations

---

## High-Value Improvements

### 1. Resource Subscriptions (High Priority)

Enable clients to subscribe to file changes in the codebase.

**Use Case:** Real-time code updates as files change

```json
// Subscribe to a file
{"method": "resources/subscribe", "params": {"uri": "file:///src/main.rs"}}

// Server sends notification when file changes
{"method": "notifications/resources/updated", "params": {"uri": "file:///src/main.rs"}}
```

**Implementation:**
- Integrate with existing `watcher` module for file system monitoring
- Track subscribed URIs per client session
- Emit notifications on file changes

### 2. Prompt Templates (High Priority)

Pre-defined prompts that guide AI assistants in common tasks.

**Proposed Prompts:**

| Prompt Name | Description | Arguments |
|-------------|-------------|-----------|
| `code_review` | Review code changes | `file_path`, `focus_areas` |
| `explain_code` | Explain a code section | `code`, `level` (beginner/advanced) |
| `write_tests` | Generate test cases | `file_path`, `function_name` |
| `debug_issue` | Help debug an issue | `error_message`, `stack_trace` |
| `refactor` | Suggest refactoring | `code`, `goals` |
| `document` | Generate documentation | `code`, `style` (jsdoc/rustdoc) |

**Implementation:**
- Add `prompts/list` and `prompts/get` handlers
- Store prompts as structured templates
- Support argument substitution

### 3. Progress Notifications (Medium Priority)

Report progress for long-running operations like indexing.

**Use Case:** Show progress during full codebase indexing

```json
// Server sends progress updates
{
  "method": "notifications/progress",
  "params": {
    "progressToken": "index-123",
    "progress": 45,
    "total": 100,
    "message": "Indexing src/..."
  }
}
```

**Implementation:**
- Add progress token to long-running tool calls
- Emit periodic progress notifications
- Track active operations for cancellation

### 4. Completions API (Medium Priority)

Provide autocomplete suggestions for tool arguments.

**Use Case:** Autocomplete file paths, function names

```json
// Request completions for file path
{
  "method": "completion/complete",
  "params": {
    "ref": {"type": "ref/resource", "uri": "file:///src/"},
    "argument": {"name": "path", "value": "src/m"}
  }
}

// Response
{
  "result": {
    "completion": {
      "values": ["src/main.rs", "src/mcp/", "src/metrics/"],
      "hasMore": true
    }
  }
}
```

**Implementation:**
- Integrate with index for file/symbol completion
- Cache recent completions for performance
- Support fuzzy matching

### 5. Request Cancellation (Low Priority)

Allow clients to cancel in-progress operations.

**Implementation:**
- Track active requests with cancellation tokens
- Check cancellation token during long operations
- Clean up resources on cancellation

---

## Performance Improvements

### 1. Caching Layer
- Cache semantic search results with LRU eviction
- Cache file content hashes for change detection
- Memoize expensive computations

### 2. Batch Operations
- Support batch tool calls in single request
- Parallel execution for independent operations

### 3. Streaming Responses
- Stream large search results
- Progressive rendering for code reviews

---

## Enhanced Tool Capabilities

### Current Tools (49)
- **Retrieval (6):** semantic_search, grep_search, file_search, etc.
- **Index (5):** index_status, index_directory, clear_index, etc.
- **Memory (4):** memory_store, memory_retrieve, memory_list, memory_delete
- **Planning (20):** create_review, analyze_changes, etc.
- **Review (14):** review_code, suggest_fixes, etc.

### Potential New Tools

| Tool | Description | Priority |
|------|-------------|----------|
| `diff_files` | Compare two files | High |
| `find_references` | Find all references to a symbol | High |
| `go_to_definition` | Find definition of a symbol | High |
| `call_hierarchy` | Show call graph for a function | Medium |
| `type_hierarchy` | Show class/type inheritance | Medium |
| `ast_query` | Query AST with tree-sitter | Medium |
| `git_blame` | Show git blame for a file | Low |
| `git_history` | Show commit history | Low |
| `dependency_graph` | Show module dependencies | Low |

---

## Architecture Improvements

### 1. Plugin System
Allow extending the server with custom tools without modifying core code.

```rust
// Plugin trait
trait McpPlugin {
    fn tools(&self) -> Vec<Tool>;
    fn resources(&self) -> Vec<Resource>;
    fn prompts(&self) -> Vec<Prompt>;
}
```

### 2. Multi-Workspace Support
Support multiple workspace roots simultaneously.

### 3. Language Server Protocol Integration
Bridge with LSP servers for richer code intelligence.

---

## Implementation Priority

### Phase 1 (Next Release)
1. ✅ Workflow improvements (PR-based releases)
2. ✅ Dependabot configuration
3. 🔲 Prompt templates (basic set)
4. 🔲 find_references tool
5. 🔲 go_to_definition tool

### Phase 2
1. 🔲 Resource subscriptions
2. 🔲 Progress notifications
3. 🔲 diff_files tool
4. 🔲 Caching layer

### Phase 3
1. 🔲 Completions API
2. 🔲 Plugin system
3. 🔲 AST query tool
4. 🔲 Request cancellation

---

## References

- [MCP Specification](https://modelcontextprotocol.io/specification)
- [MCP TypeScript SDK](https://github.com/modelcontextprotocol/typescript-sdk)
- [MCP Python SDK](https://github.com/modelcontextprotocol/python-sdk)

