# MCP Server Improvement Roadmap

This document outlines potential improvements to make the Context Engine MCP Server more powerful and fully utilize the MCP specification.

## Current Implementation Status

### ✅ Fully Implemented
- **Tools** - All 59 tools for retrieval, indexing, memory, planning, review, navigation, and workspace analysis
- **JSON-RPC 2.0** - Full request/response/notification handling
- **Stdio Transport** - Standard input/output for MCP clients
- **HTTP Transport** - Axum-based HTTP server with SSE
- **Logging Capability** - Structured logging support with `logging/setLevel` handler
- **Tools List Changed** - Dynamic tool list notifications
- **Resources** - Full `resources/list` and `resources/read` with file:// URI scheme
- **Resource Subscriptions** - Subscribe/unsubscribe to file changes
- **Prompts** - 5 pre-defined prompt templates with argument substitution
- **Completions API** - Autocomplete suggestions for tool/prompt arguments
- **Progress Notifications** - Long-running operation progress with ProgressReporter
- **Cancellation** - Cancel in-progress operations via `notifications/cancelled`
- **Roots Support** - Client-provided workspace roots via `roots/list`
- **Navigation Tools** - `find_references`, `go_to_definition`, `diff_files`
- **Workspace Tools** - `workspace_stats`, `git_status`, `extract_symbols`

### 🔶 Partially Implemented
- **Resource Templates** - URI templates for dynamic resources (planned)

### ❌ Not Yet Implemented
- **Sampling** - Server-initiated LLM requests (requires client support)

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

### Current Tools (59)
- **Retrieval (6):** semantic_search, grep_search, file_search, etc.
- **Index (5):** index_status, index_directory, clear_index, etc.
- **Memory (4):** memory_store, memory_retrieve, memory_list, memory_delete
- **Planning (20):** create_review, analyze_changes, etc.
- **Review (14):** review_code, suggest_fixes, etc.
- **Navigation (3):** find_references, go_to_definition, diff_files
- **Workspace (7):** workspace_stats, git_status, extract_symbols, git_blame, git_log, dependency_graph, file_outline

### Potential New Tools

| Tool | Description | Priority | Status |
|------|-------------|----------|--------|
| `diff_files` | Compare two files | High | ✅ Implemented |
| `find_references` | Find all references to a symbol | High | ✅ Implemented |
| `go_to_definition` | Find definition of a symbol | High | ✅ Implemented |
| `call_hierarchy` | Show call graph for a function | Medium | 🔲 Planned |
| `type_hierarchy` | Show class/type inheritance | Medium | 🔲 Planned |
| `ast_query` | Query AST with tree-sitter | Medium | 🔲 Planned |
| `git_blame` | Show git blame for a file | Low | ✅ Implemented |
| `git_history` | Show commit history | Low | ✅ Implemented (git_log) |
| `dependency_graph` | Show module dependencies | Low | ✅ Implemented |
| `file_outline` | Get structured outline of symbols | Low | ✅ Implemented |

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

### Phase 1 (v2.0.0 - Complete ✅)
1. ✅ Workflow improvements (PR-based releases)
2. ✅ Dependabot configuration
3. ✅ Prompt templates (5 templates with conditionals)
4. ✅ find_references tool
5. ✅ go_to_definition tool
6. ✅ Resource subscriptions
7. ✅ Progress notifications
8. ✅ diff_files tool
9. ✅ Completions API
10. ✅ Request cancellation
11. ✅ Workspace analysis tools (workspace_stats, git_status, extract_symbols)
12. ✅ logging/setLevel handler

### Phase 2 (Next)
1. 🔲 Caching layer for expensive operations
2. 🔲 Plugin system for extensibility
3. 🔲 AST query tool (tree-sitter integration)
4. 🔲 Dependency graph analysis

### Phase 3 (Future)
1. 🔲 LSP integration for richer code intelligence
2. 🔲 Sampling support (server-initiated LLM requests)
3. 🔲 Resource templates for dynamic URIs

---

## Future Enhancements (from Code Review)

The following enhancements were identified during code review and are documented for future implementation:

### High Priority

#### 1. Percent-Encoded URI Decoding
**File:** `src/mcp/server.rs`

Currently, file URIs with percent-encoded characters (like spaces as `%20`) may not resolve correctly. Adding the `percent-encoding` crate would enable proper URI decoding.

```rust
use percent_encoding::percent_decode_str;

// When parsing file:// URIs
if let Some(path) = root.uri.strip_prefix("file://") {
    let decoded = percent_decode_str(path).decode_utf8_lossy();
    roots.push(PathBuf::from(decoded.as_ref()));
}
```

