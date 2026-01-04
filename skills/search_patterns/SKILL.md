---
name: search_patterns
description: Specialized search patterns for finding tests, configs, callers, importers, and performing context-aware semantic search.
category: search
tags:
  - search
  - patterns
  - tests
  - config
  - semantic
always_apply: false
---

# Search Patterns Skill

Use this skill for specialized code search with preset patterns and semantic understanding.

## When to Use

- Finding test files for a specific feature
- Locating configuration files
- Tracing function callers and usages
- Finding import dependencies
- Semantic code search with context

## Available Search Patterns

### 1. Search Tests

Find test files related to a query:

```
search_tests_for(
  query: "authentication",
  limit: 10
)
```

Searches in patterns:
- `tests/**/*`, `test/**/*`
- `**/*test*.*`, `**/*_test.*`, `**/*Test*.*`
- `**/*.test.*`, `**/*.spec.*`
- `**/test_*.*`, `**/__tests__/**/*`

### 2. Search Config

Find configuration files:

```
search_config_for(
  query: "database",
  limit: 10
)
```

Searches in patterns:
- `**/*.yaml`, `**/*.yml`, `**/*.json`, `**/*.toml`
- `**/*.ini`, `**/*.cfg`, `**/*.conf`
- `**/.env*`, `**/config/**/*`
- `**/*config*.*`, `**/*settings*.*`

### 3. Search Callers

Find all callers of a function or method:

```
search_callers_for(
  symbol: "authenticate_user",
  limit: 20
)
```

Returns files and line numbers where the symbol is called.

### 4. Search Importers

Find files that import a module:

```
search_importers_for(
  module: "auth/jwt",
  limit: 20
)
```

Detects import patterns for:
- Rust: `use`, `mod`
- Python: `import`, `from ... import`
- JavaScript/TypeScript: `import`, `require`
- Go: `import`

### 5. Pattern Search

Structural code pattern matching:

```
pattern_search(
  pattern: "fn $name($args) -> Result<$ret, $err>",
  language: "rust",
  limit: 20
)
```

Supports pattern variables:
- `$name` - matches any identifier
- `$args` - matches argument list
- `$body` - matches block body
- `$_` - matches anything (wildcard)

### 6. Context Search

Semantic search with context awareness:

```
context_search(
  query: "How is user authentication implemented?",
  context: "Looking for JWT token validation",
  max_tokens: 4000
)
```

Returns semantically relevant code with explanations.

### 7. Info Request

Simplified codebase retrieval:

```
info_request(
  question: "Where is the database connection configured?",
  explain: true
)
```

## Available Tools

| Tool | Purpose |
|------|---------|
| `search_tests_for` | Find test files matching query |
| `search_config_for` | Find config files matching query |
| `search_callers_for` | Find callers of a symbol |
| `search_importers_for` | Find files importing a module |
| `pattern_search` | Structural pattern matching |
| `context_search` | Semantic search with context |
| `info_request` | Simplified codebase Q&A |

## Example Workflows

### Finding Related Tests

```
# 1. Find tests for a feature
tests = search_tests_for(query: "user_registration")

# 2. Review test coverage
for test in tests:
  review_tests(files: [test.path])
```

### Tracing Dependencies

```
# 1. Find who calls a function
callers = search_callers_for(symbol: "validate_token")

# 2. Find who imports the module
importers = search_importers_for(module: "auth/validation")

# 3. Understand the dependency graph
```

### Finding Configuration

```
# 1. Find database config
db_config = search_config_for(query: "postgres")

# 2. Find environment variables
env_config = search_config_for(query: "DATABASE_URL")
```

### Semantic Understanding

```
# 1. Ask a question about the codebase
info_request(
  question: "How does the caching layer work?",
  explain: true
)

# 2. Search with context
context_search(
  query: "cache invalidation",
  context: "Looking for TTL-based expiration logic"
)
```

## Best Practices

1. **Start specific**: Use exact symbol names when possible
2. **Combine searches**: Use multiple patterns to triangulate
3. **Use context**: Provide context for semantic searches
4. **Limit results**: Start with small limits, increase if needed
5. **Verify findings**: Cross-reference with `find_references` for accuracy

