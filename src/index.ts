#!/usr/bin/env node

/**
 * Context Engine MCP Server
 * 
 * A local-first, agent-agnostic MCP server implementation
 * using Auggie SDK as the core context engine.
 * 
 * Architecture (5 layers):
 * 1. Core Context Engine (Auggie SDK) - indexing, retrieval
 * 2. Context Service Layer (serviceClient.ts) - orchestration
 * 3. MCP Interface Layer (server.ts, tools/) - protocol adapter
 * 4. Agent Clients (Claude, Cursor, etc.) - consumers
 * 5. Storage Backend (Auggie's internal) - vectors, metadata
 */

import { ContextEngineMCPServer } from './mcp/server.js';
import * as path from 'path';

async function main() {
  // Get workspace path from command line args or use current directory
  const args = process.argv.slice(2);
  let workspacePath = process.cwd();
  let shouldIndex = false;

  // Parse command line arguments
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    
    if (arg === '--workspace' || arg === '-w') {
      workspacePath = path.resolve(args[i + 1]);
      i++;
    } else if (arg === '--index' || arg === '-i') {
      shouldIndex = true;
    } else if (arg === '--help' || arg === '-h') {
      console.error(`
Context Engine MCP Server

Usage: context-engine-mcp [options]

Options:
  --workspace, -w <path>   Workspace directory to index (default: current directory)
  --index, -i              Index the workspace before starting server
  --help, -h               Show this help message

Environment Variables:
  AUGMENT_API_TOKEN        Auggie API token (or use 'auggie login')
  AUGMENT_API_URL          Auggie API URL (default: https://api.augmentcode.com)

Examples:
  # Start server with current directory
  context-engine-mcp

  # Start server with specific workspace
  context-engine-mcp --workspace /path/to/project

  # Index workspace before starting
  context-engine-mcp --workspace /path/to/project --index

MCP Configuration (for Codex CLI):
Add to ~/.codex/config.toml:

[mcp_servers.context-engine]
command = "node"
args = ["/absolute/path/to/dist/index.js", "--workspace", "/path/to/your/project"]

Or use the CLI:
codex mcp add context-engine -- node /absolute/path/to/dist/index.js --workspace /path/to/your/project
      `);
      process.exit(0);
    }
  }

  console.error('='.repeat(80));
  console.error('Context Engine MCP Server');
  console.error('='.repeat(80));
  console.error(`Workspace: ${workspacePath}`);
  console.error('');

  try {
    const server = new ContextEngineMCPServer(workspacePath);

    // Index workspace if requested
    if (shouldIndex) {
      console.error('Indexing workspace...');
      await server.indexWorkspace();
      console.error('Indexing complete!');
      console.error('');
    }

    // Start MCP server
    console.error('Starting MCP server...');
    await server.run();
  } catch (error) {
    console.error('Failed to start server:', error);
    process.exit(1);
  }
}

main().catch((error) => {
  console.error('Fatal error:', error);
  process.exit(1);
});