#### 2. Proper Session Management
**File:** `src/mcp/server.rs`

The current implementation uses a hardcoded `"default"` session ID for subscriptions. Proper session management would require:

1. Generating unique session IDs during `initialize`
2. Passing session context through the request handling chain
3. Cleaning up subscriptions when sessions end
4. Tracking per-session state

```rust
pub struct SessionManager {
    sessions: HashMap<String, SessionState>,
}

impl SessionManager {
    pub fn create_session(&mut self) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.sessions.insert(id.clone(), SessionState::default());
        id
    }

    pub fn cleanup_session(&mut self, id: &str) {
        self.sessions.remove(id);
    }
}
```

### Medium Priority

#### 3. Progress Reporting Improvements
**File:** `src/mcp/progress.rs`

- Add warning log when `complete()` is called without a total being set
- Add debug logging when notification receiver is dropped

```rust
// In complete() method
if self.total.is_none() {
    tracing::warn!("complete() called without total set");
}

// In send methods
if self.sender.send(notification).is_err() {
    tracing::debug!("Progress notification receiver dropped");
}
```

#### 4. TypeScript Function Detection Accuracy
**File:** `src/tools/workspace.rs`

The current detection using `line.contains("function ")` can produce false positives on comments like `// This function does...`. A more precise approach:

```rust
// Check if "function " appears at start or after export/async keywords
let trimmed = line.trim();
if trimmed.starts_with("function ")
    || trimmed.starts_with("export function ")
    || trimmed.starts_with("async function ")
    || trimmed.starts_with("export async function ") {
    // Process as function declaration
}
```

#### 5. Extensionless File Handling
**File:** `src/tools/workspace.rs`

Currently, extensionless files are excluded from workspace statistics. Add special handling for known extensionless files:

```rust
fn is_known_extensionless_file(name: &str) -> Option<&'static str> {
    match name {
        "Makefile" | "GNUmakefile" => Some("make"),
        "Dockerfile" => Some("dockerfile"),
        "Jenkinsfile" => Some("groovy"),
        "Procfile" => Some("procfile"),
        "Vagrantfile" => Some("ruby"),
        "Gemfile" => Some("ruby"),
        "Rakefile" => Some("ruby"),
        ".gitignore" | ".gitattributes" => Some("git"),
        ".env" | ".env.example" => Some("env"),
        _ => None,
    }
}
```

### Low Priority

#### 6. Language Category Naming
**File:** `src/tools/workspace.rs`

The `extension_to_language()` function returns `"binary"` for unknown extensions. Consider renaming to `"other"` or `"unknown"` for accuracy, since extensionless files aren't necessarily binary.

```rust
fn extension_to_language(ext: &str) -> &'static str {
    match ext {
        // ... existing mappings ...
        _ => "other",  // Changed from "binary"
    }
}
```

#### 7. Async I/O in Resource Discovery
**File:** `src/mcp/resources.rs`

Some operations use blocking I/O patterns in async context. Consider using `tokio::task::spawn_blocking` for CPU-intensive operations or ensuring all file I/O uses async variants consistently.

#### 8. Silent Fallback on Malformed Params
**File:** `src/mcp/server.rs`

Some handlers silently use defaults when params are malformed. Consider adding debug logging for better troubleshooting:

```rust
let list_params: ListParams = params
    .map(|v| {
        serde_json::from_value(v.clone()).unwrap_or_else(|e| {
            tracing::debug!("Failed to parse params, using defaults: {}", e);
            ListParams::default()
        })
    })
    .unwrap_or_default();
```

---

## Known Limitations

The following are documented limitations of the current implementation:

| Limitation | Workaround | Future Fix |
|------------|------------|------------|
| Percent-encoded URIs not decoded | Use paths without special characters | Add `percent-encoding` crate |
| Single session support | Works for single-client scenarios | Implement session manager |
| TypeScript function false positives | Use `extract_symbols` for accurate results | Improve line-start detection |
| Extensionless files excluded | Manually include in analysis | Add known extensionless mapping |
| Hardcoded "binary" label | Use extension mapping | Rename to "other"/"unknown" |

---

## References

- [MCP Specification](https://modelcontextprotocol.io/specification)
- [MCP TypeScript SDK](https://github.com/modelcontextprotocol/typescript-sdk)
- [MCP Python SDK](https://github.com/modelcontextprotocol/python-sdk)

