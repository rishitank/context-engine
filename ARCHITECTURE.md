# Architecture Documentation

Detailed architecture documentation for the Context Engine MCP Server.

## Overview

This project implements a **local-first, agent-agnostic context engine** using the Model Context Protocol (MCP) as the interface layer and Auggie SDK as the core engine.

## 5-Layer Architecture

### Layer 1: Core Context Engine (Auggie SDK)

**Location**: External dependency (`@augmentcode/auggie`)

**Purpose**: The brain of the system. Handles all low-level context operations.

**Responsibilities**:
- File ingestion and scanning
- Code chunking with language awareness
- Embedding generation
- Semantic retrieval via vector search
- Metadata management

**What it does NOT do**:
- ❌ Serve HTTP
- ❌ Know about prompts or agents
- ❌ Generate LLM answers

**Interface**: CLI commands (`auggie index`, `auggie search`)

### Layer 2: Context Service Layer

**Location**: `src/mcp/serviceClient.ts`

**Purpose**: Adapts raw retrieval into agent-friendly context.

**Responsibilities**:
- Decide how much context to return
- Format snippets for readability
- Deduplicate results by file
- Enforce limits (max files, max tokens)
- Apply heuristics (importance, recency)

**What it does NOT do**:
- ❌ Index files
- ❌ Store vectors
- ❌ Talk to agents directly

**Key Methods**:
```typescript
semanticSearch(query, topK): SearchResult[]
getFile(path): string
getContextForPrompt(query, maxFiles): ContextBundle
```

**Context Bundle Format**:
```typescript
{
  summary: string;
  files: Array<{
    path: string;
    snippets: Array<{
      text: string;
      lines: string;
    }>;
  }>;
  hints: string[];
}
```

### Layer 3: MCP Interface Layer

**Location**: `src/mcp/server.ts`, `src/mcp/tools/`

**Purpose**: Protocol adapter that lets agents communicate with the service layer.

**Responsibilities**:
- Expose tools via MCP protocol
- Validate input/output
- Map tool calls to service layer methods
- Stay stateless

**What it does NOT do**:
- ❌ Business logic
- ❌ Retrieval logic
- ❌ Formatting decisions

**Tools Exposed**:

1. **semantic_search**
   - Input: `{ query: string, top_k?: number }`
   - Output: Formatted search results
   - Use case: Find specific code patterns

2. **get_file**
   - Input: `{ path: string }`
   - Output: Complete file contents
   - Use case: Retrieve full file after search

3. **get_context_for_prompt**
   - Input: `{ query: string, max_files?: number }`
   - Output: Rich context bundle
   - Use case: Primary tool for prompt enhancement

### Layer 4: Agent Clients

**Location**: External (Codex CLI, Cursor, etc.)

**Purpose**: Consume context and generate responses.

**Agent Responsibilities**:
- Decide when to call tools
- Decide how to use context
- Generate final answers

**What the system does NOT do**:
- ❌ Generate answers
- ❌ Make decisions for agents
- ❌ Interpret results

### Layer 5: Storage Backend

**Location**: Auggie SDK internal

**Purpose**: Persist embeddings and metadata.

**Responsibilities**:
- Store vector embeddings
- Store file metadata
- Support fast vector similarity search

**Storage Options** (handled by Auggie):
- Qdrant (recommended)
- SQLite (simple)
- Hybrid (future)

## Data Flow

### Indexing Flow

```
File System
    ↓
Scanner (Layer 1)
    ↓
Chunker (Layer 1)
    ↓
Embedder (Layer 1)
    ↓
Vector Store (Layer 5)
```

### Prompt Enhancement Flow

```
Agent Prompt
    ↓
MCP Tool Call (Layer 3)
    ↓
Context Service (Layer 2)
    ↓
Engine Retrieval (Layer 1)
    ↓
Context Bundle (Layer 2)
    ↓
Agent Final Prompt (Layer 4)
```

## Design Principles

### 1. Separation of Concerns

Each layer has **one responsibility only**. Never collapse layers.

### 2. Clean Contracts

Interfaces between layers are well-defined and stable:
- Layer 1 ↔ Layer 2: CLI commands and JSON output
- Layer 2 ↔ Layer 3: TypeScript interfaces
- Layer 3 ↔ Layer 4: MCP protocol

### 3. Stateless MCP Layer

Layer 3 maintains no state. Each tool call is independent.

### 4. Agent-Agnostic

No LLM-specific logic anywhere in the stack. Works with any MCP client.

### 5. Local-First

- No cloud dependencies
- No exposed network ports
- No data leaves the machine
- All processing happens locally

## Extension Points

### Adding New Tools

1. Create tool handler in `src/mcp/tools/`
2. Define input schema
3. Implement handler function
4. Register in `src/mcp/server.ts`

Example:
```typescript
// src/mcp/tools/myTool.ts
export const myTool = {
  name: 'my_tool',
  description: 'Does something useful',
  inputSchema: { /* ... */ }
};

export async function handleMyTool(args, serviceClient) {
  // Implementation
}
```

### Adding Service Methods

Add methods to `ContextServiceClient` in `src/mcp/serviceClient.ts`:

```typescript
async myServiceMethod(params): Promise<Result> {
  // Call Auggie CLI or process data
  // Apply Layer 2 logic (formatting, deduplication, etc.)
  return result;
}
```

### Future Enhancements

These can be added **without architectural changes**:

- File watchers (Layer 1)
- Incremental indexing (Layer 1)
- Multi-repo support (Layer 2)
- Role-based filtering (Layer 2)
- Hybrid search (Layer 1)
- Caching (Layer 2)
- Custom context strategies (Layer 2)

## Security Considerations

### Authentication

- Uses Auggie CLI session or environment variables
- No credentials stored in code
- Session file: `~/.augment/session.json`

### Data Privacy

- All data stays local
- No network calls except to Auggie API (for embeddings)
- No telemetry or tracking

### Input Validation

- All tool inputs validated in Layer 3
- Path traversal prevention in file access
- Query sanitization before CLI execution

## Performance Considerations

### Indexing

- Initial indexing can be slow for large codebases
- Incremental updates are faster
- Respects `.gitignore` and `.augmentignore`

### Search

- Vector search is fast (< 100ms typically)
- Results limited by `top_k` parameter
- Deduplication adds minimal overhead

### Context Bundling

- Limited by `max_files` parameter
- Snippet extraction is fast
- Formatting is lightweight

## Monitoring and Debugging

### Logs

- Server logs to stderr
- Codex CLI: Check `~/.codex/config.toml` and use `codex mcp list`
- Auggie CLI logs: Check auggie documentation

### Debugging Tools

- MCP Inspector for interactive testing
- Direct CLI testing with auggie
- TypeScript source maps for stack traces

## Testing Strategy

See [TESTING.md](TESTING.md) for comprehensive testing guide.

## References

- [plan.md](plan.md) - Original architecture plan
- [MCP Documentation](https://modelcontextprotocol.io/)
- [Auggie SDK](https://docs.augmentcode.com/)

