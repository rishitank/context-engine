/**
 * Layer 2: Context Service Layer
 *
 * This layer adapts raw retrieval from the Auggie SDK (Layer 1)
 * into agent-friendly context bundles optimized for prompt enhancement.
 *
 * Responsibilities:
 * - Decide how much context to return
 * - Format snippets for optimal LLM consumption
 * - Deduplicate results by file path
 * - Enforce token/file limits
 * - Apply relevance scoring and ranking
 * - Generate context summaries and hints
 * - Manage token budgets for LLM context windows
 */

import { DirectContext } from '@augmentcode/auggie-sdk';
import * as fs from 'fs';
import * as path from 'path';
import { minimatch } from 'minimatch';

// ============================================================================
// Type Definitions
// ============================================================================

export interface SearchResult {
  path: string;
  content: string;
  score?: number;
  lines?: string;
  /** Relevance score normalized to 0-1 range */
  relevanceScore?: number;
}

export interface SnippetInfo {
  text: string;
  lines: string;
  /** Relevance score for this snippet (0-1) */
  relevance: number;
  /** Estimated token count */
  tokenCount: number;
  /** Type of code (function, class, import, etc.) */
  codeType?: string;
}

export interface FileContext {
  path: string;
  /** File extension for syntax highlighting hints */
  extension: string;
  /** High-level summary of what this file contains */
  summary: string;
  /** Relevance score for this file (0-1) */
  relevance: number;
  /** Estimated total token count for this file's context */
  tokenCount: number;
  snippets: SnippetInfo[];
  /** Related files that might be needed for full context */
  relatedFiles?: string[];
}

export interface ContextBundle {
  /** High-level summary of the context */
  summary: string;
  /** Query that generated this context */
  query: string;
  /** Files with relevant context, ordered by relevance */
  files: FileContext[];
  /** Key insights and hints for the LLM */
  hints: string[];
  /** Metadata about the context bundle */
  metadata: {
    totalFiles: number;
    totalSnippets: number;
    totalTokens: number;
    tokenBudget: number;
    truncated: boolean;
    searchTimeMs: number;
  };
}

export interface ContextOptions {
  /** Maximum number of files to include (default: 5) */
  maxFiles?: number;
  /** Maximum tokens for the entire context (default: 8000) */
  tokenBudget?: number;
  /** Include related/dependency files (default: true) */
  includeRelated?: boolean;
  /** Minimum relevance score to include (0-1, default: 0.3) */
  minRelevance?: number;
  /** Include file summaries (default: true) */
  includeSummaries?: boolean;
}

// ============================================================================
// Constants
// ============================================================================

/** Maximum file size to read (10MB) */
const MAX_FILE_SIZE = 10 * 1024 * 1024;

/** Default token budget for context */
const DEFAULT_TOKEN_BUDGET = 8000;

/** Approximate characters per token (conservative estimate) */
const CHARS_PER_TOKEN = 4;

/** Cache TTL in milliseconds (1 minute) */
const CACHE_TTL_MS = 60000;

/** State file name for persisting index state */
const STATE_FILE_NAME = '.augment-context-state.json';

/** Context ignore file names (in order of preference) */
const CONTEXT_IGNORE_FILES = ['.contextignore', '.augment-ignore'];

/** Default directories to always exclude */
const DEFAULT_EXCLUDED_DIRS = new Set([
  'node_modules',
  'dist',
  'build',
  'out',
  '.git',
  '.svn',
  '.hg',
  '__pycache__',
  'venv',
  '.venv',
  'env',
  '.env',
  '.tox',
  '.nox',
  'target',          // Rust, Java/Maven
  'bin',             // Go, .NET
  'obj',             // .NET
  'vendor',          // Go, PHP
  'Pods',            // iOS/CocoaPods
  '.gradle',
  '.idea',
  '.vscode',
  '.vs',
  'coverage',
  '.nyc_output',
  '.next',
  '.nuxt',
  '.output',
  '.cache',
  '.parcel-cache',
  '.turbo',
  'resources',       // IDE resources (e.g., Antigravity)
  'extensions',      // IDE extensions
]);

/** Default file patterns to always exclude */
const DEFAULT_EXCLUDED_PATTERNS = [
  '*.min.js',
  '*.min.css',
  '*.bundle.js',
  '*.chunk.js',
  '*.map',
  '*.lock',
  'package-lock.json',
  'yarn.lock',
  'pnpm-lock.yaml',
  'Cargo.lock',
  'Gemfile.lock',
  'poetry.lock',
  '*.log',
  '*.tmp',
  '*.temp',
  '*.bak',
  '*.swp',
  '*.swo',
  '*.pyc',
  '*.pyo',
  '*.class',
  '*.dll',
  '*.exe',
  '*.so',
  '*.dylib',
  '*.o',
  '*.obj',
  '*.wasm',
];

