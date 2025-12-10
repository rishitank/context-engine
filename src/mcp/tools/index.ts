/**
 * Layer 3: MCP Interface Layer - Index Workspace Tool
 *
 * Allows triggering workspace indexing via MCP tool call.
 * This is essential for first-time setup or when files change significantly.
 */

import { ContextServiceClient } from '../serviceClient.js';

export interface IndexWorkspaceArgs {
  /** Force re-indexing even if index exists (default: false) */
  force?: boolean;
}

/**
 * Handle the index_workspace tool call
 */
export async function handleIndexWorkspace(
  args: IndexWorkspaceArgs,
  serviceClient: ContextServiceClient
): Promise<string> {
  const { force = false } = args;
  
  const startTime = Date.now();
  
  try {
    console.error(`[index_workspace] Starting workspace indexing (force=${force})...`);
    
    await serviceClient.indexWorkspace();
    
    const elapsed = Date.now() - startTime;
    
    return JSON.stringify({
      success: true,
      message: `Workspace indexed successfully in ${elapsed}ms`,
      elapsed_ms: elapsed,
    }, null, 2);
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.error(`[index_workspace] Failed: ${errorMessage}`);
    
    throw new Error(`Failed to index workspace: ${errorMessage}`);
  }
}

/**
 * Tool schema definition for MCP registration
 */
export const indexWorkspaceTool = {
  name: 'index_workspace',
  description: `Index the current workspace for semantic search.

This tool scans all source files in the workspace and builds a semantic index
that enables fast, meaning-based code search.

**When to use this tool:**
- First time using the context engine with a new project
- After making significant changes to the codebase
- When semantic_search or enhance_prompt returns no results

**What gets indexed:**
- TypeScript/JavaScript (.ts, .tsx, .js, .jsx, .mjs, .cjs)
- Python (.py)
- Go (.go)
- Rust (.rs)
- Java/Kotlin (.java, .kt)
- C/C++ (.c, .cpp, .h, .hpp)
- And many more...

**What is skipped:**
- node_modules, dist, build directories
- Hidden files/directories (starting with .)
- Binary files and files over 10MB

The index is saved to .augment-context-state.json in the workspace root
and will be automatically restored on future server starts.`,
  inputSchema: {
    type: 'object',
    properties: {
      force: {
        type: 'boolean',
        description: 'Force re-indexing even if an index already exists (default: false)',
        default: false,
      },
    },
    required: [],
  },
};

