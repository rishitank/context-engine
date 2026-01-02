# API Reference

Complete reference for all 49 MCP tools provided by Context Engine.

## Table of Contents

- [Retrieval Tools](#retrieval-tools-6)
- [Index Tools](#index-tools-5)
- [Memory Tools](#memory-tools-4)
- [Planning Tools](#planning-tools-20)
- [Review Tools](#review-tools-14)

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

Transform a simple prompt into a detailed, structured prompt with codebase context.

**Input Schema:**
```json
{
  "prompt": "string (required) - The simple prompt to enhance (max 10000 chars)"
}
```

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

## Memory Tools (4)

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

