# Usage Examples

Real-world examples of using the Context Engine MCP Server with Codex CLI and other MCP clients.

## Basic Usage

### Example 1: Finding Authentication Code

**User Query:**
> "Search for authentication logic in the codebase"

**What Happens:**
1. Codex calls `semantic_search` tool
2. Query: "authentication logic"
3. Returns relevant files with auth code

**Expected Response:**
```
Found 5 results for: "authentication logic"

1. src/auth/login.ts
   Lines: 15-45
   Relevance: 92.3%
   export async function authenticateUser(credentials) {
     const user = await validateCredentials(credentials);
     ...

2. src/middleware/auth.ts
   Lines: 8-30
   Relevance: 88.7%
   export function requireAuth(req, res, next) {
     const token = req.headers.authorization;
     ...
```

### Example 2: Getting Complete File

**User Query:**
> "Show me the package.json file"

**What Happens:**
1. Codex calls `get_file` tool
2. Path: "package.json"
3. Returns complete file contents

**Expected Response:**
```
File: package.json
Lines: 35
Size: 1247 bytes

================================================================================

{
  "name": "my-project",
  "version": "1.0.0",
  ...
}
```

### Example 3: Context for Feature Development

**User Query:**
> "I need to add a new API endpoint for user profiles. Get me relevant context."

**What Happens:**
1. Codex calls `get_context_for_prompt` tool
2. Query: "API endpoint user profiles"
3. Returns comprehensive context bundle

**Expected Response:**
```markdown
# Context for: "API endpoint user profiles"

## Key Insights
- Found code in: .ts, .js
- 5 relevant files found

## Relevant Code

### 1. src/routes/api.ts

Lines: 10-50

```typescript
export const apiRouter = express.Router();

apiRouter.get('/users/:id', async (req, res) => {
  const user = await getUserById(req.params.id);
  res.json(user);
});
```

### 2. src/controllers/userController.ts

Lines: 5-30

```typescript
export async function getUserProfile(userId: string) {
  const profile = await db.users.findOne({ id: userId });
  return formatUserProfile(profile);
}
```
```

## Advanced Usage

### Example 4: Code Review Assistance

**User Query:**
> "Review the error handling patterns in this codebase"

**Codex's Workflow:**
1. Calls `semantic_search` with query "error handling try catch"
2. Analyzes results
3. Calls `get_file` for specific files
4. Provides comprehensive review

**User Benefits:**
- Finds all error handling code
- Identifies inconsistencies
- Suggests improvements

### Example 5: Bug Investigation

**User Query:**
> "There's a bug in the payment processing. Help me find the relevant code."

**Codex's Workflow:**
1. Calls `get_context_for_prompt` with "payment processing"
2. Reviews context bundle
3. Asks clarifying questions
4. Calls `get_file` for specific files
5. Helps debug

### Example 6: Learning Codebase

**User Query:**
> "I'm new to this codebase. Explain how the database layer works."

**Codex's Workflow:**
1. Calls `get_context_for_prompt` with "database layer schema models"
2. Analyzes structure
3. Calls `semantic_search` for specific patterns
4. Provides explanation with code examples

## Tool-Specific Examples

### semantic_search Tool

**Use Cases:**
- Finding specific patterns
- Locating similar code
- Discovering implementations

**Example Queries:**
```json
{
  "name": "semantic_search",
  "arguments": {
    "query": "database connection pool",
    "top_k": 5
  }
}
```

```json
{
  "name": "semantic_search",
  "arguments": {
    "query": "error handling middleware",
    "top_k": 10
  }
}
```

### get_file Tool

**Use Cases:**
- Reading configuration files
- Reviewing specific implementations
- Getting complete context

**Example Queries:**
```json
{
  "name": "get_file",
  "arguments": {
    "path": "src/config/database.ts"
  }
}
```

```json
{
  "name": "get_file",
  "arguments": {
    "path": "README.md"
  }
}
```

### get_context_for_prompt Tool

**Use Cases:**
- Feature development
- Understanding subsystems
- Comprehensive code review

**Example Queries:**
```json
{
  "name": "get_context_for_prompt",
  "arguments": {
    "query": "authentication and authorization system",
    "max_files": 5
  }
}
```

```json
{
  "name": "get_context_for_prompt",
  "arguments": {
    "query": "API rate limiting implementation",
    "max_files": 3
  }
}
```

## Best Practices

### 1. Be Specific in Queries

❌ **Bad:** "Show me code"
✅ **Good:** "Show me the authentication middleware code"

❌ **Bad:** "Find functions"
✅ **Good:** "Find database query functions in the user service"

### 2. Use the Right Tool

- **semantic_search**: When you know what you're looking for
- **get_file**: When you know the exact file
- **get_context_for_prompt**: When you need comprehensive context

### 3. Iterate

Start broad, then narrow down:
1. `get_context_for_prompt` - Get overview
2. `semantic_search` - Find specific patterns
3. `get_file` - Read complete files

### 4. Adjust Parameters

**For broad exploration:**
```json
{
  "query": "authentication",
  "max_files": 10
}
```

**For focused investigation:**
```json
{
  "query": "JWT token validation in auth middleware",
  "max_files": 3
}
```

## Common Workflows

### Workflow 1: Adding a New Feature

1. **Understand existing patterns**
   - Query: "Get context for similar feature X"
   
2. **Find implementation examples**
   - Query: "Search for pattern Y"
   
3. **Review specific files**
   - Query: "Show me file Z"

### Workflow 2: Debugging

1. **Locate problem area**
   - Query: "Get context for module with bug"
   
2. **Find related code**
   - Query: "Search for error handling in module"
   
3. **Review implementation**
   - Query: "Show me the specific file"

### Workflow 3: Code Review

1. **Get overview**
   - Query: "Get context for the PR changes"
   
2. **Check patterns**
   - Query: "Search for similar implementations"
   
3. **Verify specific files**
   - Query: "Show me each changed file"

## Tips for Better Results

1. **Index your workspace first**
   ```bash
   node dist/index.js --workspace /path/to/project --index
   ```

2. **Use domain-specific terms**
   - Instead of "code that handles users"
   - Use "user authentication service"

3. **Combine tools**
   - Start with `get_context_for_prompt`
   - Follow up with `semantic_search`
   - Finish with `get_file`

4. **Re-index after major changes**
   ```bash
   auggie index /path/to/project
   ```

## Troubleshooting Examples

### No Results Found

**Problem:** Query returns no results

**Solutions:**
- Make query more general
- Check if workspace is indexed
- Verify file types are supported

### Too Many Results

**Problem:** Query returns too many irrelevant results

**Solutions:**
- Make query more specific
- Reduce `top_k` or `max_files`
- Use more technical terms

### Wrong Files Returned

**Problem:** Results don't match expectation

**Solutions:**
- Rephrase query with different terms
- Use `get_context_for_prompt` instead of `semantic_search`
- Check if files are actually indexed

---

For more examples and use cases, see the [TESTING.md](TESTING.md) guide.