/** File extensions to index */
const INDEXABLE_EXTENSIONS = new Set([
  '.ts', '.tsx', '.js', '.jsx', '.mjs', '.cjs',
  '.py', '.pyw',
  '.java', '.kt', '.kts', '.scala',
  '.go',
  '.rs',
  '.c', '.cpp', '.cc', '.cxx', '.h', '.hpp', '.hxx',
  '.cs',
  '.rb',
  '.php',
  '.swift',
  '.m', '.mm',
  '.vue', '.svelte',
  '.json', '.yaml', '.yml', '.toml',
  '.md', '.mdx',
  '.sql',
  '.sh', '.bash', '.zsh', '.fish',
  '.ps1', '.psm1',
  '.dockerfile',
  '.tf', '.hcl',
]);

// ============================================================================
// Cache Entry Type
// ============================================================================

interface CacheEntry<T> {
  data: T;
  timestamp: number;
}

// ============================================================================
// Context Service Client
// ============================================================================

export class ContextServiceClient {
  private workspacePath: string;
  private context: DirectContext | null = null;
  private initPromise: Promise<void> | null = null;

  /** LRU cache for search results */
  private searchCache: Map<string, CacheEntry<SearchResult[]>> = new Map();

  /** Maximum cache size */
  private readonly maxCacheSize = 100;

  /** Loaded ignore patterns (from .gitignore and .contextignore) */
  private ignorePatterns: string[] = [];

  /** Flag to track if ignore patterns have been loaded */
  private ignorePatternsLoaded: boolean = false;

  constructor(workspacePath: string) {
    this.workspacePath = workspacePath;
  }

  /**
   * Load ignore patterns from .gitignore and .contextignore files
   */
  private loadIgnorePatterns(): void {
    if (this.ignorePatternsLoaded) return;

    this.ignorePatterns = [...DEFAULT_EXCLUDED_PATTERNS];

    // Try to load .gitignore
    const gitignorePath = path.join(this.workspacePath, '.gitignore');
    if (fs.existsSync(gitignorePath)) {
      try {
        const content = fs.readFileSync(gitignorePath, 'utf-8');
        const patterns = this.parseIgnoreFile(content);
        this.ignorePatterns.push(...patterns);
        console.error(`Loaded ${patterns.length} patterns from .gitignore`);
      } catch (error) {
        console.error('Error loading .gitignore:', error);
      }
    }

    // Try to load context ignore files (.contextignore or .augment-ignore)
    for (const ignoreFileName of CONTEXT_IGNORE_FILES) {
      const contextIgnorePath = path.join(this.workspacePath, ignoreFileName);
      if (fs.existsSync(contextIgnorePath)) {
        try {
          const content = fs.readFileSync(contextIgnorePath, 'utf-8');
          const patterns = this.parseIgnoreFile(content);
          this.ignorePatterns.push(...patterns);
          console.error(`Loaded ${patterns.length} patterns from ${ignoreFileName}`);
        } catch (error) {
          console.error(`Error loading ${ignoreFileName}:`, error);
        }
      }
    }

    console.error(`Total ignore patterns loaded: ${this.ignorePatterns.length}`);
    this.ignorePatternsLoaded = true;
  }

  /**
   * Parse an ignore file content into patterns
   */
  private parseIgnoreFile(content: string): string[] {
    return content
      .split('\n')
      .map(line => line.trim())
      .filter(line => line && !line.startsWith('#')); // Remove empty lines and comments
  }

  /**
   * Check if a path should be ignored based on loaded patterns
   *
   * Handles gitignore-style patterns:
   * - Patterns starting with / are anchored to root
   * - Patterns ending with / only match directories
   * - Other patterns match anywhere in the path
   */
  private shouldIgnorePath(relativePath: string): boolean {
    // Normalize path separators
    const normalizedPath = relativePath.replace(/\\/g, '/');
    const fileName = path.basename(normalizedPath);

    for (const rawPattern of this.ignorePatterns) {
      let pattern = rawPattern;

      // Skip negation patterns (gitignore !pattern)
      if (pattern.startsWith('!')) continue;

      // Handle root-anchored patterns (starting with /)
      const isRootAnchored = pattern.startsWith('/');
      if (isRootAnchored) {
        pattern = pattern.slice(1);
      }

      // Handle directory-only patterns (ending with /)
      const isDirOnly = pattern.endsWith('/');
      if (isDirOnly) {
        pattern = pattern.slice(0, -1);
      }

      // For simple patterns without wildcards or slashes, match against filename
      if (!pattern.includes('/') && !pattern.includes('*') && !pattern.includes('?')) {
        if (fileName === pattern || normalizedPath === pattern) {
          return true;
        }
        continue;
      }

      // For glob patterns, use minimatch
      try {
        // If root-anchored, match from the start
        if (isRootAnchored) {
          if (minimatch(normalizedPath, pattern, { dot: true })) {
            return true;
          }
        } else {
          // Match anywhere in path (using matchBase for simple patterns)
          if (minimatch(normalizedPath, pattern, { dot: true, matchBase: !pattern.includes('/') })) {
            return true;
          }
          // Also try matching with ** prefix for patterns without it
          if (!pattern.startsWith('**') && minimatch(normalizedPath, `**/${pattern}`, { dot: true })) {
            return true;
          }
        }
      } catch {
        // Invalid pattern, skip
      }
    }
    return false;
  }

