/**
 * Layer 3: MCP Interface Layer - Server
 *
 * Main MCP server that exposes tools to coding agents
 *
 * Architecture:
 * - Stateless adapter between MCP protocol and service layer
 * - No business logic
 * - No retrieval logic
 * - Pure protocol translation
 *
 * Features:
 * - Graceful shutdown handling (SIGTERM, SIGINT)
 * - Request logging for debugging
 * - Proper error formatting for agents
 */

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';

import { ContextServiceClient } from './serviceClient.js';
import { semanticSearchTool, handleSemanticSearch } from './tools/search.js';
import { getFileTool, handleGetFile } from './tools/file.js';
import { getContextTool, handleGetContext } from './tools/context.js';
import { enhancePromptTool, handleEnhancePrompt } from './tools/enhance.js';
import { indexWorkspaceTool, handleIndexWorkspace } from './tools/index.js';

export class ContextEngineMCPServer {
  private server: Server;
  private serviceClient: ContextServiceClient;
  private isShuttingDown = false;
  private workspacePath: string;

  constructor(workspacePath: string, serverName: string = 'context-engine') {
    this.workspacePath = workspacePath;
    this.serviceClient = new ContextServiceClient(workspacePath);

    this.server = new Server(
      {
        name: serverName,
        version: '1.0.0',
      },
      {
        capabilities: {
          tools: {},
        },
      }
    );

    this.setupHandlers();
    this.setupGracefulShutdown();
  }

  /**
   * Set up graceful shutdown handlers
   */
  private setupGracefulShutdown(): void {
    const shutdown = async (signal: string) => {
      if (this.isShuttingDown) return;
      this.isShuttingDown = true;

      console.error(`\nReceived ${signal}, shutting down gracefully...`);

      try {
        // Clear caches
        this.serviceClient.clearCache();

        // Close server connection
        await this.server.close();

        console.error('Server shutdown complete');
        process.exit(0);
      } catch (error) {
        console.error('Error during shutdown:', error);
        process.exit(1);
      }
    };

    process.on('SIGTERM', () => shutdown('SIGTERM'));
    process.on('SIGINT', () => shutdown('SIGINT'));

    // Handle uncaught errors
    process.on('uncaughtException', (error) => {
      console.error('Uncaught exception:', error);
      shutdown('uncaughtException');
    });

    process.on('unhandledRejection', (reason) => {
      console.error('Unhandled rejection:', reason);
      // Don't exit on unhandled rejection, just log
    });
  }

  private setupHandlers(): void {
    // List available tools
    this.server.setRequestHandler(ListToolsRequestSchema, async () => {
      return {
        tools: [
          indexWorkspaceTool,
          semanticSearchTool,
          getFileTool,
          getContextTool,
          enhancePromptTool,
        ],
      };
    });

    // Handle tool calls
    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args } = request.params;
      const startTime = Date.now();

      // Log request (to stderr so it doesn't interfere with stdio transport)
      console.error(`[${new Date().toISOString()}] Tool: ${name}`);

      try {
        let result: string;

        switch (name) {
          case 'index_workspace':
            result = await handleIndexWorkspace(args as any, this.serviceClient);
            break;

          case 'semantic_search':
            result = await handleSemanticSearch(args as any, this.serviceClient);
            break;

          case 'get_file':
            result = await handleGetFile(args as any, this.serviceClient);
            break;

          case 'get_context_for_prompt':
            result = await handleGetContext(args as any, this.serviceClient);
            break;

          case 'enhance_prompt':
            result = await handleEnhancePrompt(args as any, this.serviceClient);
            break;

          default:
            throw new Error(`Unknown tool: ${name}`);
        }

        const elapsed = Date.now() - startTime;
        console.error(`[${new Date().toISOString()}] Tool ${name} completed in ${elapsed}ms`);

        return {
          content: [
            {
              type: 'text',
              text: result,
            },
          ],
        };
      } catch (error) {
        const elapsed = Date.now() - startTime;
        const errorMessage = error instanceof Error ? error.message : String(error);

        console.error(`[${new Date().toISOString()}] Tool ${name} failed after ${elapsed}ms: ${errorMessage}`);

        return {
          content: [
            {
              type: 'text',
              text: `Error: ${errorMessage}`,
            },
          ],
          isError: true,
        };
      }
    });
  }

  async run(): Promise<void> {
    const transport = new StdioServerTransport();
    await this.server.connect(transport);

    console.error('='.repeat(60));
    console.error('Context Engine MCP Server v1.0.0');
    console.error('='.repeat(60));
    console.error(`Workspace: ${this.workspacePath}`);
    console.error('Transport: stdio');
    console.error('');
    console.error('Available tools:');
    console.error('  - index_workspace: Index workspace files for semantic search');
    console.error('  - semantic_search: Find code by semantic meaning');
    console.error('  - get_file: Retrieve file contents');
    console.error('  - get_context_for_prompt: Get enhanced context for prompts');
    console.error('  - enhance_prompt: Enhance prompts with codebase context');
    console.error('');
    console.error('Server ready. Waiting for requests...');
    console.error('='.repeat(60));
  }

  async indexWorkspace(): Promise<void> {
    await this.serviceClient.indexWorkspace();
  }

  /**
   * Get the workspace path
   */
  getWorkspacePath(): string {
    return this.workspacePath;
  }
}

