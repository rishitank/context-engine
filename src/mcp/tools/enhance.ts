/**
 * Layer 3: MCP Interface Layer - Enhance Prompt Tool
 *
 * Transforms simple user prompts into detailed, structured prompts following
 * the Augment CLI Prompt Enhancer pattern.
 *
 * Key Behavior:
 * - Takes a simple prompt like "fix the login bug"
 * - Searches the codebase for relevant files and patterns
 * - Transforms it into a comprehensive, actionable prompt with:
 *   - Specific file references from the codebase
 *   - Numbered action steps
 *   - Context about the project's architecture
 *   - Coding conventions and patterns
 *
 * Example:
 * Input:  "fix the login bug"
 * Output: "Fix the authentication bug in the login flow. Please:
 *          1. Review the current login implementation in `src/auth/login.ts`
 *          2. Check for issues with token validation and session management
 *          ..."
 */

import { ContextServiceClient, ContextOptions, ContextBundle } from '../serviceClient.js';

export interface EnhancePromptArgs {
  /** The raw user prompt to enhance */
  prompt: string;
  /** Maximum number of files to search for context (default: 10) */
  max_files?: number;
  /**
   * Use AI-powered enhancement via Augment's searchAndAsk() API (default: true)
   *
   * When true (default):
   * - Uses Augment's LLM API to intelligently rewrite the prompt
   * - Produces natural language enhancement
   * - Requires network access and valid authentication (auggie login)
   *
   * When false:
   * - Uses fast, template-based enhancement
   * - Works offline
   * - Produces structured output with numbered steps and file references
   */
  use_ai?: boolean;
}

// ============================================================================
// AI-Powered Enhancement (using searchAndAsk)
// ============================================================================

/**
 * Enhancement prompt template following the official Augment SDK example
 * from enhance-handler.ts in the prompt-enhancer-server
 */
function buildAIEnhancementPrompt(originalPrompt: string): string {
  return (
    "Here is an instruction that I'd like to give you, but it needs to be improved. " +
    "Rewrite and enhance this instruction to make it clearer, more specific, " +
    "less ambiguous, and correct any mistakes. " +
    "If there is code in triple backticks (```) consider whether it is a code sample and should remain unchanged. " +
    "Reply with the following format:\n\n" +
    "### BEGIN RESPONSE ###\n" +
    "Here is an enhanced version of the original instruction that is more specific and clear:\n" +
    "<enhanced-prompt>enhanced prompt goes here</enhanced-prompt>\n\n" +
    "### END RESPONSE ###\n\n" +
    "Here is my original instruction:\n\n" +
    originalPrompt
  );
}

/**
 * Parse the enhanced prompt from the AI response
 * Extracts content between <enhanced-prompt> and </enhanced-prompt> tags
 *
 * Following the official SDK's response-parser.ts pattern
 */
function parseEnhancedPrompt(response: string): string | null {
  // Regex for extracting enhanced prompt from AI response
  const ENHANCED_PROMPT_REGEX = /<enhanced-prompt>([\s\S]*?)<\/enhanced-prompt>/;

  const match = response.match(ENHANCED_PROMPT_REGEX);

  if (match?.[1]) {
    return match[1].trim();
  }

  return null;
}

/**
 * Handle AI-powered prompt enhancement using searchAndAsk
 *
 * Uses the official Augment SDK pattern from the prompt-enhancer-server example:
 * 1. Use the original prompt as the search query to find relevant code
 * 2. Send an enhancement prompt to the LLM with the codebase context
 * 3. Parse the enhanced prompt from the response
 */
async function handleAIEnhance(
  prompt: string,
  serviceClient: ContextServiceClient
): Promise<string> {
  console.error(`[AI Enhancement] Enhancing prompt: "${prompt.substring(0, 100)}..."`);

  // Build the enhancement instruction
  const enhancementPrompt = buildAIEnhancementPrompt(prompt);

  try {
    // Use searchAndAsk to get the enhancement with relevant codebase context
    // The original prompt is used as the search query to find relevant code
    const response = await serviceClient.searchAndAsk(prompt, enhancementPrompt);

    // Parse the enhanced prompt from the response
    const enhanced = parseEnhancedPrompt(response);

    if (!enhanced) {
      // If parsing fails, return the raw response with a note
      console.error('[AI Enhancement] Failed to parse enhanced prompt from response, returning raw response');
      console.error(`[AI Enhancement] Response preview: ${response.substring(0, 200)}...`);

      // Try to extract any useful content from the response
      // If the response doesn't contain the expected tags, it might still be useful
      if (response && response.length > 0) {
        return `${response}\n\n---\n_Note: AI enhancement completed but response format was unexpected._`;
      }

      throw new Error('AI enhancement returned empty response');
    }

    console.error(`[AI Enhancement] Successfully enhanced prompt (${enhanced.length} chars)`);
    return enhanced;
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.error(`[AI Enhancement] Error: ${errorMessage}`);

    // Check for authentication errors
    if (errorMessage.includes('API key') || errorMessage.includes('authentication') || errorMessage.includes('Login')) {
      throw new Error(
        'AI enhancement requires authentication. Please run "auggie login" or set AUGMENT_API_TOKEN environment variable.'
      );
    }

    throw error;
  }
}

