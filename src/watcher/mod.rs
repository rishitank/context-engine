//! File watcher for automatic index updates.
//!
//! Uses the `notify` crate to watch for file system changes
//! and trigger index updates.

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::error::{Error, Result};

/// File change event.
#[derive(Debug, Clone)]
pub struct FileChange {
    /// Path to the changed file
    pub path: PathBuf,
    /// Type of change
    pub kind: ChangeKind,
}

/// Type of file change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Created,
    Modified,
    Deleted,
}

/// File watcher for monitoring workspace changes.
pub struct FileWatcher {
    workspace: PathBuf,
    watcher: Option<RecommendedWatcher>,
    pending_changes: Arc<RwLock<Vec<FileChange>>>,
    ignore_patterns: HashSet<String>,
    debounce_ms: u64,
}

impl FileWatcher {
    /// Create a new file watcher.
    pub fn new(workspace: PathBuf, debounce_ms: u64) -> Self {
        Self {
            workspace,
            watcher: None,
            pending_changes: Arc::new(RwLock::new(Vec::new())),
            ignore_patterns: HashSet::new(),
            debounce_ms,
        }
    }

    /// Add patterns to ignore.
    pub fn add_ignore_patterns(&mut self, patterns: impl IntoIterator<Item = String>) {
        self.ignore_patterns.extend(patterns);
    }

    /// Start watching for changes.
    pub async fn start(&mut self) -> Result<mpsc::Receiver<Vec<FileChange>>> {
        let (tx, rx) = mpsc::channel::<Vec<FileChange>>(100);
        let pending = self.pending_changes.clone();
        let debounce_ms = self.debounce_ms;

        // Create the watcher
        let pending_clone = pending.clone();
        let ignore_patterns = self.ignore_patterns.clone();

        let watcher = RecommendedWatcher::new(
            move |res: std::result::Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        for path in event.paths {
                            // Check if should ignore
                            let path_str = path.to_string_lossy();
                            let should_ignore =
                                ignore_patterns.iter().any(|p| path_str.contains(p));

                            if should_ignore {
                                continue;
                            }

                            let kind = match event.kind {
                                notify::EventKind::Create(_) => ChangeKind::Created,
                                notify::EventKind::Modify(_) => ChangeKind::Modified,
                                notify::EventKind::Remove(_) => ChangeKind::Deleted,
                                _ => continue,
                            };

                            let change = FileChange { path, kind };

                            // Add to pending (blocking)
                            if let Ok(mut pending) = pending_clone.try_write() {
                                pending.push(change);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Watch error: {:?}", e);
                    }
                }
            },
            Config::default(),
        )
        .map_err(|e| Error::Internal(format!("Failed to create watcher: {}", e)))?;

        self.watcher = Some(watcher);

        // Start watching the workspace
        if let Some(ref mut w) = self.watcher {
            w.watch(&self.workspace, RecursiveMode::Recursive)
                .map_err(|e| Error::Internal(format!("Failed to watch directory: {}", e)))?;
        }

        // Spawn debounce task
        let pending_for_debounce = pending.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(debounce_ms)).await;

                let changes: Vec<FileChange> = {
                    let mut pending = pending_for_debounce.write().await;
                    std::mem::take(&mut *pending)
                };

                if !changes.is_empty() {
                    debug!("Flushing {} file changes", changes.len());
                    if tx.send(changes).await.is_err() {
                        break;
                    }
                }
            }
        });

        info!("File watcher started for {:?}", self.workspace);
        Ok(rx)
    }

    /// Stop watching.
    pub fn stop(&mut self) {
        self.watcher = None;
        info!("File watcher stopped");
    }

    /// Get the number of pending changes.
    pub async fn pending_count(&self) -> usize {
        self.pending_changes.read().await.len()
    }
}