  // ==========================================================================
  // SDK Initialization
  // ==========================================================================

  /**
   * Get the state file path for this workspace
   */
  private getStateFilePath(): string {
    return path.join(this.workspacePath, STATE_FILE_NAME);
  }

  /**
   * Initialize the DirectContext SDK
   * Tries to restore from saved state if available
   */
  private async ensureInitialized(): Promise<DirectContext> {
    if (this.context) {
      return this.context;
    }

    // Prevent concurrent initialization
    if (this.initPromise) {
      await this.initPromise;
      return this.context!;
    }

    this.initPromise = this.doInitialize();
    await this.initPromise;
    return this.context!;
  }

  private async doInitialize(): Promise<void> {
    const stateFilePath = this.getStateFilePath();

    try {
      // Try to restore from saved state
      if (fs.existsSync(stateFilePath)) {
        console.error(`Restoring context from ${stateFilePath}`);
        this.context = await DirectContext.importFromFile(stateFilePath);
        console.error('Context restored successfully');
        return;
      }
    } catch (error) {
      console.error('Failed to restore context state, creating new context:', error);
      // Delete corrupted state file
      try {
        fs.unlinkSync(stateFilePath);
        console.error('Deleted corrupted state file');
      } catch {
        // Ignore deletion errors
      }
    }

    // Create new context
    console.error('Creating new DirectContext');
    try {
      this.context = await DirectContext.create();
      console.error('DirectContext created successfully');
    } catch (createError) {
      console.error('Failed to create DirectContext:', createError);
      // Check if this is an API/authentication error
      const errorMessage = String(createError);
      if (errorMessage.includes('invalid character') || errorMessage.includes('Login')) {
        console.error('\n*** AUTHENTICATION ERROR ***');
        console.error('The API returned an invalid response. Please check:');
        console.error('1. AUGMENT_API_TOKEN is set correctly');
        console.error('2. AUGMENT_API_URL is set correctly');
        console.error('3. Your API token has not expired');
        console.error('');
      }
      throw createError;
    }

    // Auto-index workspace if no state file exists
    console.error('No existing index found - auto-indexing workspace...');
    try {
      await this.indexWorkspace();
      console.error('Auto-indexing completed');
    } catch (error) {
      console.error('Auto-indexing failed (you can manually call index_workspace tool):', error);
      // Don't throw - allow server to start even if auto-indexing fails
      // User can manually trigger indexing later
    }
  }

  /**
   * Save the current context state to disk
   */
  private async saveState(): Promise<void> {
    if (!this.context) return;

    try {
      const stateFilePath = this.getStateFilePath();
      await this.context.exportToFile(stateFilePath);
      console.error(`Context state saved to ${stateFilePath}`);
    } catch (error) {
      console.error('Failed to save context state:', error);
    }
  }

  // ==========================================================================
  // File Discovery
  // ==========================================================================

  /**
   * Check if a file should be indexed based on extension
   */
  private shouldIndexFile(filePath: string): boolean {
    const ext = path.extname(filePath).toLowerCase();
    return INDEXABLE_EXTENSIONS.has(ext);
  }

  /**
   * Recursively discover all indexable files in a directory
   */
  private async discoverFiles(dirPath: string, relativeTo: string = dirPath): Promise<string[]> {
    // Load ignore patterns on first call
    this.loadIgnorePatterns();

    const files: string[] = [];

    try {
      const entries = fs.readdirSync(dirPath, { withFileTypes: true });

      for (const entry of entries) {
        const fullPath = path.join(dirPath, entry.name);
        const relativePath = path.relative(relativeTo, fullPath);

        // Skip hidden files/directories (starting with .)
        if (entry.name.startsWith('.')) {
          continue;
        }

        // Skip default excluded directories
        if (entry.isDirectory() && DEFAULT_EXCLUDED_DIRS.has(entry.name)) {
          console.error(`Skipping excluded directory: ${relativePath}`);
          continue;
        }

        // Check against loaded ignore patterns
        if (this.shouldIgnorePath(relativePath)) {
          console.error(`Skipping ignored path: ${relativePath}`);
          continue;
        }

        if (entry.isDirectory()) {
          const subFiles = await this.discoverFiles(fullPath, relativeTo);
          files.push(...subFiles);
        } else if (entry.isFile() && this.shouldIndexFile(entry.name)) {
          files.push(relativePath);
        }
      }
    } catch (error) {
      console.error(`Error discovering files in ${dirPath}:`, error);
    }

    return files;
  }

  /**
   * Check if file content appears to be binary
   */
  private isBinaryContent(content: string): boolean {
    // Check for null bytes or high concentration of non-printable characters
    const nonPrintableCount = (content.match(/[\x00-\x08\x0B\x0C\x0E-\x1F]/g) || []).length;
    const ratio = nonPrintableCount / content.length;
    return ratio > 0.1 || content.includes('\x00');
  }

