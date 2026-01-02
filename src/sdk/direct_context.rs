//! DirectContext - Main context management class.
//!
//! This is the primary interface for indexing files and performing
//! semantic search against the Augment backend.

use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::error::{Error, Result};
use crate::sdk::api_client::ApiClient;
use crate::sdk::blob::{BlobNameCalculator, DEFAULT_MAX_BLOB_SIZE};
use crate::sdk::credentials::resolve_credentials;
use crate::sdk::types::*;

/// Maximum files per batch upload.
const MAX_BATCH_UPLOAD_SIZE: usize = 1000;

/// Maximum bytes per batch upload (2MB).
const MAX_BATCH_CONTENT_BYTES: usize = 2 * 1024 * 1024;

/// Checkpoint threshold (create checkpoint after this many changes).
const CHECKPOINT_THRESHOLD: usize = 1000;

/// DirectContext for managing file indexing and search.
#[derive(Debug)]
pub struct DirectContext {
    api_client: ApiClient,
    blob_calculator: BlobNameCalculator,
    state: Arc<RwLock<DirectContextState>>,
    debug: bool,
}

impl DirectContext {
    /// Create a new DirectContext with the given options.
    pub async fn create(options: DirectContextOptions) -> Result<Self> {
        let credentials = resolve_credentials(
            options.api_key.as_deref(),
            options.api_url.as_deref(),
        )
        .await?;

        let api_client = ApiClient::new(
            credentials.api_url,
            credentials.api_key,
            options.debug,
        )?;

        let max_file_size = options.max_file_size.unwrap_or(DEFAULT_MAX_BLOB_SIZE);
        let blob_calculator = BlobNameCalculator::new(max_file_size);

        Ok(Self {
            api_client,
            blob_calculator,
            state: Arc::new(RwLock::new(DirectContextState::default())),
            debug: options.debug,
        })
    }

    /// Add files to the index.
    pub async fn add_to_index(&self, files: Vec<crate::types::File>) -> Result<IndexingResult> {
        let mut indexed = 0;
        let mut skipped = 0;
        let mut errors = Vec::new();

        let mut state = self.state.write().await;
        let mut blobs_to_upload: Vec<BlobEntry> = Vec::new();
        let mut current_batch_size = 0;

        for file in files {
            // Calculate blob name
            let blob_name = match self.blob_calculator.calculate(&file.path, file.contents.as_bytes()) {
                Some(name) => name,
                None => {
                    skipped += 1;
                    if self.debug {
                        debug!("Skipping large file: {}", file.path);
                    }
                    continue;
                }
            };

            // Check if already indexed
            if let Some(existing) = state.blob_map.get(&file.path) {
                if existing == &blob_name {
                    skipped += 1;
                    continue;
                }
            }

            // Add to pending
            state.pending_added.push(BlobInfo {
                blob_name: blob_name.clone(),
                path: file.path.clone(),
            });
            state.client_blob_map.insert(file.path.clone(), blob_name.clone());

            // Prepare for upload
            let content_size = file.contents.len();
            if current_batch_size + content_size > MAX_BATCH_CONTENT_BYTES
                || blobs_to_upload.len() >= MAX_BATCH_UPLOAD_SIZE
            {
                // Upload current batch
                if !blobs_to_upload.is_empty() {
                    match self.upload_batch(&blobs_to_upload).await {
                        Ok(_) => indexed += blobs_to_upload.len(),
                        Err(e) => errors.push(format!("Batch upload failed: {}", e)),
                    }
                    blobs_to_upload.clear();
                    current_batch_size = 0;
                }
            }

            blobs_to_upload.push(BlobEntry {
                blob_name,
                path: file.path,
                content: file.contents,
            });
            current_batch_size += content_size;
        }

        // Upload remaining batch
        if !blobs_to_upload.is_empty() {
            match self.upload_batch(&blobs_to_upload).await {
                Ok(_) => indexed += blobs_to_upload.len(),
                Err(e) => errors.push(format!("Batch upload failed: {}", e)),
            }
        }

        // Create checkpoint if needed
        if state.pending_added.len() >= CHECKPOINT_THRESHOLD {
            drop(state); // Release lock before checkpoint
            self.create_checkpoint().await?;
        }

        Ok(IndexingResult {
            indexed,
            skipped,
            errors,
        })
    }

