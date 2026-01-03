# API Reference

Complete reference for all 73 MCP tools provided by Context Engine.

## Table of Contents

- [Retrieval Tools](#retrieval-tools-6)
- [Index Tools](#index-tools-5)
- [Memory Tools](#memory-tools-6)
- [Planning Tools](#planning-tools-20)
- [Review Tools](#review-tools-14)
- [Navigation Tools](#navigation-tools-3)
- [Workspace Tools](#workspace-tools-7)
- [Specialized Search Tools](#specialized-search-tools-7)

---

## Retrieval Tools (6)

### `codebase_retrieval`

Search the codebase using natural language. Returns relevant code snippets and context based on semantic understanding.

**Input Schema:**
```json
{
  "information_request": "string (required) - Natural language description of what you're looking for",
  "max_tokens": "integer (optional) - Maximum tokens in the response"
}
```

**Example:**
```json
{
  "information_request": "How is user authentication implemented?",
  "max_tokens": 4000
}
```

---

### `semantic_search`

Search for code patterns, functions, classes, or specific text in the codebase.

**Input Schema:**
```json
{
  "query": "string (required) - Search query (can be natural language or code pattern)",
  "file_pattern": "string (optional) - Glob pattern to filter files (e.g., '*.rs', 'src/**/*.ts')",
  "max_results": "integer (optional) - Maximum number of results to return"
}
```

**Example:**
```json
{
  "query": "async function that handles HTTP requests",
  "file_pattern": "src/**/*.rs",
  "max_results": 10
}
```

---

### `get_file`

Retrieve complete or partial contents of a file from the codebase.

**Input Schema:**
```json
{
  "path": "string (required) - File path relative to workspace root",
  "start_line": "integer (optional) - First line to include (1-based)",
  "end_line": "integer (optional) - Last line to include (1-based)"
}
```

**Example:**
```json
{
  "path": "src/main.rs",
  "start_line": 1,
  "end_line": 50
}
```

---

### `get_context_for_prompt`

Get relevant codebase context optimized for prompt enhancement.

**Input Schema:**
```json
{
  "query": "string (required) - Description of what you need context for",
  "max_files": "integer (optional) - Maximum number of files to include (default: 5, max: 20)",
  "token_budget": "integer (optional) - Maximum tokens for the entire context (default: 8000)",
  "include_related": "boolean (optional) - Include related/imported files (default: true)",
  "min_relevance": "number (optional) - Minimum relevance score 0-1 (default: 0.3)"
}
```

---

### `enhance_prompt`

Transform a simple prompt into a detailed, structured prompt by injecting relevant codebase context and using AI to create actionable instructions. The enhanced prompt will reference specific files, functions, and patterns from your codebase.

**Input Schema:**
```json
{
  "prompt": "string (required) - The simple prompt to enhance with codebase context (max 10000 chars)",
  "token_budget": "integer (optional) - Maximum tokens for codebase context (default: 6000)"
}
```

**What it does:**
1. Retrieves relevant codebase context based on your prompt
2. Bundles the context with your original prompt
3. Uses AI to create an enhanced, actionable prompt that references specific code locations

---

### `bundle_prompt`

Bundle a raw prompt with relevant codebase context. Returns the original prompt alongside retrieved code snippets, file summaries, and related context. Use this when you want direct control over how the context is used without AI rewriting.

**Input Schema:**
```json
{
  "prompt": "string (required) - The prompt to bundle with codebase context (max 10000 chars)",
  "token_budget": "integer (optional) - Maximum tokens for codebase context (default: 8000)",
  "format": "string (optional) - Output format: 'structured' (sections), 'formatted' (single string), or 'json' (machine-readable). Default: 'structured'",
  "system_instruction": "string (optional) - Optional system instruction to include in the formatted output"
}
```

**Use cases:**
- AI agents that need to construct their own prompts with context
- Custom prompt engineering workflows
- Building context-aware tool chains

---

### `tool_manifest`

Discover available tools and capabilities exposed by the server.

**Input Schema:**
```json
{}
```

**Response includes:** version, capabilities list, and all available tool names.

---

## Index Tools (5)

### `index_workspace`

Index the current workspace for semantic search.

**Input Schema:**
```json
{
  "force": "boolean (optional) - Force re-indexing even if index exists (default: false)",
  "background": "boolean (optional) - Run indexing in background thread (non-blocking)"
}
```

**Indexed file types (50+):**
- TypeScript/JavaScript: `.ts`, `.tsx`, `.js`, `.jsx`, `.mjs`, `.cjs`
- Python: `.py`, `.pyi`
- Systems: `.go`, `.rs`, `.java`, `.kt`, `.c`, `.cpp`, `.h`, `.hpp`, `.swift`
- Web: `.vue`, `.svelte`, `.astro`, `.html`, `.css`, `.scss`
- Config: `.json`, `.yaml`, `.yml`, `.toml`, `.xml`
- Documentation: `.md`, `.txt`

---

### `index_status`

Get the current status of the codebase index.

**Input Schema:**
```json
{}
```

**Response includes:** indexed file count, last indexed time, workspace path.

---

### `reindex_workspace`

Clear current index state and rebuild from scratch.

**Input Schema:**
```json
{}
```

---

### `clear_index`

Remove saved index state and clear caches without rebuilding.

**Input Schema:**
```json
{}
```

---

### `refresh_index`

Refresh the codebase index by re-scanning all files.

**Input Schema:**
```json
{
  "force": "boolean (optional) - Force full re-index even if files haven't changed"
}
```

---

## Memory Tools (6)

### `add_memory`

Store a piece of information in persistent memory for later retrieval.

**Input Schema:**
```json
{
  "key": "string (required) - Unique key to identify this memory",
  "value": "string (required) - The information to store",
  "type": "string (optional) - Category/type for the memory"
}
```

**Example:**
```json
{
  "key": "project-architecture",
  "value": "This project uses a layered architecture with services, handlers, and tools",
  "type": "documentation"
}
```

---

### `retrieve-memory`

Retrieve a previously stored memory by its key.

**Input Schema:**
```json
{
  "key": "string (required) - The key of the memory to retrieve"
}
```

---

### `list_memories`

List all stored memories, optionally filtered by type.

**Input Schema:**
```json
{
  "type": "string (optional) - Type to filter memories"
}
```

---

### `delete-memory`

Delete a stored memory by its key.

**Input Schema:**
```json
{
  "key": "string (required) - The key of the memory to delete"
}
```

---

### `memory_store`

Store information with rich metadata for enhanced retrieval. Compatible with m1rl0k/Context-Engine.

**Input Schema:**
```json
{
  "key": "string (required) - Unique key to identify this memory",
  "information": "string (required) - The information to store",
  "kind": "string (optional) - Type of memory: snippet, explanation, pattern, example, reference, memory",
  "language": "string (optional) - Programming language if applicable",
  "path": "string (optional) - File path if related to a specific file",
  "tags": "array (optional) - Tags for categorization",
  "priority": "integer (optional) - Priority 1-10 (higher = more important)",
  "topic": "string (optional) - Topic or subject area",
  "code": "string (optional) - Associated code snippet",
  "author": "string (optional) - Author of the memory"
}
```

**Example:**
```json
{
  "key": "auth-pattern",
  "information": "JWT authentication pattern used in this project",
  "kind": "pattern",
  "language": "typescript",
  "tags": ["auth", "jwt", "security"],
  "priority": 8,
  "topic": "authentication"
}
```

---

### `memory_find`

Find memories using hybrid search with filtering. Compatible with m1rl0k/Context-Engine.

**Input Schema:**
```json
{
  "query": "string (required) - Search query",
  "kind": "string (optional) - Filter by kind: snippet, explanation, pattern, example, reference, memory",
  "language": "string (optional) - Filter by programming language",
  "topic": "string (optional) - Filter by topic",
  "tags": "array (optional) - Filter by tags (any match)",
  "priority_min": "integer (optional) - Minimum priority (1-10)",
  "limit": "integer (optional) - Maximum results (default: 10)"
}
```

**Example:**
```json
{
  "query": "authentication",
  "kind": "pattern",
  "language": "typescript",
  "priority_min": 5,
  "limit": 5
}
```

---

## Planning Tools (20)

### `create_plan`

Create a new plan for a task or feature implementation.

**Input Schema:**
```json
{
  "title": "string (required) - Title of the plan",
  "description": "string (required) - Detailed description of what the plan accomplishes"
}
```

---

### `get_plan`

Get details of a specific plan by ID.

**Input Schema:**
```json
{
  "plan_id": "string (required) - The ID of the plan to retrieve"
}
```

---

### `list_plans`

List all plans, optionally filtered by status.

**Input Schema:**
```json
{
  "status": "string (optional) - Status filter (draft, active, completed, etc.)"
}
```

---

### `add_step`

Add a step to an existing plan.

**Input Schema:**
```json
{
  "plan_id": "string (required) - The plan ID",
  "title": "string (required) - Step title",
  "description": "string (required) - Step description",
  "step_type": "string (optional) - Type of step"
}
```

---

### `update_step`

Update the status of a step in a plan.

**Input Schema:**
```json
{
  "plan_id": "string (required) - The plan ID",
  "step_id": "integer (required) - The step ID",
  "status": "string (required) - New status (pending, ready, in_progress, completed, failed, skipped)"
}
```

---

### `refine_plan`

Refine an existing plan with AI assistance.

**Input Schema:**
```json
{
  "plan_id": "string (required) - The plan ID to refine",
  "feedback": "string (optional) - Feedback or instructions for refinement"
}
```

---

### `visualize_plan`

Generate a visual representation of a plan (Mermaid diagram).

**Input Schema:**
```json
{
  "plan_id": "string (required) - The plan ID to visualize",
  "format": "string (optional) - Output format (mermaid, ascii, json)"
}
```

---

### `execute_plan`

Execute a plan step by step.

**Input Schema:**
```json
{
  "plan_id": "string (required) - The plan ID to execute",
  "auto_approve": "boolean (optional) - Auto-approve steps"
}
```

---

### `save_plan`

Save a plan to persistent storage.

**Input Schema:**
```json
{
  "plan_id": "string (required) - The plan ID to save",
  "path": "string (optional) - File path"
}
```

---

### `load_plan`

Load a plan from persistent storage.

**Input Schema:**
```json
{
  "path": "string (required) - File path to load from"
}
```

---

### `delete_plan`

Delete a plan.

**Input Schema:**
```json
{
  "plan_id": "string (required) - The plan ID to delete"
}
```

---

### `start_step`

Mark a step as in progress.

**Input Schema:**
```json
{
  "plan_id": "string (required) - The plan ID",
  "step_id": "integer (required) - The step ID"
}
```

---

### `complete_step`

Mark a step as completed.

**Input Schema:**
```json
{
  "plan_id": "string (required) - The plan ID",
  "step_id": "integer (required) - The step ID",
  "output": "string (optional) - Output/result"
}
```

---

### `fail_step`

Mark a step as failed.

**Input Schema:**
```json
{
  "plan_id": "string (required) - The plan ID",
  "step_id": "integer (required) - The step ID",
  "error": "string (optional) - Error message"
}
```

---

### `view_progress`

View the progress of a plan.

**Input Schema:**
```json
{
  "plan_id": "string (required) - The plan ID"
}
```

**Response includes:** total_steps, completed, in_progress, failed, pending, progress_percent.

---

### `view_history`

View the execution history of a plan.

**Input Schema:**
```json
{
  "plan_id": "string (required) - The plan ID"
}
```

---

### `request_approval`

Create an approval request for a plan or specific steps.

**Input Schema:**
```json
{
  "plan_id": "string (required) - Plan ID",
  "step_numbers": "array of integers (optional) - Specific steps to approve"
}
```

---

### `respond_approval`

Respond to an approval request.

**Input Schema:**
```json
{
  "request_id": "string (required) - Approval request ID",
  "action": "string (required) - Action to take: 'approve' or 'reject'",
  "comments": "string (optional) - Comments"
}
```

---

### `compare_plan_versions`

Generate a diff between two plan versions.

**Input Schema:**
```json
{
  "plan_id": "string (required) - Plan ID",
  "from_version": "integer (required) - Source version",
  "to_version": "integer (required) - Target version"
}
```

---

### `rollback_plan`

Rollback a plan to a previous version.

**Input Schema:**
```json
{
  "plan_id": "string (required) - Plan ID",
  "version": "integer (required) - Version to rollback to",
  "reason": "string (optional) - Reason for rollback"
}
```

---

## Review Tools (14)

### `review_diff`

Review a code diff and provide feedback on potential issues.

**Input Schema:**
```json
{
  "diff": "string (required) - The unified diff to review",
  "context": "string (optional) - Context about the changes"
}
```

---

### `analyze_risk`

Analyze the risk level of proposed code changes.

**Input Schema:**
```json
{
  "files": "array of strings (required) - List of files being changed",
  "change_description": "string (required) - Description of the changes"
}
```

**Response includes:** Risk Level (HIGH/MEDIUM/LOW), Risk Score (0-100), analysis.

---

### `review_changes`

Review code changes in specified files.

**Input Schema:**
```json
{
  "files": "array of strings (required) - List of files to review"
}
```

---

### `review_git_diff`

Review the current git diff.

**Input Schema:**
```json
{
  "base": "string (optional) - Base branch/commit (default: HEAD~1)",
  "head": "string (optional) - Head branch/commit (default: HEAD)"
}
```

---

### `check_invariants`

Check code invariants and constraints.

**Input Schema:**
```json
{
  "files": "array of strings (optional) - Files to check"
}
```

---

### `run_static_analysis`

Run static analysis on the codebase.

**Input Schema:**
```json
{
  "files": "array of strings (optional) - Files to analyze"
}
```

---

### `scrub_secrets`

Scan content for potential secrets and sensitive data.

**Input Schema:**
```json
{
  "content": "string (required) - Content to scan"
}
```

**Detects:** API keys, secret keys, passwords, tokens, bearer tokens.

---

### `validate_content`

Validate content against rules and constraints.

**Input Schema:**
```json
{
  "content": "string (required) - Content to validate",
  "rules": "array of strings (optional) - Validation rules to apply"
}
```

---

### `reactive_review_pr`

Start a session-based, parallelized code review.

**Input Schema:**
```json
{
  "pr_number": "integer (optional) - PR number to review",
  "base": "string (optional) - Base branch",
  "head": "string (optional) - Head branch"
}
```

---

### `get_review_status`

Get the status of an ongoing review.

**Input Schema:**
```json
{
  "review_id": "string (optional) - Review ID"
}
```

---

### `pause_review`

Pause a running review session.

**Input Schema:**
```json
{
  "session_id": "string (required) - Session ID to pause"
}
```

---

### `resume_review`

Resume a paused review session.

**Input Schema:**
```json
{
  "session_id": "string (required) - Session ID to resume"
}
```

---

### `get_review_telemetry`

Get detailed metrics for a review session.

**Input Schema:**
```json
{
  "session_id": "string (required) - Session ID"
}
```

**Response includes:** tokens_used, cache_hits, cache_misses, duration_ms.

---

## Navigation Tools (3)

### `find_references`

Find all references to a symbol in the codebase.

**Input Schema:**
```json
{
  "symbol": "string (required) - The symbol name to find references for",
  "file_pattern": "string (optional) - Glob pattern to filter files"
}
```

---

### `go_to_definition`

Navigate to the definition of a symbol.

**Input Schema:**
```json
{
  "symbol": "string (required) - The symbol name to find definition for",
  "file_pattern": "string (optional) - Glob pattern to filter files"
}
```

---

### `diff_files`

Compare two files and show differences.

**Input Schema:**
```json
{
  "file1": "string (required) - Path to first file",
  "file2": "string (required) - Path to second file",
  "context_lines": "integer (optional) - Number of context lines (default: 3)"
}
```

---

## Workspace Tools (7)

### `workspace_stats`

Get comprehensive workspace statistics.

**Input Schema:**
```json
{}
```

**Returns:** File counts by type, total lines of code, repository information.

---

### `git_status`

Get current git status of the workspace.

**Input Schema:**
```json
{}
```

**Returns:** Modified, staged, and untracked files.

---

### `extract_symbols`

Extract all symbols (functions, classes, etc.) from a file.

**Input Schema:**
```json
{
  "path": "string (required) - File path relative to workspace"
}
```

**Supported Languages:** Rust, Python, TypeScript, JavaScript, Go, Java, C, C++, Ruby, PHP, Swift, Kotlin, Scala, Elixir, Haskell, Lua, Dart, Clojure, and more.

---

### `git_blame`

Get git blame information for a file.

**Input Schema:**
```json
{
  "path": "string (required) - File path relative to workspace",
  "start_line": "integer (optional) - Starting line number",
  "end_line": "integer (optional) - Ending line number"
}
```

---

### `git_log`

Get git commit history.

**Input Schema:**
```json
{
  "path": "string (optional) - File path to get history for",
  "max_count": "integer (optional) - Maximum number of commits (default: 10)"
}
```

---

### `dependency_graph`

Generate a dependency graph for the project.

**Input Schema:**
```json
{
  "format": "string (optional) - Output format: 'mermaid' or 'text' (default: 'mermaid')"
}
```

---

### `file_outline`

Get the structural outline of a file.

**Input Schema:**
```json
{
  "path": "string (required) - File path relative to workspace"
}
```

**Returns:** Hierarchical structure of symbols in the file.

---

## Specialized Search Tools (7)

These tools are compatible with m1rl0k/Context-Engine and provide specialized search capabilities.

### `search_tests_for`

Search for test files related to a query using preset test file patterns.

**Input Schema:**
```json
{
  "query": "string (required) - Search query (function name, class name, or keyword)",
  "limit": "integer (optional) - Maximum results (default: 10, max: 50)"
}
```

**Preset Patterns:** `tests/**/*`, `test/**/*`, `**/*test*.*`, `**/*.spec.*`, `**/__tests__/**/*`

---

### `search_config_for`

Search for configuration files related to a query.

**Input Schema:**
```json
{
  "query": "string (required) - Search query (setting name, config key, or keyword)",
  "limit": "integer (optional) - Maximum results (default: 10, max: 50)"
}
```

**Preset Patterns:** `**/*.yaml`, `**/*.json`, `**/*.toml`, `**/*.ini`, `**/.env*`, `**/config/**/*`

---

### `search_callers_for`

Find all callers/usages of a symbol in the codebase.

**Input Schema:**
```json
{
  "symbol": "string (required) - The symbol name to find callers for",
  "file_pattern": "string (optional) - File pattern to limit search (e.g., '*.rs')",
  "limit": "integer (optional) - Maximum results (default: 20, max: 100)"
}
```

---

### `search_importers_for`

Find files that import a specific module or symbol.

**Input Schema:**
```json
{
  "module": "string (required) - The module or symbol name to find importers for",
  "file_pattern": "string (optional) - File pattern to limit search",
  "limit": "integer (optional) - Maximum results (default: 20, max: 100)"
}
```

---

### `info_request`

Simplified codebase retrieval with optional explanation mode.

**Input Schema:**
```json
{
  "query": "string (required) - Natural language query about the codebase",
  "explain": "boolean (optional) - Include relationship explanations (default: false)",
  "max_results": "integer (optional) - Maximum results (default: 10, max: 50)"
}
```

**Example:**
```json
{
  "query": "How does authentication work?",
  "explain": true,
  "max_results": 5
}
```

---

### `pattern_search`

Search for structural code patterns across the codebase.

**Input Schema:**
```json
{
  "pattern": "string (optional) - Custom regex pattern to search for",
  "pattern_type": "string (optional) - Preset pattern type: function, class, import, variable, custom",
  "language": "string (optional) - Filter by language (rust, python, typescript, go, java, kotlin)",
  "file_pattern": "string (optional) - File pattern to limit search",
  "limit": "integer (optional) - Maximum results (default: 20, max: 100)"
}
```

**Example:**
```json
{
  "pattern_type": "function",
  "language": "rust",
  "file_pattern": "*.rs",
  "limit": 10
}
```

---

### `context_search`

Context-aware semantic search with file context anchoring.

**Input Schema:**
```json
{
  "query": "string (required) - Natural language query",
  "context_file": "string (optional) - File path to use as context anchor",
  "include_related": "boolean (optional) - Include related files and symbols (default: true)",
  "max_tokens": "integer (optional) - Maximum tokens in response (default: 4000, max: 50000)"
}
```

**Example:**
```json
{
  "query": "error handling patterns",
  "context_file": "src/error.rs",
  "include_related": true,
  "max_tokens": 8000
}
```

---

## Error Handling

All tools return a `ToolResult` with:
- `isError: false` on success with content in `content[0].text`
- `isError: true` on failure with error message in `content[0].text`

## Transport Protocols

Context Engine supports two MCP transport protocols:

### stdio (default)
```bash
context-engine --workspace /path/to/project
```

### HTTP/SSE
```bash
context-engine --workspace /path/to/project --transport http --port 3000
```