  /**
   * Read file contents with size limit check
   */
  private readFileContents(relativePath: string): string | null {
    try {
      const fullPath = path.join(this.workspacePath, relativePath);
      const stats = fs.statSync(fullPath);

      if (stats.size > MAX_FILE_SIZE) {
        console.error(`Skipping large file: ${relativePath} (${(stats.size / 1024 / 1024).toFixed(2)}MB)`);
        return null;
      }

      const content = fs.readFileSync(fullPath, 'utf-8');

      // Check for binary content
      if (this.isBinaryContent(content)) {
        console.error(`Skipping binary file: ${relativePath}`);
        return null;
      }

      return content;
    } catch (error) {
      console.error(`Error reading file ${relativePath}:`, error);
      return null;
    }
  }

  // ==========================================================================
  // Cache Management
  // ==========================================================================

  /**
   * Get cached search results if valid
   */
  private getCachedSearch(cacheKey: string): SearchResult[] | null {
    const entry = this.searchCache.get(cacheKey);
    if (entry && Date.now() - entry.timestamp < CACHE_TTL_MS) {
      return entry.data;
    }
    // Remove stale entry
    if (entry) {
      this.searchCache.delete(cacheKey);
    }
    return null;
  }

  /**
   * Cache search results with LRU eviction
   */
  private setCachedSearch(cacheKey: string, results: SearchResult[]): void {
    // LRU eviction if cache is full
    if (this.searchCache.size >= this.maxCacheSize) {
      const oldestKey = this.searchCache.keys().next().value;
      if (oldestKey) {
        this.searchCache.delete(oldestKey);
      }
    }
    this.searchCache.set(cacheKey, {
      data: results,
      timestamp: Date.now(),
    });
  }

  /**
   * Clear the search cache
   */
  clearCache(): void {
    this.searchCache.clear();
  }

  // ==========================================================================
  // Core Operations
  // ==========================================================================

  /**
   * Index the workspace directory using DirectContext SDK
   */
  async indexWorkspace(): Promise<void> {
    console.error(`Indexing workspace: ${this.workspacePath}`);
    console.error(`API URL: ${process.env.AUGMENT_API_URL || '(default)'}`);
    console.error(`API Token: ${process.env.AUGMENT_API_TOKEN ? '(set)' : '(NOT SET)'}`);

    const context = await this.ensureInitialized();

    // Discover all indexable files
    const filePaths = await this.discoverFiles(this.workspacePath);
    console.error(`Found ${filePaths.length} files to index`);

    if (filePaths.length === 0) {
      console.error('No indexable files found');
      return;
    }

    // Log all discovered files for debugging
    console.error('Files to index:');
    for (const fp of filePaths) {
      console.error(`  - ${fp}`);
    }

    // Read file contents and prepare for indexing
    const files: Array<{ path: string; contents: string }> = [];
    let skippedCount = 0;
    for (const relativePath of filePaths) {
      const contents = this.readFileContents(relativePath);
      if (contents !== null) {
        files.push({ path: relativePath, contents });
      } else {
        skippedCount++;
      }
    }

    console.error(`Prepared ${files.length} files for indexing (skipped ${skippedCount})`);

    if (files.length === 0) {
      console.error('No files to index after filtering');
      return;
    }

    // Add files to index in batches with error handling
    const BATCH_SIZE = 10; // Reduced batch size for better error isolation
    let successCount = 0;
    let errorCount = 0;

    for (let i = 0; i < files.length; i += BATCH_SIZE) {
      const batch = files.slice(i, i + BATCH_SIZE);
      const batchNum = Math.floor(i / BATCH_SIZE) + 1;
      const totalBatches = Math.ceil(files.length / BATCH_SIZE);

      console.error(`\nIndexing batch ${batchNum}/${totalBatches}:`);
      for (const file of batch) {
        console.error(`  - ${file.path} (${file.contents.length} chars)`);
      }

      try {
        // Don't wait for indexing on intermediate batches
        const isLastBatch = i + BATCH_SIZE >= files.length;
        await context.addToIndex(batch, { waitForIndexing: isLastBatch });
        successCount += batch.length;
        console.error(`  ✓ Batch ${batchNum} indexed successfully`);
      } catch (error) {
        errorCount += batch.length;
        console.error(`  ✗ Batch ${batchNum} failed:`, error);

        // Try indexing files individually to isolate the problematic file
        console.error(`  Attempting individual file indexing for batch ${batchNum}...`);
        for (const file of batch) {
          try {
            await context.addToIndex([file], { waitForIndexing: false });
            successCount++;
            console.error(`    ✓ ${file.path}`);
          } catch (fileError) {
            console.error(`    ✗ ${file.path} FAILED:`, fileError);
            // Log file content preview for debugging
            const preview = file.contents.substring(0, 200).replace(/\n/g, '\\n');
            console.error(`      Content preview: ${preview}...`);
          }
        }
      }
    }

    console.error(`\nIndexing complete: ${successCount} succeeded, ${errorCount} had errors`);

    // Save state after indexing (even if some files failed)
    if (successCount > 0) {
      await this.saveState();
      console.error('Context state saved');
    }

    // Clear cache after reindexing
    this.clearCache();
    console.error('Workspace indexing finished');
  }

