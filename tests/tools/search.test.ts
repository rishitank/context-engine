/**
 * Unit tests for semantic_search tool
 *
 * Tests the Layer 3 - MCP Interface functionality for semantic search
 */

import { jest, describe, it, expect, beforeEach } from '@jest/globals';
import { handleSemanticSearch, SemanticSearchArgs, semanticSearchTool } from '../../src/mcp/tools/search.js';
import { ContextServiceClient, SearchResult } from '../../src/mcp/serviceClient.js';

describe('semantic_search Tool', () => {
  let mockServiceClient: any;

  beforeEach(() => {
    jest.clearAllMocks();
    mockServiceClient = {
      getFile: jest.fn(),
      semanticSearch: jest.fn(),
      getContextForPrompt: jest.fn(),
      indexWorkspace: jest.fn(),
      clearCache: jest.fn(),
    };
  });

  describe('Input Validation', () => {
    it('should reject empty query', async () => {
      await expect(handleSemanticSearch({ query: '' }, mockServiceClient as any))
        .rejects.toThrow(/invalid query/i);
    });

    it('should reject null query', async () => {
      await expect(handleSemanticSearch({ query: null as any }, mockServiceClient as any))
        .rejects.toThrow(/invalid query/i);
    });

    it('should reject query over 500 characters', async () => {
      const longQuery = 'a'.repeat(501);
      await expect(handleSemanticSearch({ query: longQuery }, mockServiceClient as any))
        .rejects.toThrow(/query too long/i);
    });

    it('should reject top_k less than 1', async () => {
      await expect(handleSemanticSearch({ query: 'test', top_k: 0 }, mockServiceClient as any))
        .rejects.toThrow(/invalid top_k/i);
    });

    it('should reject top_k greater than 50', async () => {
      await expect(handleSemanticSearch({ query: 'test', top_k: 51 }, mockServiceClient as any))
        .rejects.toThrow(/invalid top_k/i);
    });

    it('should accept valid parameters', async () => {
      mockServiceClient.semanticSearch.mockResolvedValue([]);

      await expect(handleSemanticSearch({
        query: 'test query',
        top_k: 10,
      }, mockServiceClient as any)).resolves.toBeDefined();
    });
  });

  describe('Output Formatting', () => {
    it('should show empty state message when no results', async () => {
      mockServiceClient.semanticSearch.mockResolvedValue([]);

      const result = await handleSemanticSearch({ query: 'nonexistent' }, mockServiceClient as any);

      expect(result).toContain('No results found');
      expect(result).toContain('# 🔍 Search Results');
    });

    it('should include search results header', async () => {
      const mockResults: SearchResult[] = [
        { path: 'src/test.ts', content: 'test content', score: 0.9, lines: '1-5', relevanceScore: 0.9 },
      ];
      mockServiceClient.semanticSearch.mockResolvedValue(mockResults);

      const result = await handleSemanticSearch({ query: 'test' }, mockServiceClient as any);

      expect(result).toContain('# 🔍 Search Results');
      expect(result).toContain('**Found:**');
    });

    it('should group results by file', async () => {
      const mockResults: SearchResult[] = [
        { path: 'src/a.ts', content: 'content a', score: 0.9, lines: '1-5', relevanceScore: 0.9 },
        { path: 'src/a.ts', content: 'more a', score: 0.8, lines: '10-15', relevanceScore: 0.8 },
        { path: 'src/b.ts', content: 'content b', score: 0.7, lines: '1-3', relevanceScore: 0.7 },
      ];
      mockServiceClient.semanticSearch.mockResolvedValue(mockResults);

      const result = await handleSemanticSearch({ query: 'test' }, mockServiceClient as any);

      // Should have grouped file headings
      expect(result).toContain('`src/a.ts`');
      expect(result).toContain('`src/b.ts`');
    });

    it('should show code previews', async () => {
      const mockResults: SearchResult[] = [
        { path: 'src/test.ts', content: 'function test() { return true; }', score: 0.9, lines: '1-1', relevanceScore: 0.9 },
      ];
      mockServiceClient.semanticSearch.mockResolvedValue(mockResults);

      const result = await handleSemanticSearch({ query: 'test' }, mockServiceClient as any);

      expect(result).toContain('```');
      expect(result).toContain('function test()');
    });
  });

  describe('Tool Schema', () => {
    it('should have correct name', () => {
      expect(semanticSearchTool.name).toBe('semantic_search');
    });

    it('should have required query property', () => {
      expect(semanticSearchTool.inputSchema.required).toContain('query');
    });

    it('should have query and top_k properties', () => {
      const props = Object.keys(semanticSearchTool.inputSchema.properties);
      expect(props).toContain('query');
      expect(props).toContain('top_k');
    });
  });
});

