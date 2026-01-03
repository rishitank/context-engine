//! Context service - main service for code context operations.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::error::{Error, Result};
use crate::sdk::{DirectContext, DirectContextOptions};
use crate::types::{IndexState, IndexStatus};

/// Patterns to ignore when indexing.
const DEFAULT_IGNORE_PATTERNS: &[&str] = &[
    "node_modules",
    ".git",
    ".svn",
    ".hg",
    "target",
    "dist",
    "build",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    "*.pyc",
    "*.pyo",
    "*.so",
    "*.dylib",
    "*.dll",
    "*.exe",
    "*.o",
    "*.a",
    "*.lib",
    ".DS_Store",
    "Thumbs.db",
    "*.log",
    "*.lock",
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "Cargo.lock",
];

/// Context service for managing code indexing and retrieval.
pub struct ContextService {
    config: Config,
    context: Arc<RwLock<Option<DirectContext>>>,
    workspace: PathBuf,
    ignore_patterns: HashSet<String>,
    state: Arc<RwLock<ServiceState>>,
}

/// Internal service state.
#[derive(Debug, Default)]
struct ServiceState {
    status: IndexState,
    file_count: usize,
    last_indexed: Option<String>,
    last_error: Option<String>,
}

impl ContextService {
    /// Create a new context service.
    pub async fn new(config: &Config) -> Result<Self> {
        let workspace = config.workspace.clone();
        let config = config.clone();

        // Initialize ignore patterns
        let mut ignore_patterns: HashSet<String> = DEFAULT_IGNORE_PATTERNS
            .iter()
            .map(|s| s.to_string())
            .collect();

        // Load .gitignore if present
        let gitignore_path = workspace.join(".gitignore");
        if gitignore_path.exists() {
            if let Ok(content) = fs::read_to_string(&gitignore_path).await {
                for line in content.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        ignore_patterns.insert(line.to_string());
                    }
                }
            }
        }

        Ok(Self {
            config,
            context: Arc::new(RwLock::new(None)),
            workspace,
            ignore_patterns,
            state: Arc::new(RwLock::new(ServiceState::default())),
        })
    }

    /// Initialize the context (lazy initialization).
    pub async fn initialize(&self) -> Result<()> {
        let mut context_guard = self.context.write().await;

        if context_guard.is_some() {
            return Ok(());
        }

        let options = DirectContextOptions {
            api_key: self.config.api_key.clone(),
            api_url: self.config.api_url.clone(),
            debug: self.config.debug,
            max_file_size: Some(self.config.max_file_size),
        };

        let context = DirectContext::create(options).await?;
        *context_guard = Some(context);

        info!(
            "Context service initialized for workspace: {:?}",
            self.workspace
        );
        Ok(())
    }

    /// Get the workspace path.
    pub fn workspace(&self) -> &Path {
        &self.workspace
    }

    /// Get the workspace path (alias for compatibility).
    pub fn workspace_path(&self) -> &Path {
        &self.workspace
    }

    /// Get the current index status.
    pub async fn status(&self) -> IndexStatus {
        let state = self.state.read().await;
        let context = self.context.read().await;

        let file_count = if let Some(ctx) = context.as_ref() {
            ctx.file_count().await
        } else {
            0
        };

        IndexStatus {
            workspace: self.workspace.display().to_string(),
            status: state.status,
            last_indexed: state.last_indexed.clone(),
            file_count,
            is_stale: false,
            last_error: state.last_error.clone(),
        }
    }

    /// Check if a path should be ignored.
    fn should_ignore(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.ignore_patterns {
            if pattern.contains('*') {
                // Simple glob matching
                let pattern = pattern.replace("*", "");
                if path_str.contains(&pattern) {
                    return true;
                }
            } else if path_str.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Perform semantic search.
    pub async fn search(&self, query: &str, max_tokens: Option<usize>) -> Result<String> {
        self.initialize().await?;

        let context = self.context.read().await;
        let ctx = context.as_ref().ok_or(Error::IndexNotInitialized)?;

        ctx.search(query, max_tokens).await
    }

    /// Index the entire workspace.
    pub async fn index_workspace(&self) -> Result<crate::types::IndexResult> {
        self.initialize().await?;

        // Update status to indexing
        {
            let mut state = self.state.write().await;
            state.status = IndexState::Indexing;
            state.last_error = None;
        }

        info!("Starting workspace indexing: {:?}", self.workspace);

        // Discover all files
        let files = self.discover_files(&self.workspace).await?;
        let file_count = files.len();

        info!("Discovered {} files to index", file_count);

        // Index files in batches
        let mut indexed = 0;
        let mut skipped = 0;
        let mut errors = Vec::new();
        let start_time = std::time::Instant::now();

        let context = self.context.read().await;
        let ctx = context.as_ref().ok_or(Error::IndexNotInitialized)?;

        // Collect files into batches
        const BATCH_SIZE: usize = 100;
        let mut batch: Vec<crate::types::File> = Vec::with_capacity(BATCH_SIZE);

        for file_path in files {
            let relative_path = file_path
                .strip_prefix(&self.workspace)
                .unwrap_or(&file_path)
                .to_string_lossy()
                .to_string();

            match fs::read_to_string(&file_path).await {
                Ok(contents) => {
                    // Check file size
                    if contents.len() > self.config.max_file_size {
                        debug!(
                            "Skipping large file: {} ({} bytes)",
                            relative_path,
                            contents.len()
                        );
                        skipped += 1;
                        continue;
                    }

                    batch.push(crate::types::File {
                        path: relative_path,
                        contents,
                    });

                    // Process batch when full
                    if batch.len() >= BATCH_SIZE {
                        match ctx.add_to_index(std::mem::take(&mut batch)).await {
                            Ok(result) => {
                                indexed += result.indexed;
                                skipped += result.skipped;
                                errors.extend(result.errors);
                            }
                            Err(e) => {
                                warn!("Batch indexing failed: {}", e);
                                errors.push(format!("Batch error: {}", e));
                            }
                        }
                    }
                }
                Err(e) => {
                    debug!("Failed to read {}: {}", relative_path, e);
                    skipped += 1;
                }
            }
        }

        // Process remaining files
        if !batch.is_empty() {
            match ctx.add_to_index(batch).await {
                Ok(result) => {
                    indexed += result.indexed;
                    skipped += result.skipped;
                    errors.extend(result.errors);
                }
                Err(e) => {
                    warn!("Final batch indexing failed: {}", e);
                    errors.push(format!("Batch error: {}", e));
                }
            }
        }

        let duration = start_time.elapsed().as_millis() as u64;

        // Update status
        {
            let mut state = self.state.write().await;
            state.status = IndexState::Idle;
            state.file_count = indexed;
            state.last_indexed = Some(chrono::Utc::now().to_rfc3339());
        }

        info!(
            "Indexing complete: {} indexed, {} skipped in {}ms",
            indexed, skipped, duration
        );

        Ok(crate::types::IndexResult {
            indexed,
            skipped,
            errors,
            duration,
        })
    }

    /// Discover all indexable files in a directory.
    async fn discover_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let mut dirs_to_visit = vec![dir.to_path_buf()];

        while let Some(current_dir) = dirs_to_visit.pop() {
            let mut entries = match fs::read_dir(&current_dir).await {
                Ok(e) => e,
                Err(e) => {
                    debug!("Cannot read directory {:?}: {}", current_dir, e);
                    continue;
                }
            };

            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                let relative = path.strip_prefix(&self.workspace).unwrap_or(&path);

                // Check if should be ignored
                if self.should_ignore(relative) {
                    debug!("Ignoring: {:?}", relative);
                    continue;
                }

                if path.is_dir() {
                    dirs_to_visit.push(path);
                } else if path.is_file() {
                    // Check if it's a text file we should index
                    if self.should_index_file(&path) {
                        files.push(path);
                    }
                }
            }
        }

        Ok(files)
    }

    /// Check if a file should be indexed based on its extension.
    fn should_index_file(&self, path: &Path) -> bool {
        const INDEXABLE_EXTENSIONS: &[&str] = &[
            "rs",
            "py",
            "js",
            "ts",
            "jsx",
            "tsx",
            "go",
            "java",
            "c",
            "cpp",
            "h",
            "hpp",
            "rb",
            "php",
            "swift",
            "kt",
            "scala",
            "cs",
            "fs",
            "clj",
            "ex",
            "exs",
            "hs",
            "ml",
            "mli",
            "lua",
            "r",
            "jl",
            "dart",
            "v",
            "sv",
            "vhd",
            "sql",
            "sh",
            "bash",
            "zsh",
            "fish",
            "ps1",
            "bat",
            "cmd",
            "html",
            "css",
            "scss",
            "sass",
            "less",
            "vue",
            "svelte",
            "json",
            "yaml",
            "yml",
            "toml",
            "xml",
            "md",
            "txt",
            "rst",
            "dockerfile",
            "makefile",
            "cmake",
            "gradle",
            "sbt",
            "tf",
            "hcl",
            "nix",
            "dhall",
        ];

        if let Some(ext) = path.extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            return INDEXABLE_EXTENSIONS.contains(&ext_lower.as_str());
        }

        // Check for extensionless files that should be indexed
        if let Some(name) = path.file_name() {
            let name_lower = name.to_string_lossy().to_lowercase();
            return matches!(
                name_lower.as_str(),
                "dockerfile"
                    | "makefile"
                    | "rakefile"
                    | "gemfile"
                    | "brewfile"
                    | ".gitignore"
                    | ".dockerignore"
            );
        }

        false
    }

    /// Clear the index.
    pub async fn clear(&self) {
        let context = self.context.read().await;
        if let Some(ctx) = context.as_ref() {
            if let Err(e) = ctx.clear().await {
                warn!("Failed to clear context: {}", e);
            }
        }

        let mut state = self.state.write().await;
        state.status = IndexState::Idle;
        state.file_count = 0;
        state.last_indexed = None;
        state.last_error = None;

        info!("Index cleared");
    }

    /// Bundle a prompt with relevant codebase context (no AI rewriting).
    ///
    /// This retrieves relevant code snippets based on the prompt and returns
    /// a structured bundle containing both the original prompt and the context.
    /// Use this when you want direct control over how the context is used.
    pub async fn bundle_prompt(
        &self,
        prompt: &str,
        token_budget: Option<usize>,
    ) -> Result<BundledPrompt> {
        self.initialize().await?;

        let budget = token_budget.unwrap_or(8000);

        // Retrieve relevant codebase context
        let context_result = self.search(prompt, Some(budget)).await?;

        Ok(BundledPrompt {
            original_prompt: prompt.to_string(),
            codebase_context: context_result,
            token_budget: budget,
        })
    }

    /// Enhance a prompt with codebase context using AI.
    ///
    /// This performs three steps:
    /// 1. Retrieves relevant codebase context based on the prompt
    /// 2. Bundles the context with the original prompt
    /// 3. Uses AI to create an enhanced, structured prompt
    pub async fn enhance_prompt(
        &self,
        prompt: &str,
        token_budget: Option<usize>,
    ) -> Result<String> {
        self.initialize().await?;

        let context = self.context.read().await;
        let ctx = context.as_ref().ok_or(Error::IndexNotInitialized)?;

        let budget = token_budget.unwrap_or(6000);

        // Step 1: Retrieve relevant codebase context
        let codebase_context = ctx.search(prompt, Some(budget)).await?;

        // Step 2: Build the enhancement prompt with actual context
        let enhancement_prompt = format!(
            r#"You are an AI prompt enhancement assistant with access to the user's codebase.

## Task
Transform the user's simple prompt into a detailed, actionable prompt that:
1. Incorporates relevant context from their codebase
2. References specific files, functions, or patterns found in the codebase
3. Provides clear objectives and requirements
4. Suggests implementation approaches based on existing code patterns

## User's Original Prompt
{prompt}

## Relevant Codebase Context
{codebase_context}

## Instructions
Based on the codebase context above, create an enhanced prompt that:
- References specific code locations (files, line numbers, function names)
- Identifies existing patterns the user should follow
- Highlights potential integration points
- Suggests tests or validation approaches based on existing test patterns
- Maintains the original intent while adding actionable detail

## Enhanced Prompt"#,
            prompt = prompt,
            codebase_context = codebase_context
        );

        // Step 3: Use AI to generate the enhanced prompt
        ctx.chat(&enhancement_prompt).await
    }
}

/// A prompt bundled with relevant codebase context.
#[derive(Debug, Clone)]
pub struct BundledPrompt {
    /// The original user prompt.
    pub original_prompt: String,
    /// Relevant codebase context retrieved via semantic search.
    pub codebase_context: String,
    /// The token budget used for context retrieval.
    pub token_budget: usize,
}

impl BundledPrompt {
    /// Format the bundled prompt as a single string.
    pub fn to_formatted_string(&self) -> String {
        format!(
            r#"# User Request
{prompt}

# Relevant Codebase Context
{context}"#,
            prompt = self.original_prompt,
            context = self.codebase_context
        )
    }

    /// Format with a custom system instruction.
    pub fn to_formatted_string_with_system(&self, system_instruction: &str) -> String {
        format!(
            r#"# System
{system}

# User Request
{prompt}

# Relevant Codebase Context
{context}"#,
            system = system_instruction,
            prompt = self.original_prompt,
            context = self.codebase_context
        )
    }
}