  /**
   * Perform semantic search using DirectContext SDK
   */
  async semanticSearch(query: string, topK: number = 10): Promise<SearchResult[]> {
    // Check cache first
    const cacheKey = `${query}:${topK}`;
    const cached = this.getCachedSearch(cacheKey);
    if (cached) {
      console.error(`[semanticSearch] Cache hit for query: ${query}`);
      return cached;
    }

    const context = await this.ensureInitialized();

    try {
      console.error(`[semanticSearch] Searching for: ${query}`);

      // Use the SDK's search method
      const formattedResults = await context.search(query, {
        maxOutputLength: topK * 2000, // Approximate output length based on topK
      });

      console.error(`[semanticSearch] Raw results length: ${formattedResults?.length || 0}`);
      console.error(`[semanticSearch] Raw results preview: ${formattedResults?.substring(0, 200) || '(empty)'}`);

      // Parse the formatted results into SearchResult objects
      const searchResults = this.parseFormattedResults(formattedResults, topK);

      console.error(`[semanticSearch] Parsed ${searchResults.length} results`);

      // Cache results
      this.setCachedSearch(cacheKey, searchResults);
      return searchResults;
    } catch (error) {
      console.error('Search failed:', error);
      return [];
    }
  }

  /**
   * Perform AI-powered search and ask using DirectContext SDK
   *
   * This method combines semantic search with an LLM call to answer questions
   * about the codebase. It uses Augment's backend LLM API.
   *
   * @param searchQuery - The semantic search query to find relevant code
   * @param prompt - Optional prompt to ask the LLM about the search results
   * @returns The LLM's response as a string
   * @throws Error if the API call fails or authentication is invalid
   */
  async searchAndAsk(searchQuery: string, prompt?: string): Promise<string> {
    const context = await this.ensureInitialized();

    try {
      console.error(`[searchAndAsk] Searching for: ${searchQuery}`);
      console.error(`[searchAndAsk] Prompt: ${prompt?.substring(0, 100) || '(using search query)'}`);

      // Use the SDK's searchAndAsk method
      const response = await context.searchAndAsk(searchQuery, prompt);

      console.error(`[searchAndAsk] Response length: ${response?.length || 0}`);

      return response;
    } catch (error) {
      console.error('[searchAndAsk] Failed:', error);
      throw error;
    }
  }

  /**
   * Parse the formatted search results from DirectContext into SearchResult objects
   *
   * The SDK returns results in this format:
   * ```
   * The following code sections were retrieved:
   * Path: src/file.ts
   * ...
   *     1  code line 1
   *     2  code line 2
   * ...
   * Path: src/other.ts
   * ...
   * ```
   */
  private parseFormattedResults(formattedResults: string, topK: number): SearchResult[] {
    const results: SearchResult[] = [];

    if (!formattedResults || formattedResults.trim() === '') {
      return results;
    }

    // Split by "Path:" prefix to get individual file blocks
    const pathBlocks = formattedResults.split(/(?=^Path:\s*)/m).filter(block => block.trim());

    for (const block of pathBlocks) {
      if (results.length >= topK) break;

      // Extract file path from "Path: filepath" line
      const pathMatch = block.match(/^Path:\s*(.+?)(?:\s*\n|$)/);
      if (!pathMatch) continue;

      const filePath = pathMatch[1].trim();

      // Extract code content (everything after the Path: line)
      const contentStart = block.indexOf('\n');
      if (contentStart === -1) continue;

      let content = block.substring(contentStart + 1).trim();

      // Remove the "..." markers that indicate truncation
      content = content.replace(/^\.\.\.\s*$/gm, '').trim();

      // Remove line number prefixes (e.g., "   30  code" -> "code")
      const lines: number[] = [];
      const cleanedLines = content.split('\n').map(line => {
        const lineNumMatch = line.match(/^\s*(\d+)\s{2}(.*)$/);
        if (lineNumMatch) {
          lines.push(parseInt(lineNumMatch[1], 10));
          return lineNumMatch[2];
        }
        return line;
      });
      content = cleanedLines.join('\n').trim();

      // Determine line range
      const lineRange = lines.length > 0
        ? `${Math.min(...lines)}-${Math.max(...lines)}`
        : undefined;

      if (content) {
        results.push({
          path: filePath.replace(/\\/g, '/'), // Normalize path separators
          content,
          lines: lineRange,
          relevanceScore: 1 - (results.length / topK), // Approximate relevance based on order
        });
      }
    }

    return results;
  }

  // ==========================================================================
  // File Operations with Security
  // ==========================================================================