/**
 * Detect the type of task from the prompt
 */
type TaskType = 'fix' | 'implement' | 'refactor' | 'explain' | 'test' | 'review' | 'general';

function detectTaskType(prompt: string): TaskType {
  const lower = prompt.toLowerCase();

  if (/\b(fix|bug|error|issue|problem|broken|crash|fail)/i.test(lower)) return 'fix';
  if (/\b(implement|create|add|build|make|develop|write)/i.test(lower)) return 'implement';
  if (/\b(refactor|clean|improve|optimize|reorganize|restructure)/i.test(lower)) return 'refactor';
  if (/\b(explain|understand|how does|what is|describe|tell me)/i.test(lower)) return 'explain';
  if (/\b(test|spec|coverage|unit test|integration)/i.test(lower)) return 'test';
  if (/\b(review|check|audit|analyze|evaluate)/i.test(lower)) return 'review';

  return 'general';
}

/**
 * Extract key terms from the prompt for searching
 */
function extractSearchTerms(prompt: string): string {
  // Remove common filler words
  return prompt
    .replace(/^(please|can you|could you|i want to|i need to|help me|show me)\s+/gi, '')
    .replace(/\?$/g, '')
    .trim();
}

/**
 * Generate action steps based on task type and discovered files
 */
function generateActionSteps(
  taskType: TaskType,
  files: ContextBundle['files'],
  originalPrompt: string
): string[] {
  const steps: string[] = [];
  const filePaths = files.map(f => f.path);

  // Get primary files (most relevant)
  const primaryFiles = filePaths.slice(0, 3);
  const hasTests = filePaths.some(f => /\.(test|spec)\.(ts|tsx|js|jsx)$/.test(f) || f.includes('__tests__'));
  const hasMigrations = filePaths.some(f => f.includes('migrations') || f.includes('.sql'));
  const hasActions = filePaths.some(f => f.includes('actions/') || f.includes('/actions'));
  const hasComponents = filePaths.some(f => /\.(tsx|jsx)$/.test(f) && !f.includes('page.'));

  switch (taskType) {
    case 'fix':
      if (primaryFiles.length > 0) {
        steps.push(`Review the current implementation in \`${primaryFiles[0]}\``);
      }
      steps.push('Identify the root cause of the issue');
      if (primaryFiles.length > 1) {
        steps.push(`Check related code in \`${primaryFiles.slice(1).join('`, `')}\``);
      }
      steps.push('Implement the fix while maintaining existing patterns');
      if (hasTests) {
        steps.push('Update or add tests to cover the fix');
      }
      steps.push('Verify the fix works and doesn\'t break existing functionality');
      break;

    case 'implement':
      steps.push('Review existing patterns in the codebase for consistency');
      if (primaryFiles.length > 0) {
        steps.push(`Use \`${primaryFiles[0]}\` as a reference for implementation style`);
      }
      if (hasMigrations) {
        steps.push('Create necessary database migrations if needed');
      }
      if (hasActions) {
        steps.push('Implement server actions following the existing pattern in `actions/`');
      }
      if (hasComponents) {
        steps.push('Create UI components following the existing component patterns');
      }
      steps.push('Add proper TypeScript types and interfaces');
      steps.push('Implement error handling following existing conventions');
      if (hasTests) {
        steps.push('Write tests for the new functionality');
      }
      break;

    case 'refactor':
      if (primaryFiles.length > 0) {
        steps.push(`Analyze the current structure in \`${primaryFiles[0]}\``);
      }
      steps.push('Identify areas for improvement while preserving functionality');
      steps.push('Apply consistent naming conventions and code style');
      steps.push('Ensure type safety is maintained or improved');
      if (hasTests) {
        steps.push('Run existing tests to verify refactoring doesn\'t break functionality');
      }
      steps.push('Document any significant architectural changes');
      break;

    case 'explain':
      if (primaryFiles.length > 0) {
        steps.push(`Examine the implementation in \`${primaryFiles[0]}\``);
      }
      steps.push('Trace the data flow and component relationships');
      if (hasMigrations) {
        steps.push('Review the database schema for understanding the data model');
      }
      steps.push('Identify key patterns and architectural decisions');
      steps.push('Provide a clear explanation with code references');
      break;

    case 'test':
      if (primaryFiles.length > 0) {
        steps.push(`Review the code to test in \`${primaryFiles[0]}\``);
      }
      steps.push('Identify edge cases and important scenarios to cover');
      steps.push('Write unit tests following the existing test patterns');
      steps.push('Add integration tests if needed');
      steps.push('Ensure good test coverage for the functionality');
      break;

    case 'review':
      if (primaryFiles.length > 0) {
        steps.push(`Examine the code in \`${primaryFiles.join('`, `')}\``);
      }
      steps.push('Check for potential bugs or logic errors');
      steps.push('Verify error handling is comprehensive');
      steps.push('Assess code readability and maintainability');
      steps.push('Look for security concerns or performance issues');
      steps.push('Suggest improvements following best practices');
      break;

    default:
      if (primaryFiles.length > 0) {
        steps.push(`Review relevant files: \`${primaryFiles.join('`, `')}\``);
      }
      steps.push('Understand the current implementation');
      steps.push('Make changes following existing patterns and conventions');
      steps.push('Ensure proper error handling');
      if (hasTests) {
        steps.push('Update tests as needed');
      }
  }

  return steps;
}

