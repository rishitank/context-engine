# 🚀 Get Started Checklist

Follow this checklist to get your Context Engine MCP Server up and running!

## Prerequisites Checklist

### System Requirements
- [ ] Node.js 18 or higher installed
  ```bash
  node --version  # Should show v18.x.x or higher
  ```

- [ ] npm installed
  ```bash
  npm --version
  ```

- [ ] Git installed (optional, for cloning)
  ```bash
  git --version
  ```

### Install Auggie CLI
- [ ] Install Auggie globally
  ```bash
  npm install -g @augmentcode/auggie
  ```

- [ ] Verify installation
  ```bash
  auggie --version
  ```

## Setup Checklist

### 1. Project Setup
- [ ] Navigate to project directory
  ```bash
  cd context-engine
  ```

- [ ] Install dependencies
  ```bash
  npm install
  ```

- [ ] Build the project
  ```bash
  npm run build
  ```

- [ ] Verify setup
  ```bash
  npm run verify
  ```

### 2. Authentication
Choose ONE of these methods:

**Option A: Auggie CLI (Recommended)**
- [ ] Login via CLI
  ```bash
  auggie login
  ```

**Option B: Environment Variables**
- [ ] Copy example file
  ```bash
  cp .env.example .env
  ```

- [ ] Edit `.env` and add your token
  ```
  AUGMENT_API_TOKEN=your-token-here
  AUGMENT_API_URL=https://api.augmentcode.com
  ```

### 3. Test the Server
- [ ] Test help command
  ```bash
  node dist/index.js --help
  ```

- [ ] Test with current directory
  ```bash
  node dist/index.js --workspace .
  ```
  Press Ctrl+C to stop

- [ ] Index a workspace (optional)
  ```bash
  node dist/index.js --workspace /path/to/your/project --index
  ```

## Codex CLI Integration Checklist

### 4. Install Codex CLI
- [ ] Install Codex CLI globally
  ```bash
  npm install -g @openai/codex
  ```

- [ ] Verify installation
  ```bash
  codex --version
  ```

### 5. Configure Codex CLI

**Option A: Using the CLI (Recommended)**
- [ ] Add MCP server via CLI
  ```bash
  codex mcp add context-engine -- node /ABSOLUTE/PATH/TO/context-engine/dist/index.js --workspace /PATH/TO/YOUR/PROJECT
  ```

**Option B: Edit config.toml Directly**

**macOS/Linux:**
- [ ] Open config file
  ```bash
  mkdir -p ~/.codex
  code ~/.codex/config.toml
  ```

**Windows:**
- [ ] Open config file
  ```powershell
  mkdir -Force $env:USERPROFILE\.codex
  code $env:USERPROFILE\.codex\config.toml
  ```

- [ ] Add this configuration (replace paths):
  ```toml
  [mcp_servers.context-engine]
  command = "node"
  args = [
      "/ABSOLUTE/PATH/TO/context-engine/dist/index.js",
      "--workspace",
      "/PATH/TO/YOUR/PROJECT"
  ]
  ```

- [ ] Save the file

### 6. Restart Codex CLI
- [ ] Exit Codex CLI if running
- [ ] Relaunch with `codex`

### 7. Verify Connection
- [ ] In Codex CLI TUI, type `/mcp`
- [ ] Verify available tools:
  - [ ] `semantic_search`
  - [ ] `get_file`
  - [ ] `get_context_for_prompt`

## First Usage Checklist

### 8. Try Example Queries

- [ ] **Test 1**: Basic search
  ```
  "Search for authentication logic in the codebase"
  ```

- [ ] **Test 2**: Get file
  ```
  "Show me the package.json file"
  ```

- [ ] **Test 3**: Get context
  ```
  "Get context about the database schema"
  ```

### 9. Verify Results
- [ ] Results are relevant
- [ ] No error messages
- [ ] Tools are being called correctly

## Troubleshooting Checklist

If something doesn't work:

- [ ] Check `npm run verify` output
- [ ] Review error messages
- [ ] Verify MCP configuration
  ```bash
  # Check config file
  cat ~/.codex/config.toml

  # List configured MCP servers
  codex mcp list
  ```

- [ ] Verify authentication
  ```bash
  auggie search "test" --limit 1
  ```

- [ ] Check [TROUBLESHOOTING.md](TROUBLESHOOTING.md)

## Next Steps Checklist

### Learn More
- [ ] Read [EXAMPLES.md](EXAMPLES.md) for usage examples
- [ ] Review [ARCHITECTURE.md](ARCHITECTURE.md) to understand design
- [ ] Check [TESTING.md](TESTING.md) for testing strategies

### Customize
- [ ] Index your actual project
- [ ] Adjust tool parameters
- [ ] Add custom tools (see ARCHITECTURE.md)

### Share
- [ ] Share with your team
- [ ] Document your use cases
- [ ] Contribute improvements

## Quick Reference

### Common Commands
```bash
# Verify setup
npm run verify

# Build project
npm run build

# Test server
node dist/index.js --help

# Index workspace
node dist/index.js --workspace /path/to/project --index

# Debug with inspector
npm run inspector
```

### Documentation Quick Links
- **Setup**: [QUICKSTART.md](QUICKSTART.md)
- **Usage**: [EXAMPLES.md](EXAMPLES.md)
- **Problems**: [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
- **Architecture**: [ARCHITECTURE.md](ARCHITECTURE.md)
- **All Docs**: [INDEX.md](INDEX.md)

## Success Criteria

You're ready when:
- ✅ `npm run verify` passes all checks
- ✅ Server starts without errors
- ✅ Tools appear in Codex CLI (`/mcp` command)
- ✅ Example queries return results
- ✅ No error messages in logs

---

## 🎉 Congratulations!

If you've completed this checklist, you're all set!

**Need help?** Check [TROUBLESHOOTING.md](TROUBLESHOOTING.md) or [INDEX.md](INDEX.md)

**Ready to learn more?** See [EXAMPLES.md](EXAMPLES.md) for real-world usage patterns