  /**
   * Validate file path to prevent path traversal attacks
   */
  private validateFilePath(filePath: string): string {
    // Normalize the path
    const normalized = path.normalize(filePath);

    // Reject absolute paths (must be relative to workspace)
    if (path.isAbsolute(normalized)) {
      throw new Error(`Invalid path: absolute paths not allowed. Use paths relative to workspace.`);
    }

    // Reject path traversal attempts
    if (normalized.startsWith('..') || normalized.includes(`..${path.sep}`)) {
      throw new Error(`Invalid path: path traversal not allowed.`);
    }

    // Build full path safely
    const fullPath = path.resolve(this.workspacePath, normalized);

    // Ensure the resolved path is still within workspace
    if (!fullPath.startsWith(path.resolve(this.workspacePath))) {
      throw new Error(`Invalid path: path must be within workspace.`);
    }

    return fullPath;
  }

  /**
   * Get file contents with security checks
   */
  async getFile(filePath: string): Promise<string> {
    const fullPath = this.validateFilePath(filePath);

    if (!fs.existsSync(fullPath)) {
      throw new Error(`File not found: ${filePath}`);
    }

    // Check file size
    const stats = fs.statSync(fullPath);
    if (stats.size > MAX_FILE_SIZE) {
      throw new Error(`File too large: ${filePath} (${(stats.size / 1024 / 1024).toFixed(2)}MB > ${MAX_FILE_SIZE / 1024 / 1024}MB limit)`);
    }

    return fs.readFileSync(fullPath, 'utf-8');
  }

  // ==========================================================================
  // Token Estimation Utilities
  // ==========================================================================

  /**
   * Estimate token count for a string (conservative estimate)
   */
  private estimateTokens(text: string): number {
    return Math.ceil(text.length / CHARS_PER_TOKEN);
  }

  /**
   * Detect the type of code in a snippet
   */
  private detectCodeType(content: string): string {
    const trimmed = content.trim();

    // Common patterns
    if (/^(export\s+)?(async\s+)?function\s+\w+/.test(trimmed)) return 'function';
    if (/^(export\s+)?(abstract\s+)?class\s+\w+/.test(trimmed)) return 'class';
    if (/^(export\s+)?interface\s+\w+/.test(trimmed)) return 'interface';
    if (/^(export\s+)?type\s+\w+/.test(trimmed)) return 'type';
    if (/^(export\s+)?const\s+\w+/.test(trimmed)) return 'constant';
    if (/^import\s+/.test(trimmed)) return 'import';
    if (/^(export\s+)?enum\s+\w+/.test(trimmed)) return 'enum';
    if (/^\s*\/\*\*/.test(trimmed)) return 'documentation';
    if (/^(describe|it|test)\s*\(/.test(trimmed)) return 'test';

    return 'code';
  }

  /**
   * Generate a summary for a file based on its path and content patterns
   */
  private generateFileSummary(filePath: string, snippets: SnippetInfo[]): string {
    const ext = path.extname(filePath);
    const basename = path.basename(filePath, ext);
    const dirname = path.dirname(filePath);

    // Analyze code types in snippets
    const codeTypes = snippets.map(s => s.codeType).filter(Boolean);
    const uniqueTypes = [...new Set(codeTypes)];

    // Generate contextual summary
    let summary = '';

    // Infer purpose from path
    if (dirname.includes('test') || basename.includes('.test') || basename.includes('.spec')) {
      summary = `Test file for ${basename.replace(/\.(test|spec)$/, '')}`;
    } else if (dirname.includes('types') || basename.includes('types')) {
      summary = 'Type definitions';
    } else if (dirname.includes('utils') || basename.includes('util')) {
      summary = 'Utility functions';
    } else if (dirname.includes('components')) {
      summary = `UI component: ${basename}`;
    } else if (dirname.includes('hooks')) {
      summary = `React hook: ${basename}`;
    } else if (dirname.includes('api') || dirname.includes('routes')) {
      summary = `API endpoint: ${basename}`;
    } else if (basename === 'index') {
      summary = `Entry point for ${dirname}`;
    } else {
      summary = `${basename} module`;
    }

    // Add code type info
    if (uniqueTypes.length > 0) {
      summary += ` (contains: ${uniqueTypes.slice(0, 3).join(', ')})`;
    }

    return summary;
  }

  // ==========================================================================
  // Enhanced Prompt Context Engine
  // ==========================================================================

  /**
   * Find related files based on imports and references
   */
  private async findRelatedFiles(filePath: string, existingPaths: Set<string>): Promise<string[]> {
    try {
      const content = await this.getFile(filePath);
      const relatedFiles: string[] = [];

      // Extract imports (TypeScript/JavaScript)
      const importRegex = /(?:import|from)\s+['"]([^'"]+)['"]/g;
      let match;
      while ((match = importRegex.exec(content)) !== null) {
        const importPath = match[1];
        // Skip node_modules imports
        if (!importPath.startsWith('.')) continue;

        // Resolve the import path
        const dir = path.dirname(filePath);
        let resolvedPath = path.join(dir, importPath);

        // Try common extensions
        const extensions = ['.ts', '.tsx', '.js', '.jsx', ''];
        for (const ext of extensions) {
          const testPath = resolvedPath + ext;
          if (!existingPaths.has(testPath)) {
            try {
              await this.getFile(testPath);
              relatedFiles.push(testPath);
              break;
            } catch {
              // File doesn't exist with this extension
            }
          }
        }
      }

      return relatedFiles.slice(0, 3); // Limit related files
    } catch {
      return [];
    }
  }

  /**
   * Smart snippet extraction - get the most relevant parts of content
   */
  private extractSmartSnippet(content: string, maxTokens: number): string {
    const lines = content.split('\n');

    // If content fits, return as-is
    if (this.estimateTokens(content) <= maxTokens) {
      return content;
    }

    // Priority: function/class definitions, then imports, then other
    const priorityLines: { line: string; index: number; priority: number }[] = [];

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      let priority = 0;

      // High priority: function/class definitions
      if (/^(export\s+)?(async\s+)?function\s+\w+/.test(line.trim())) priority = 10;
      else if (/^(export\s+)?(abstract\s+)?class\s+\w+/.test(line.trim())) priority = 10;
      else if (/^(export\s+)?interface\s+\w+/.test(line.trim())) priority = 9;
      else if (/^(export\s+)?type\s+\w+/.test(line.trim())) priority = 8;
      // Medium priority: exports and constants
      else if (/^export\s+(const|let|var)\s+/.test(line.trim())) priority = 7;
      // Lower priority: imports (useful for context)
      else if (/^import\s+/.test(line.trim())) priority = 5;
      // Documentation comments
      else if (/^\s*\/\*\*/.test(line) || /^\s*\*/.test(line)) priority = 4;
      // Regular code
      else if (line.trim().length > 0) priority = 1;

      priorityLines.push({ line, index: i, priority });
    }

    // Sort by priority (descending) then by original order
    priorityLines.sort((a, b) => {
      if (b.priority !== a.priority) return b.priority - a.priority;
      return a.index - b.index;
    });

    // Build snippet within token budget
    const selectedLines: { line: string; index: number }[] = [];
    let tokenCount = 0;

    for (const { line, index, priority } of priorityLines) {
      const lineTokens = this.estimateTokens(line + '\n');
      if (tokenCount + lineTokens > maxTokens) break;
      selectedLines.push({ line, index });
      tokenCount += lineTokens;
    }

    // Sort by original order and join
    selectedLines.sort((a, b) => a.index - b.index);

    // Add ellipsis indicators for gaps
    let result = '';
    let lastIndex = -1;
    for (const { line, index } of selectedLines) {
      if (lastIndex !== -1 && index > lastIndex + 1) {
        result += '\n// ... (lines omitted) ...\n';
      }
      result += line + '\n';
      lastIndex = index;
    }

    return result.trim();
  }

