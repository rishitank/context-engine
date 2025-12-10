# Troubleshooting Guide

Common issues and solutions for the Context Engine MCP Server.

## Installation Issues

### "Cannot find module '@modelcontextprotocol/sdk'"

**Cause**: Dependencies not installed

**Solution**:
```bash
npm install
```

### "auggie: command not found"

**Cause**: Auggie CLI not installed globally

**Solution**:
```bash
npm install -g @augmentcode/auggie

# Verify installation
auggie --version
```

### TypeScript compilation errors

**Cause**: TypeScript version mismatch or missing types

**Solution**:
```bash
# Clean and reinstall
rm -rf node_modules package-lock.json
npm install

# Rebuild
npm run build
```

## Authentication Issues

### "No API token found"

**Cause**: Not authenticated with Auggie

**Solution Option 1** (Recommended):
```bash
auggie login
```

**Solution Option 2** (Environment variables):
```bash
export AUGMENT_API_TOKEN="your-token-here"
export AUGMENT_API_URL="https://api.augmentcode.com"
```

**Solution Option 3** (Check session file):
```bash
# macOS/Linux
cat ~/.augment/session.json

# Windows
type %USERPROFILE%\.augment\session.json
```

### "Authentication failed" or "Invalid token"

**Cause**: Expired or invalid token

**Solution**:
```bash
# Re-authenticate
auggie logout
auggie login
```

## Server Issues

### Server not starting

**Symptom**: No output or immediate crash

**Debug Steps**:
```bash
# 1. Check if build succeeded
npm run build

# 2. Test with help command
node dist/index.js --help

# 3. Check for errors
node dist/index.js --workspace . 2>&1 | tee server.log
```

### "Failed to execute auggie CLI"

**Cause**: Auggie not in PATH or wrong permissions

**Solution**:
```bash
# Check if auggie is accessible
which auggie  # macOS/Linux
where auggie  # Windows

# If not found, reinstall
npm install -g @augmentcode/auggie

# Check permissions (macOS/Linux)
ls -la $(which auggie)
```

## Codex CLI Integration Issues

### Tools not showing up in Codex CLI

**Symptom**: MCP server not listed when running `/mcp` command

**Debug Steps**:

1. **Verify config file syntax**:
   ```bash
   # macOS/Linux
   cat ~/.codex/config.toml

   # Windows
   Get-Content "$env:USERPROFILE\.codex\config.toml"
   ```

2. **Check paths are absolute**:
   ```toml
   [mcp_servers.context-engine]
   command = "node"
   args = [
       "/absolute/path/to/context-engine/dist/index.js",
       "--workspace",
       "/absolute/path/to/your/project"
   ]
   ```

3. **Verify MCP configuration via CLI**:
   ```bash
   codex mcp list
   ```

4. **Restart Codex CLI**:
   - Exit and restart the codex command

5. **Test the server manually**:
   ```bash
   node /path/to/context-engine/dist/index.js --workspace /path/to/project
   ```

### "Server disconnected" or timeout errors

**Cause**: Server taking too long to respond or crashing

**Debug Steps**:

1. **Test server manually**:
   ```bash
   node dist/index.js --workspace /path/to/project
   ```

2. **Check for large workspace**:
   - Large codebases may need indexing first
   ```bash
   node dist/index.js --workspace /path/to/project --index
   ```

3. **Increase timeout** (if supported by client)

4. **Check system resources**:
   - Memory usage
   - CPU usage
   - Disk space

## Search and Indexing Issues

### "No results found" for valid queries

**Cause**: Workspace not indexed

**Solution**:
```bash
# Index the workspace
node dist/index.js --workspace /path/to/project --index

# Or use auggie CLI directly
auggie index /path/to/project
```

### Indexing is very slow

**Cause**: Large codebase or slow disk

**Solutions**:
- Use `.augmentignore` to exclude unnecessary files
- Exclude `node_modules`, `dist`, `build` directories
- Use SSD instead of HDD

**Create `.augmentignore`**:
```
node_modules/
dist/
build/
.git/
*.log
*.tmp
```

### Search returns irrelevant results

**Cause**: Query too vague or index needs updating

**Solutions**:
- Be more specific in queries
- Re-index after major code changes
- Use `get_context_for_prompt` instead of `semantic_search`

### "File not found" errors

**Cause**: File path incorrect or file deleted

**Debug**:
```bash
# Check if file exists
ls -la /path/to/file

# Use relative path from workspace root
# Instead of: /absolute/path/to/file
# Use: relative/path/from/workspace
```

## Performance Issues

### High memory usage

**Cause**: Large context bundles or many concurrent requests

**Solutions**:
- Reduce `max_files` parameter
- Reduce `top_k` parameter
- Index smaller portions of codebase

### Slow response times

**Cause**: Large codebase or complex queries

**Solutions**:
- Pre-index workspace
- Use more specific queries
- Reduce result limits

## Windows-Specific Issues

### Path issues with backslashes

**Solution**: Use forward slashes or double backslashes
```json
{
  "args": [
    "C:/path/to/project/dist/index.js",
    "--workspace",
    "C:/path/to/workspace"
  ]
}
```

### PowerShell execution policy

**Symptom**: Scripts won't run

**Solution**:
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

## macOS-Specific Issues

### "Operation not permitted" errors

**Cause**: macOS security restrictions

**Solution**:
- Grant Terminal/iTerm full disk access in System Preferences
- System Preferences → Security & Privacy → Privacy → Full Disk Access

## Getting Help

### Enable verbose logging

Modify `src/mcp/server.ts` to add debug output:

```typescript
console.error('DEBUG: Tool called:', name);
console.error('DEBUG: Arguments:', JSON.stringify(args, null, 2));
console.error('DEBUG: Result:', result);
```

### Collect diagnostic information

```bash
# System info
node --version
npm --version
auggie --version

# Check installation
npm list @modelcontextprotocol/sdk
npm list @augmentcode/auggie

# Test components
auggie search "test" --limit 1
node dist/index.js --help
```

### Use MCP Inspector

```bash
npm install -g @modelcontextprotocol/inspector
mcp-inspector node dist/index.js --workspace /path/to/project
```

This opens a web interface for interactive debugging.

## Still Having Issues?

1. Check the [TESTING.md](TESTING.md) guide
2. Review [ARCHITECTURE.md](ARCHITECTURE.md) for design details
3. Check `~/.codex/config.toml` for syntax errors
4. Test with MCP Inspector
5. Verify Auggie CLI works independently
6. Run `codex mcp list` to check configuration

## Common Error Messages

| Error | Likely Cause | Solution |
|-------|--------------|----------|
| `ENOENT: no such file` | File path incorrect | Use absolute paths or check workspace |
| `EACCES: permission denied` | File permissions | Check file/directory permissions |
| `spawn auggie ENOENT` | Auggie not in PATH | Reinstall auggie globally |
| `Invalid TOML` | Config file syntax | Validate TOML syntax in config.toml |
| `Connection refused` | Server not running | Start server first |
| `Timeout` | Server too slow | Index workspace, reduce limits |