    /// Upload a batch of blobs.
    async fn upload_batch(&self, blobs: &[BlobEntry]) -> Result<()> {
        if blobs.is_empty() {
            return Ok(());
        }

        // First check which blobs are missing
        let blob_names: Vec<String> = blobs.iter().map(|b| b.blob_name.clone()).collect();
        let missing = self.api_client.find_missing(blob_names).await?;

        // Filter to only upload missing blobs
        let to_upload: Vec<BlobEntry> = blobs
            .iter()
            .filter(|b| {
                missing.unknown_memory_names.contains(&b.blob_name)
                    || missing.nonindexed_blob_names.contains(&b.blob_name)
            })
            .cloned()
            .collect();

        if !to_upload.is_empty() {
            self.api_client.batch_upload(to_upload).await?;
        }

        Ok(())
    }

    /// Create a checkpoint of the current state.
    async fn create_checkpoint(&self) -> Result<()> {
        let mut state = self.state.write().await;
        
        if state.pending_added.is_empty() && state.pending_deleted.is_empty() {
            return Ok(());
        }

        let response = self
            .api_client
            .checkpoint_blobs(
                state.checkpoint_id.clone(),
                std::mem::take(&mut state.pending_added),
                std::mem::take(&mut state.pending_deleted),
            )
            .await?;

        state.checkpoint_id = Some(response.new_checkpoint_id);

        // Update blob_map from client_blob_map
        state.blob_map = state.client_blob_map.clone();

        info!("Created checkpoint: {:?}", state.checkpoint_id);
        Ok(())
    }

    /// Perform semantic search against the indexed codebase.
    pub async fn search(&self, query: &str, max_output_length: Option<usize>) -> Result<String> {
        // Ensure we have a checkpoint
        self.create_checkpoint().await?;

        let state = self.state.read().await;

        if state.checkpoint_id.is_none() && state.blob_map.is_empty() {
            return Err(Error::IndexNotInitialized);
        }

        let blobs = Blobs {
            checkpoint_id: state.checkpoint_id.clone(),
            added_blobs: state.pending_added.clone(),
            deleted_blobs: state.pending_deleted.clone(),
        };

        drop(state); // Release lock before API call

        let response = self
            .api_client
            .agent_codebase_retrieval(query, blobs, max_output_length)
            .await?;

        Ok(response.formatted_retrieval)
    }

    /// Remove files from the index.
    pub async fn remove_from_index(&self, paths: Vec<String>) -> Result<usize> {
        let mut state = self.state.write().await;
        let mut removed = 0;

        for path in paths {
            if let Some(blob_name) = state.client_blob_map.remove(&path) {
                state.pending_deleted.push(blob_name);
                removed += 1;
            }
        }

        // Create checkpoint if needed
        if state.pending_deleted.len() >= CHECKPOINT_THRESHOLD {
            drop(state);
            self.create_checkpoint().await?;
        }

        Ok(removed)
    }

    /// Export the current state to a file.
    pub async fn export_to_file(&self, path: &Path) -> Result<()> {
        // Ensure checkpoint is current
        self.create_checkpoint().await?;

        let state = self.state.read().await;
        let json = serde_json::to_string_pretty(&*state)?;
        fs::write(path, json).await?;

        info!("Exported state to {:?}", path);
        Ok(())
    }

    /// Import state from a file.
    pub async fn import_from_file(&self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path).await?;
        let imported: DirectContextState = serde_json::from_str(&content)?;

        let mut state = self.state.write().await;
        *state = imported;

        info!("Imported state from {:?}", path);
        Ok(())
    }

    /// Get the current state (for debugging/inspection).
    pub async fn get_state(&self) -> DirectContextState {
        self.state.read().await.clone()
    }

    /// Get the number of indexed files.
    pub async fn file_count(&self) -> usize {
        self.state.read().await.client_blob_map.len()
    }

    /// Check if a file is indexed.
    pub async fn is_indexed(&self, path: &str) -> bool {
        self.state.read().await.client_blob_map.contains_key(path)
    }

    /// Clear all indexed files.
    pub async fn clear(&self) -> Result<()> {
        let mut state = self.state.write().await;

        // Collect blob names first to avoid borrow conflict
        let blob_names: Vec<String> = state.client_blob_map.values().cloned().collect();

        // Mark all files as deleted
        state.pending_deleted.extend(blob_names);

        state.client_blob_map.clear();
        state.blob_map.clear();

        drop(state);
        self.create_checkpoint().await?;

        info!("Cleared all indexed files");
        Ok(())
    }

    /// Chat with the AI using the indexed codebase context.
    pub async fn chat(&self, prompt: &str) -> Result<String> {
        // Ensure we have a checkpoint
        self.create_checkpoint().await?;

        let state = self.state.read().await;

        let blobs = Blobs {
            checkpoint_id: state.checkpoint_id.clone(),
            added_blobs: state.pending_added.clone(),
            deleted_blobs: state.pending_deleted.clone(),
        };

        drop(state); // Release lock before API call

        self.api_client.chat_stream(prompt, blobs).await
    }
}