  /**
   * Get enhanced context bundle for prompt enhancement
   * This is the primary method for Layer 2 - Context Service
   */
  async getContextForPrompt(query: string, options?: ContextOptions): Promise<ContextBundle>;
  async getContextForPrompt(query: string, maxFiles?: number): Promise<ContextBundle>;
  async getContextForPrompt(
    query: string,
    optionsOrMaxFiles?: ContextOptions | number
  ): Promise<ContextBundle> {
    const startTime = Date.now();

    // Parse options
    const options: ContextOptions = typeof optionsOrMaxFiles === 'number'
      ? { maxFiles: optionsOrMaxFiles }
      : optionsOrMaxFiles || {};

    const {
      maxFiles = 5,
      tokenBudget = DEFAULT_TOKEN_BUDGET,
      includeRelated = true,
      minRelevance = 0.3,
      includeSummaries = true,
    } = options;

    // Perform semantic search (get more results than needed for filtering)
    const searchResults = await this.semanticSearch(query, maxFiles * 3);

    // Filter by minimum relevance
    const relevantResults = searchResults.filter(
      r => (r.relevanceScore || 0) >= minRelevance
    );

    // Deduplicate and group by file path
    const fileMap = new Map<string, SearchResult[]>();
    for (const result of relevantResults) {
      if (!fileMap.has(result.path)) {
        fileMap.set(result.path, []);
      }
      fileMap.get(result.path)!.push(result);
    }

    // Calculate file-level relevance (max of snippet relevances)
    const fileRelevance = new Map<string, number>();
    for (const [filePath, results] of fileMap) {
      const maxRelevance = Math.max(...results.map(r => r.relevanceScore || 0));
      fileRelevance.set(filePath, maxRelevance);
    }

    // Sort files by relevance and take top files
    const sortedFiles = Array.from(fileMap.entries())
      .sort((a, b) => (fileRelevance.get(b[0]) || 0) - (fileRelevance.get(a[0]) || 0))
      .slice(0, maxFiles);

    // Track token usage
    let totalTokens = 0;
    let truncated = false;
    const existingPaths = new Set(sortedFiles.map(([p]) => p));

    // Build enhanced file contexts
    const files: FileContext[] = [];

    for (const [filePath, results] of sortedFiles) {
      // Calculate token budget for this file
      const remainingBudget = tokenBudget - totalTokens;
      const perFileBudget = Math.floor(remainingBudget / (maxFiles - files.length));

      if (perFileBudget < 100) {
        truncated = true;
        break;
      }

      // Build snippets with smart extraction
      const snippets: SnippetInfo[] = [];
      let fileTokens = 0;

      for (const result of results) {
        const snippetBudget = Math.floor(perFileBudget / results.length);
        const smartContent = this.extractSmartSnippet(result.content, snippetBudget);
        const tokenCount = this.estimateTokens(smartContent);

        if (fileTokens + tokenCount > perFileBudget) {
          truncated = true;
          break;
        }

        snippets.push({
          text: smartContent,
          lines: result.lines || 'unknown',
          relevance: result.relevanceScore || 0,
          tokenCount,
          codeType: this.detectCodeType(smartContent),
        });

        fileTokens += tokenCount;
      }

      // Find related files if enabled
      let relatedFiles: string[] | undefined;
      if (includeRelated) {
        relatedFiles = await this.findRelatedFiles(filePath, existingPaths);
        relatedFiles.forEach(p => existingPaths.add(p));
      }

      // Generate file summary if enabled
      const summary = includeSummaries
        ? this.generateFileSummary(filePath, snippets)
        : '';

      files.push({
        path: filePath,
        extension: path.extname(filePath),
        summary,
        relevance: fileRelevance.get(filePath) || 0,
        tokenCount: fileTokens,
        snippets,
        relatedFiles: relatedFiles?.length ? relatedFiles : undefined,
      });

      totalTokens += fileTokens;
    }

    // Generate intelligent hints
    const hints = this.generateContextHints(query, files, searchResults.length);

    // Build context summary
    const summary = this.generateContextSummary(query, files);

    const searchTimeMs = Date.now() - startTime;

    return {
      summary,
      query,
      files,
      hints,
      metadata: {
        totalFiles: files.length,
        totalSnippets: files.reduce((sum, f) => sum + f.snippets.length, 0),
        totalTokens,
        tokenBudget,
        truncated,
        searchTimeMs,
      },
    };
  }

