# Quick Start Guide

Get your Context Engine MCP Server running in 5 minutes!

## Step 1: Install Prerequisites

```bash
# Install Auggie CLI globally
npm install -g @augmentcode/auggie

# Verify installation
auggie --version
```

## Step 2: Authenticate

```bash
# Login to Auggie (creates ~/.augment/session.json)
auggie login

# Or set environment variables
export AUGMENT_API_TOKEN="your-token"
export AUGMENT_API_URL="https://api.augmentcode.com"
```

## Step 3: Build the Server

```bash
# Navigate to the project directory
cd context-engine

# Install dependencies
npm install

# Build the TypeScript code
npm run build
```

## Step 4: Test the Server

```bash
# Test with current directory
node dist/index.js --help

# Index and start server with a specific project
node dist/index.js --workspace /path/to/your/project --index
```

## Step 5: Configure Codex CLI

Codex CLI uses TOML configuration stored in `~/.codex/config.toml`.

### Option A: Using the CLI (Recommended)

```bash
# Add the MCP server using codex mcp add
codex mcp add context-engine -- node /absolute/path/to/context-engine/dist/index.js --workspace /path/to/your/project
```

**Important**: Replace paths with your actual paths:
- Get context-engine path by running `pwd` in this directory
- Replace workspace path with your project directory

### Option B: Editing config.toml Directly

1. Open or create the config file:

   **macOS/Linux:**
   ```bash
   mkdir -p ~/.codex
   code ~/.codex/config.toml
   ```

   **Windows:**
   ```powershell
   mkdir -Force $env:USERPROFILE\.codex
   code $env:USERPROFILE\.codex\config.toml
   ```

2. Add the MCP server configuration:

   **macOS/Linux:**
   ```toml
   [mcp_servers.context-engine]
   command = "node"
   args = [
       "/absolute/path/to/context-engine/dist/index.js",
       "--workspace",
       "/path/to/your/project"
   ]
   ```

   **Windows:**
   ```toml
   [mcp_servers.context-engine]
   command = "node"
   args = [
       "D:\\GitProjects\\context-engine\\dist\\index.js",
       "--workspace",
       "D:\\GitProjects\\your-project"
   ]
   ```

## Step 6: Restart Codex CLI

If Codex is running, exit and restart it:

```bash
# Start Codex CLI fresh
codex
```

## Step 7: Verify Connection

1. Launch Codex CLI: `codex`
2. In the TUI, type `/mcp` to see connected MCP servers
3. You should see `context-engine` listed with available tools:
   - `semantic_search`
   - `get_file`
   - `get_context_for_prompt`

## Step 8: Try It Out!

Ask Codex:

- "Search for authentication logic in the codebase"
- "Get context about the database schema"
- "Show me the main entry point file"
- "Find error handling patterns"

## Troubleshooting

### Tools not showing up in /mcp

```bash
# Verify your config.toml is correct
cat ~/.codex/config.toml

# Verify server builds correctly
npm run build

# Test server manually
node dist/index.js --workspace . --index

# Check for configuration errors
codex mcp list
```

### Authentication errors

```bash
# Re-authenticate
auggie login

# Or check environment variables
echo $AUGMENT_API_TOKEN
```

### No search results

```bash
# Index your workspace first
node dist/index.js --workspace /path/to/project --index

# Or use auggie CLI directly
auggie index /path/to/project
```

## Next Steps

- Read [README.md](README.md) for detailed documentation
- Review [plan.md](plan.md) for architecture details
- Explore the source code in `src/mcp/`

## Common Use Cases

### 1. Code Understanding
"Get context about how authentication works in this codebase"

### 2. Bug Investigation
"Search for error handling in the payment processing module"

### 3. Feature Development
"Show me examples of API endpoint implementations"

### 4. Code Review
"Find all database query patterns in the codebase"

## Tips

1. **Be specific** in your queries for better results
2. **Use get_context_for_prompt** for comprehensive context
3. **Index regularly** if your codebase changes frequently
4. **Check logs** if something doesn't work

Happy coding! 🚀

