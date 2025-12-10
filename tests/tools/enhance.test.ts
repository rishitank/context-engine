/**
 * Unit tests for enhance_prompt tool
 *
 * Tests the Augment-style Prompt Enhancer that transforms simple prompts
 * into detailed, structured prompts with codebase context
 */

import { jest, describe, it, expect, beforeEach } from '@jest/globals';
import { handleEnhancePrompt, EnhancePromptArgs, enhancePromptTool } from '../../src/mcp/tools/enhance.js';
import { ContextServiceClient, ContextBundle, FileContext } from '../../src/mcp/serviceClient.js';

describe('enhance_prompt Tool', () => {
  let mockServiceClient: any;

  beforeEach(() => {
    jest.clearAllMocks();
    mockServiceClient = {
      getFile: jest.fn(),
      semanticSearch: jest.fn(),
      getContextForPrompt: jest.fn(),
      indexWorkspace: jest.fn(),
      clearCache: jest.fn(),
      searchAndAsk: jest.fn(),
    };
  });

  describe('Input Validation', () => {
    it('should reject empty prompt', async () => {
      await expect(handleEnhancePrompt({ prompt: '' }, mockServiceClient as any))
        .rejects.toThrow(/invalid prompt/i);
    });

    it('should reject null prompt', async () => {
      await expect(handleEnhancePrompt({ prompt: null as any }, mockServiceClient as any))
        .rejects.toThrow(/invalid prompt/i);
    });

    it('should reject undefined prompt', async () => {
      await expect(handleEnhancePrompt({ prompt: undefined as any }, mockServiceClient as any))
        .rejects.toThrow(/invalid prompt/i);
    });

    it('should reject prompt over 10000 characters', async () => {
      const longPrompt = 'a'.repeat(10001);
      await expect(handleEnhancePrompt({ prompt: longPrompt }, mockServiceClient as any))
        .rejects.toThrow(/prompt too long/i);
    });

    it('should clamp max_files to valid range instead of rejecting', async () => {
      const mockBundle = createMockContextBundle();
      mockServiceClient.getContextForPrompt.mockResolvedValue(mockBundle);

      // max_files 0 should be clamped to 1, not rejected (use template mode for these tests)
      await expect(handleEnhancePrompt({ prompt: 'test', max_files: 0, use_ai: false }, mockServiceClient as any))
        .resolves.toBeDefined();

      // max_files 20 should be clamped to 15, not rejected
      await expect(handleEnhancePrompt({ prompt: 'test', max_files: 20, use_ai: false }, mockServiceClient as any))
        .resolves.toBeDefined();
    });

    it('should accept valid parameters', async () => {
      const mockBundle = createMockContextBundle();
      mockServiceClient.getContextForPrompt.mockResolvedValue(mockBundle);

      await expect(handleEnhancePrompt({
        prompt: 'How do I implement authentication?',
        max_files: 10,
        use_ai: false,
      }, mockServiceClient as any)).resolves.toBeDefined();
    });
  });

  describe('Task Type Detection (Template Mode)', () => {
    it('should detect fix task type', async () => {
      const mockBundle = createMockContextBundle();
      mockServiceClient.getContextForPrompt.mockResolvedValue(mockBundle);

      const result = await handleEnhancePrompt({ prompt: 'fix the login bug', use_ai: false }, mockServiceClient as any);

      expect(result).toContain('fix');
      expect(result.toLowerCase()).toContain('identify');
    });

    it('should detect implement task type', async () => {
      const mockBundle = createMockContextBundle();
      mockServiceClient.getContextForPrompt.mockResolvedValue(mockBundle);

      const result = await handleEnhancePrompt({ prompt: 'implement user authentication', use_ai: false }, mockServiceClient as any);

      expect(result.toLowerCase()).toContain('implement');
    });

    it('should detect refactor task type', async () => {
      const mockBundle = createMockContextBundle();
      mockServiceClient.getContextForPrompt.mockResolvedValue(mockBundle);

      const result = await handleEnhancePrompt({ prompt: 'refactor the database module', use_ai: false }, mockServiceClient as any);

      expect(result.toLowerCase()).toContain('refactor');
    });

    it('should detect explain task type', async () => {
      const mockBundle = createMockContextBundle();
      mockServiceClient.getContextForPrompt.mockResolvedValue(mockBundle);

      const result = await handleEnhancePrompt({ prompt: 'explain how the API works', use_ai: false }, mockServiceClient as any);

      expect(result.toLowerCase()).toContain('explain');
    });
  });

  describe('Output Formatting (Template Mode)', () => {
    it('should include numbered action steps', async () => {
      const mockBundle = createMockContextBundle();
      mockServiceClient.getContextForPrompt.mockResolvedValue(mockBundle);

      const result = await handleEnhancePrompt({ prompt: 'fix the login bug', use_ai: false }, mockServiceClient as any);

      // Should have numbered steps
      expect(result).toMatch(/\n1\./);
      expect(result).toMatch(/\n2\./);
    });

    it('should include relevant file references', async () => {
      const mockBundle = createMockContextBundle();
      mockServiceClient.getContextForPrompt.mockResolvedValue(mockBundle);

      const result = await handleEnhancePrompt({ prompt: 'test', use_ai: false }, mockServiceClient as any);

      expect(result).toContain('Relevant files to consider:');
      expect(result).toContain('src/auth/login.ts');
    });

    it('should include context about directories', async () => {
      const mockBundle = createMockContextBundle();
      mockServiceClient.getContextForPrompt.mockResolvedValue(mockBundle);

      const result = await handleEnhancePrompt({ prompt: 'test', use_ai: false }, mockServiceClient as any);

      expect(result).toContain('Context:');
      expect(result).toContain('src');
    });

    it('should handle empty context gracefully', async () => {
      const emptyBundle: ContextBundle = {
        summary: 'No results',
        query: 'test',
        files: [],
        hints: [],
        metadata: {
          totalFiles: 0,
          totalSnippets: 0,
          totalTokens: 0,
          tokenBudget: 4000,
          truncated: false,
          searchTimeMs: 50,
        },
      };
      mockServiceClient.getContextForPrompt.mockResolvedValue(emptyBundle);

      const result = await handleEnhancePrompt({ prompt: 'nonexistent feature', use_ai: false }, mockServiceClient as any);

      // Should still produce an enhanced prompt without crashing
      expect(result).toBeDefined();
      expect(typeof result).toBe('string');
    });

    it('should expand short prompts into descriptive text', async () => {
      const mockBundle = createMockContextBundle();
      mockServiceClient.getContextForPrompt.mockResolvedValue(mockBundle);

      const result = await handleEnhancePrompt({ prompt: 'login', use_ai: false }, mockServiceClient as any);

      // Short prompts should be expanded
      expect(result.length).toBeGreaterThan('login'.length);
    });
  });

  describe('Intent Extraction (Template Mode)', () => {
    it('should pass extracted intent to context service', async () => {
      const mockBundle = createMockContextBundle();
      mockServiceClient.getContextForPrompt.mockResolvedValue(mockBundle);

      await handleEnhancePrompt({
        prompt: 'How do I implement user authentication?',
        use_ai: false,
      }, mockServiceClient as any);

      expect(mockServiceClient.getContextForPrompt).toHaveBeenCalled();
      const callArgs = mockServiceClient.getContextForPrompt.mock.calls[0];
      // The intent should be extracted and passed to the service
      expect(callArgs[0]).toBeDefined();
      expect(typeof callArgs[0]).toBe('string');
    });

    it('should handle various prompt formats', async () => {
      const mockBundle = createMockContextBundle();
      mockServiceClient.getContextForPrompt.mockResolvedValue(mockBundle);

      const prompts = [
        'How does the login work?',
        'Please help me understand the database schema',
        'Can you show me the API endpoints?',
        'implement a new feature for user profiles',
      ];

      for (const prompt of prompts) {
        await handleEnhancePrompt({ prompt, use_ai: false }, mockServiceClient as any);
      }

      expect(mockServiceClient.getContextForPrompt).toHaveBeenCalledTimes(prompts.length);
    });
  });

  describe('Tool Schema', () => {
    it('should have correct name', () => {
      expect(enhancePromptTool.name).toBe('enhance_prompt');
    });

    it('should have required prompt property', () => {
      expect(enhancePromptTool.inputSchema.required).toContain('prompt');
    });

    it('should have expected properties for new simpler API', () => {
      const props = Object.keys(enhancePromptTool.inputSchema.properties);
      expect(props).toContain('prompt');
      expect(props).toContain('max_files');
    });

    it('should have descriptive description mentioning Augment Prompt Enhancer', () => {
      expect(enhancePromptTool.description).toContain('Augment');
      expect(enhancePromptTool.description).toContain('Prompt Enhancer');
    });

    it('should include example in description', () => {
      // Check for examples - description now has "Example (Template Mode):" and "Example (AI Mode):"
      expect(enhancePromptTool.description).toContain('Example');
      expect(enhancePromptTool.description).toContain('fix the login bug');
    });
  });

  describe('Action Steps Generation (Template Mode)', () => {
    it('should generate test-specific steps when testing is the task', async () => {
      const mockBundle = createMockContextBundle();
      // Add a test file to the context
      mockBundle.files.push({
        path: 'src/auth/login.test.ts',
        extension: '.ts',
        summary: 'Test file for login',
        relevance: 0.7,
        tokenCount: 100,
        snippets: [],
      });
      mockServiceClient.getContextForPrompt.mockResolvedValue(mockBundle);

      const result = await handleEnhancePrompt({ prompt: 'test the login functionality', use_ai: false }, mockServiceClient as any);

      expect(result.toLowerCase()).toContain('test');
    });

    it('should generate review-specific steps when reviewing', async () => {
      const mockBundle = createMockContextBundle();
      mockServiceClient.getContextForPrompt.mockResolvedValue(mockBundle);

      const result = await handleEnhancePrompt({ prompt: 'review the authentication code', use_ai: false }, mockServiceClient as any);

      expect(result.toLowerCase()).toContain('review');
    });
  });

  describe('AI Enhancement Mode (use_ai=true)', () => {
    it('should use searchAndAsk when use_ai is true', async () => {
      const aiResponse = `### BEGIN RESPONSE ###
Here is an enhanced version of the original instruction that is more specific and clear:
<enhanced-prompt>Fix the authentication bug in the login flow. Review the JWT token validation logic in src/auth/login.ts and ensure proper session management.</enhanced-prompt>

### END RESPONSE ###`;

      mockServiceClient.searchAndAsk.mockResolvedValue(aiResponse);

      const result = await handleEnhancePrompt({
        prompt: 'fix the login bug',
        use_ai: true,
      }, mockServiceClient as any);

      expect(mockServiceClient.searchAndAsk).toHaveBeenCalledTimes(1);
      expect(result).toContain('Fix the authentication bug');
      expect(result).toContain('JWT token validation');
    });

    it('should NOT use searchAndAsk when use_ai is explicitly false', async () => {
      const mockBundle = createMockContextBundle();
      mockServiceClient.getContextForPrompt.mockResolvedValue(mockBundle);

      await handleEnhancePrompt({
        prompt: 'fix the login bug',
        use_ai: false,
      }, mockServiceClient as any);

      expect(mockServiceClient.searchAndAsk).not.toHaveBeenCalled();
      expect(mockServiceClient.getContextForPrompt).toHaveBeenCalled();
    });

    it('should parse enhanced prompt from AI response with XML tags', async () => {
      const enhancedText = 'This is the enhanced prompt that was generated by AI';
      const aiResponse = `Some preamble text
<enhanced-prompt>${enhancedText}</enhanced-prompt>
Some postamble text`;

      mockServiceClient.searchAndAsk.mockResolvedValue(aiResponse);

      const result = await handleEnhancePrompt({
        prompt: 'simple prompt',
        use_ai: true,
      }, mockServiceClient as any);

      expect(result).toBe(enhancedText);
    });

    it('should handle multi-line enhanced prompts', async () => {
      const multiLinePrompt = `Line 1 of the prompt
Line 2 with more details
Line 3 with even more context`;
      const aiResponse = `<enhanced-prompt>${multiLinePrompt}</enhanced-prompt>`;

      mockServiceClient.searchAndAsk.mockResolvedValue(aiResponse);

      const result = await handleEnhancePrompt({
        prompt: 'test',
        use_ai: true,
      }, mockServiceClient as any);

      expect(result).toBe(multiLinePrompt);
    });

    it('should return raw response when XML tags are missing', async () => {
      const rawResponse = 'AI response without expected XML tags';
      mockServiceClient.searchAndAsk.mockResolvedValue(rawResponse);

      const result = await handleEnhancePrompt({
        prompt: 'test',
        use_ai: true,
      }, mockServiceClient as any);

      expect(result).toContain(rawResponse);
      expect(result).toContain('response format was unexpected');
    });

    it('should throw error when searchAndAsk returns empty response', async () => {
      mockServiceClient.searchAndAsk.mockResolvedValue('');

      await expect(handleEnhancePrompt({
        prompt: 'test',
        use_ai: true,
      }, mockServiceClient as any)).rejects.toThrow(/empty response/i);
    });

    it('should throw authentication error with helpful message', async () => {
      mockServiceClient.searchAndAsk.mockRejectedValue(new Error('API key is required'));

      await expect(handleEnhancePrompt({
        prompt: 'test',
        use_ai: true,
      }, mockServiceClient as any)).rejects.toThrow(/authentication/i);
    });

    it('should propagate other errors from searchAndAsk', async () => {
      mockServiceClient.searchAndAsk.mockRejectedValue(new Error('Network timeout'));

      await expect(handleEnhancePrompt({
        prompt: 'test',
        use_ai: true,
      }, mockServiceClient as any)).rejects.toThrow('Network timeout');
    });
  });

  describe('use_ai Parameter in Tool Schema', () => {
    it('should have use_ai property defined in schema', () => {
      const props = Object.keys(enhancePromptTool.inputSchema.properties);
      expect(props).toContain('use_ai');
    });

    it('should have use_ai with correct type and default', () => {
      const useAiProp = (enhancePromptTool.inputSchema.properties as any).use_ai;
      expect(useAiProp.type).toBe('boolean');
      expect(useAiProp.default).toBe(true);
    });

    it('should mention AI mode in description', () => {
      expect(enhancePromptTool.description).toContain('AI Mode');
      expect(enhancePromptTool.description).toContain('use_ai');
    });
  });
});

/**
 * Helper to create mock context bundle
 */
function createMockContextBundle(): ContextBundle {
  const mockFile: FileContext = {
    path: 'src/auth/login.ts',
    extension: '.ts',
    summary: 'Authentication module with login functions',
    relevance: 0.85,
    tokenCount: 200,
    snippets: [
      {
        text: 'export function testHelper() {\n  return true;\n}',
        lines: '10-15',
        relevance: 0.85,
        tokenCount: 20,
        codeType: 'function',
      },
    ],
  };

  return {
    summary: 'Context for "authentication": 1 files from src/auth, primarily containing function definitions',
    query: 'authentication',
    files: [mockFile],
    hints: ['File types: .ts (1)', 'Code patterns: function (1)'],
    metadata: {
      totalFiles: 1,
      totalSnippets: 1,
      totalTokens: 200,
      tokenBudget: 4000,
      truncated: false,
      searchTimeMs: 100,
    },
  };
}