  /**
   * Generate intelligent hints based on the context
   */
  private generateContextHints(
    query: string,
    files: FileContext[],
    totalResults: number
  ): string[] {
    const hints: string[] = [];

    // File type distribution
    const extensions = new Map<string, number>();
    for (const file of files) {
      const ext = file.extension || 'unknown';
      extensions.set(ext, (extensions.get(ext) || 0) + 1);
    }
    if (extensions.size > 0) {
      const extList = Array.from(extensions.entries())
        .map(([ext, count]) => `${ext} (${count})`)
        .join(', ');
      hints.push(`File types: ${extList}`);
    }

    // Code type distribution
    const codeTypes = new Map<string, number>();
    for (const file of files) {
      for (const snippet of file.snippets) {
        if (snippet.codeType) {
          codeTypes.set(snippet.codeType, (codeTypes.get(snippet.codeType) || 0) + 1);
        }
      }
    }
    if (codeTypes.size > 0) {
      const typeList = Array.from(codeTypes.entries())
        .sort((a, b) => b[1] - a[1])
        .slice(0, 4)
        .map(([type, count]) => `${type} (${count})`)
        .join(', ');
      hints.push(`Code patterns: ${typeList}`);
    }

    // Related files hint
    const relatedFiles = files.flatMap(f => f.relatedFiles || []);
    if (relatedFiles.length > 0) {
      hints.push(`Related files to consider: ${relatedFiles.slice(0, 3).join(', ')}`);
    }

    // Coverage hint
    if (totalResults > files.length) {
      hints.push(`Showing ${files.length} of ${totalResults} matching files`);
    }

    // High relevance hint
    const highRelevanceFiles = files.filter(f => f.relevance > 0.7);
    if (highRelevanceFiles.length > 0) {
      hints.push(`Highly relevant: ${highRelevanceFiles.map(f => path.basename(f.path)).join(', ')}`);
    }

    return hints;
  }

  /**
   * Generate a high-level summary of the context
   */
  private generateContextSummary(query: string, files: FileContext[]): string {
    if (files.length === 0) {
      return `No relevant code found for: "${query}"`;
    }

    // Get the most common directory
    const dirs = files.map(f => path.dirname(f.path));
    const dirCounts = new Map<string, number>();
    for (const dir of dirs) {
      dirCounts.set(dir, (dirCounts.get(dir) || 0) + 1);
    }
    const topDir = Array.from(dirCounts.entries())
      .sort((a, b) => b[1] - a[1])[0]?.[0] || '';

    // Get the dominant code types
    const allCodeTypes = files.flatMap(f => f.snippets.map(s => s.codeType)).filter(Boolean);
    const dominantType = allCodeTypes.length > 0
      ? allCodeTypes.sort((a, b) =>
          allCodeTypes.filter(t => t === b).length - allCodeTypes.filter(t => t === a).length
        )[0]
      : 'code';

    return `Context for "${query}": ${files.length} files from ${topDir || 'multiple directories'}, primarily containing ${dominantType} definitions`;
  }
}

