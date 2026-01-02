# Troubleshooting Guide

Common issues and solutions for Context Engine.

## Table of Contents

- [Installation Issues](#installation-issues)
- [Connection Issues](#connection-issues)
- [Search and Indexing](#search-and-indexing)
- [Performance Issues](#performance-issues)
- [Docker Issues](#docker-issues)
- [MCP Client Configuration](#mcp-client-configuration)

---

## Installation Issues

### Binary Not Found

**Symptom:** `command not found: context-engine`

**Solutions:**

1. **Check installation path:**
   ```bash
   # If installed via cargo
   ls ~/.cargo/bin/context-engine
   
   # Add to PATH if needed
   export PATH="$HOME/.cargo/bin:$PATH"
   ```

2. **Verify binary exists:**
   ```bash
   which context-engine
   ```

3. **For downloaded binaries:**
   ```bash
   chmod +x context-engine-darwin-arm64
   sudo mv context-engine-darwin-arm64 /usr/local/bin/context-engine
   ```

### Permission Denied

**Symptom:** `Permission denied` when running the binary

**Solution:**
```bash
chmod +x /path/to/context-engine
```

### macOS Gatekeeper Block

**Symptom:** "context-engine cannot be opened because it is from an unidentified developer"

**Solution:**
```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine /path/to/context-engine

# Or allow in System Preferences > Security & Privacy
```

---

## Connection Issues

### MCP Client Can't Connect

**Symptom:** MCP client shows "Failed to connect" or "Server not responding"

**Solutions:**

1. **Verify server is running:**
   ```bash
   context-engine --workspace /path/to/project --version
   ```

2. **Check workspace path exists:**
   ```bash
   ls -la /path/to/project
   ```

3. **Test stdio transport manually:**
   ```bash
   echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}' | context-engine --workspace .
   ```

4. **For HTTP transport, verify port is available:**
   ```bash
   lsof -i :3000
   ```

### Connection Timeout

**Symptom:** Client times out waiting for response

**Solutions:**

1. **Increase client timeout** in your MCP client configuration

2. **Check if indexing is in progress:**
   ```json
   // Tool: index_status
   {}
   ```

3. **Use background indexing for large codebases:**
   ```json
   // Tool: index_workspace
   { "background": true }
   ```

### HTTP Transport Not Working

**Symptom:** Can't connect to HTTP endpoint

**Solutions:**

1. **Verify correct transport flag:**
   ```bash
   context-engine --workspace . --transport http --port 3000
   ```

2. **Check SSE endpoint:**
   ```bash
   curl http://localhost:3000/sse
   ```

3. **Verify no firewall blocking:**
   ```bash
   # macOS
   sudo pfctl -s rules | grep 3000
   ```

---

## Search and Indexing

### No Search Results

**Symptom:** `codebase_retrieval` or `semantic_search` returns empty results

**Solutions:**

1. **Index the workspace first:**
   ```json
   // Tool: index_workspace
   { "force": true }
   ```

2. **Check index status:**
   ```json
   // Tool: index_status
   {}
   ```

3. **Verify files are being indexed:**
   - Check file extensions are supported (see API_REFERENCE.md)
   - Ensure files aren't in `.gitignore` patterns

### Indexing Fails

**Symptom:** `index_workspace` returns an error

**Solutions:**

1. **Check workspace permissions:**
   ```bash
   ls -la /path/to/workspace
   ```

2. **Verify disk space:**
   ```bash
   df -h
   ```

3. **Clear and retry:**
   ```json
   // Tool: clear_index
   {}
   
   // Then:
   // Tool: index_workspace
   { "force": true }
   ```

### Stale Search Results

**Symptom:** Search returns outdated code

**Solution:**
```json
// Tool: reindex_workspace
{}
```

---

## Performance Issues

### Slow Indexing

**Symptom:** Indexing takes too long

**Solutions:**

1. **Use background indexing:**
   ```json
   // Tool: index_workspace
   { "background": true }
   ```

2. **Check for large files:**
   - Binary files are skipped automatically
   - Very large text files may slow indexing

3. **Exclude unnecessary directories:**
   - Ensure `node_modules`, `target`, `.git` are in `.gitignore`

### High Memory Usage

**Symptom:** Context Engine uses excessive memory

**Solutions:**

1. **Reduce token budgets:**
   ```json
   // Tool: get_context_for_prompt
   { "query": "...", "token_budget": 4000 }
   ```

2. **Limit search results:**
   ```json
   // Tool: semantic_search
   { "query": "...", "max_results": 5 }
   ```

3. **Clear index periodically:**
   ```json
   // Tool: clear_index
   {}
   ```

### Slow Response Times

**Symptom:** Tools take too long to respond

**Solutions:**

1. **Check if re-indexing is needed:**
   ```json
   // Tool: index_status
   {}
   ```

2. **Reduce max_tokens/token_budget parameters**

3. **Use more specific queries** to reduce search scope

---

## Docker Issues

### Container Won't Start

**Symptom:** Docker container exits immediately

**Solutions:**

1. **Check logs:**
   ```bash
   docker logs <container_id>
   ```

2. **Verify volume mount:**
   ```bash
   docker run -v /path/to/project:/workspace context-engine --workspace /workspace
   ```

3. **Check image exists:**
   ```bash
   docker images | grep context-engine
   ```

### Volume Mount Issues

**Symptom:** "Workspace not found" or empty search results in Docker

**Solutions:**

1. **Use absolute paths:**
   ```bash
   docker run -v $(pwd):/workspace context-engine --workspace /workspace
   ```

2. **Check Docker Desktop file sharing settings** (macOS/Windows)

3. **Verify mount inside container:**
   ```bash
   docker run -it -v $(pwd):/workspace context-engine ls /workspace
   ```

### Image Size Concerns

**Current image size:** ~20 MB (Alpine-based)

If you need a smaller image:
1. Use the pre-built binary directly instead of Docker
2. Binary sizes: 5.6 MB (macOS ARM64), 6.5 MB (macOS x86_64), 7.3 MB (Linux x86_64)

---

## MCP Client Configuration

### Claude Desktop Issues

**Config location:** `~/Library/Application Support/Claude/claude_desktop_config.json`

**Common issues:**

1. **Invalid JSON:**
   ```bash
   # Validate JSON
   cat ~/Library/Application\ Support/Claude/claude_desktop_config.json | jq .
   ```

2. **Wrong path to binary:**
   ```json
   {
     "mcpServers": {
       "context-engine": {
         "command": "/absolute/path/to/context-engine",
         "args": ["--workspace", "/absolute/path/to/project"]
       }
     }
   }
   ```

3. **Restart Claude Desktop** after config changes

### VS Code Continue Issues

**Config location:** `.continue/config.json` in your project

**Common issues:**

1. **Use `${workspaceFolder}` variable:**
   ```json
   {
     "mcpServers": [
       {
         "name": "context-engine",
         "command": "context-engine",
         "args": ["--workspace", "${workspaceFolder}"]
       }
     ]
   }
   ```

2. **Ensure binary is in PATH**

### Cursor Issues

**Config location:** Cursor settings

**Solutions:**

1. **Use absolute paths** for both command and workspace
2. **Check Cursor's MCP logs** for connection errors

---

## Debugging

### Enable Verbose Logging

```bash
RUST_LOG=debug context-engine --workspace .
```

### Check Server Version

```bash
context-engine --version
```

### Test Tool Execution

Send a test request via stdio:

```bash
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"index_status","arguments":{}},"id":1}' | context-engine --workspace .
```

### View Available Tools

```bash
echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":1}' | context-engine --workspace .
```

---

## Getting Help

If you're still experiencing issues:

1. **Check the README** for the latest documentation
2. **Review API_REFERENCE.md** for correct tool usage
3. **Check EXAMPLES.md** for working examples
4. **Open an issue** on GitHub with:
   - Context Engine version (`context-engine --version`)
   - Operating system and architecture
   - MCP client being used
   - Error messages and logs
   - Steps to reproduce

---

## Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| `Workspace not found` | Invalid workspace path | Use absolute path, verify directory exists |
| `Index not initialized` | Workspace not indexed | Run `index_workspace` first |
| `Failed to read file` | File permissions or path issue | Check file exists and is readable |
| `Search failed` | Index corruption or missing | Run `reindex_workspace` |
| `Connection refused` | Server not running or wrong port | Verify server is running, check port |
| `Invalid JSON-RPC` | Malformed request | Check request format matches MCP spec |