/**
 * Generate context section describing what was found
 */
function generateContextSection(
  files: ContextBundle['files'],
  originalPrompt: string
): string {
  if (files.length === 0) {
    return '';
  }

  const fileTypes = new Set<string>();
  const directories = new Set<string>();

  for (const file of files) {
    fileTypes.add(file.extension.replace('.', '').toUpperCase() || 'unknown');
    const dir = file.path.split(/[/\\]/)[0];
    if (dir) directories.add(dir);
  }

  let context = '\nContext: ';
  context += `This relates to code in ${Array.from(directories).slice(0, 3).join(', ')}. `;
  context += `Relevant file types: ${Array.from(fileTypes).slice(0, 4).join(', ')}. `;
  context += 'Please maintain consistency with existing patterns ';
  context += 'and ensure proper error handling is in place.';

  return context;
}

/**
 * Capitalize the first letter of a string
 */
function capitalizeFirst(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

/**
 * Expand/enhance the original prompt into a more detailed description
 */
function enhancePromptText(prompt: string, taskType: TaskType): string {
  const clean = prompt
    .replace(/^(please|can you|could you|i want to|i need to|help me)\s+/gi, '')
    .trim();

  // Expand common short prompts
  const expansions: Record<string, string> = {
    'fix': 'Fix the issue in',
    'implement': 'Implement the functionality for',
    'add': 'Add the feature for',
    'create': 'Create the implementation for',
    'refactor': 'Refactor and improve',
    'test': 'Write tests for',
    'review': 'Review and analyze',
  };

  // Check if prompt starts with a common verb
  for (const [verb, expansion] of Object.entries(expansions)) {
    if (clean.toLowerCase().startsWith(verb + ' ')) {
      return capitalizeFirst(clean);
    }
  }

  // For very short prompts, add more context
  if (clean.length < 30) {
    switch (taskType) {
      case 'fix':
        return `Fix the issue related to ${clean}`;
      case 'implement':
        return `Implement ${clean}`;
      case 'refactor':
        return `Refactor and improve ${clean}`;
      case 'test':
        return `Write comprehensive tests for ${clean}`;
      case 'review':
        return `Review and analyze ${clean}`;
      default:
        return capitalizeFirst(clean);
    }
  }

  return capitalizeFirst(clean);
}

/**
 * Build the final enhanced prompt in Augment's style
 */
function buildEnhancedPrompt(
  originalPrompt: string,
  taskType: TaskType,
  bundle: ContextBundle
): string {
  // Generate the enhanced prompt text
  const enhancedDescription = enhancePromptText(originalPrompt, taskType);

  // Generate action steps based on discovered files
  const steps = generateActionSteps(taskType, bundle.files, originalPrompt);

  // Generate context section
  const contextInfo = generateContextSection(bundle.files, originalPrompt);

  // Build the final enhanced prompt
  let enhanced = enhancedDescription;

  // Add context if files were found
  if (contextInfo) {
    enhanced += contextInfo;
  }

  // Add action steps
  if (steps.length > 0) {
    enhanced += '\n\nPlease:';
    steps.forEach((step, index) => {
      enhanced += `\n${index + 1}. ${step}`;
    });
  }

  // Add file reference section if we have relevant files
  if (bundle.files.length > 0) {
    enhanced += '\n\nRelevant files to consider:';
    for (const file of bundle.files.slice(0, 6)) {
      enhanced += `\n- \`${file.path}\``;
      if (file.summary) {
        enhanced += ` - ${file.summary}`;
      }
    }
  }

  return enhanced;
}

/**
 * Handle the enhance_prompt tool call
 *
 * Supports two modes:
 * 1. AI-powered (default, use_ai=true): Uses Augment's LLM API for intelligent enhancement
 * 2. Template-based (use_ai=false): Fast, offline, structured output
 */
export async function handleEnhancePrompt(
  args: EnhancePromptArgs,
  serviceClient: ContextServiceClient
): Promise<string> {
  const { prompt, max_files = 10, use_ai = true } = args;

  // Validate inputs
  if (!prompt || typeof prompt !== 'string') {
    throw new Error('Invalid prompt parameter: must be a non-empty string');
  }

  if (prompt.length > 10000) {
    throw new Error('Prompt too long: maximum 10000 characters');
  }

  // Use AI-powered enhancement if requested
  if (use_ai) {
    console.error('[enhance_prompt] Using AI-powered enhancement mode');
    return handleAIEnhance(prompt, serviceClient);
  }

  // Template-based enhancement (default)
  console.error('[enhance_prompt] Using template-based enhancement mode');

  const clampedMaxFiles = Math.min(Math.max(max_files, 1), 15);

  // Detect task type
  const taskType = detectTaskType(prompt);

  // Extract search terms
  const searchTerms = extractSearchTerms(prompt);

  // Build context options
  const contextOptions: ContextOptions = {
    maxFiles: clampedMaxFiles,
    tokenBudget: 8000,
    includeRelated: true,
    minRelevance: 0.2, // Lower threshold for wider search
    includeSummaries: true,
  };

  // Retrieve relevant context using the service client
  const contextBundle = await serviceClient.getContextForPrompt(searchTerms, contextOptions);

  // Build the enhanced prompt
  const enhancedPrompt = buildEnhancedPrompt(prompt, taskType, contextBundle);

  return enhancedPrompt;
}

/**
 * Tool schema definition for MCP registration
 */
export const enhancePromptTool = {
  name: 'enhance_prompt',
  description: `Transform a simple prompt into a detailed, structured prompt with codebase context.

This tool follows Augment's Prompt Enhancer pattern and supports two modes:

**AI Mode (default, use_ai=true):**
- Uses Augment's LLM API for intelligent rewriting
- Produces natural language enhancement
- Requires network access and authentication (auggie login)

**Template Mode (use_ai=false):**
- Fast, works offline
- Returns structured output with numbered steps and file references
- Deterministic - same input produces same output

Example (AI Mode - default):
  Input:  { prompt: "fix the login bug" }
  Output: "Debug and fix the user authentication issue in the login flow.
           Specifically, investigate the login function in src/auth/login.ts
           which handles JWT token validation and session management..."

Example (Template Mode):
  Input:  { prompt: "fix the login bug", use_ai: false }
  Output: "Fix the authentication issue in the login flow.
           Context: This relates to code in src/auth. Relevant file types: TS, TSX.

           Please:
           1. Review the current implementation in \`src/auth/login.ts\`
           2. Identify the root cause of the issue
           3. Implement the fix while maintaining existing patterns

           Relevant files to consider:
           - \`src/auth/login.ts\` - handles user login"

Use AI mode for intelligent, context-aware enhancement. Use template mode for fast, predictable results when offline.`,
  inputSchema: {
    type: 'object',
    properties: {
      prompt: {
        type: 'string',
        description: 'The simple prompt to enhance (e.g., "fix the login bug")',
      },
      max_files: {
        type: 'number',
        description: 'Maximum number of relevant files to search for in template mode (default: 10)',
        default: 10,
      },
      use_ai: {
        type: 'boolean',
        description: 'Use AI-powered enhancement via Augment LLM API (default: true). Uses searchAndAsk() for intelligent rewriting. Requires authentication (auggie login). Set to false for fast, offline template-based enhancement.',
        default: true,
      },
    },
    required: ['prompt'],
  },
};
