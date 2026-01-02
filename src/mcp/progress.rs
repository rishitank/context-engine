//! MCP Progress Notifications
//!
//! Support for emitting progress updates during long-running operations.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Progress token for tracking operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum ProgressToken {
    String(String),
    Number(i64),
}

/// Progress notification params.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressParams {
    pub progress_token: ProgressToken,
    pub progress: u64,
    pub total: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Progress notification message.
#[derive(Debug, Clone, Serialize)]
pub struct ProgressNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: ProgressParams,
}

impl ProgressNotification {
    /// Create a new progress notification.
    pub fn new(
        token: ProgressToken,
        progress: u64,
        total: Option<u64>,
        message: Option<String>,
    ) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: "notifications/progress".to_string(),
            params: ProgressParams {
                progress_token: token,
                progress,
                total,
                message,
            },
        }
    }
}

/// Progress reporter for emitting updates.
#[derive(Clone)]
pub struct ProgressReporter {
    token: ProgressToken,
    sender: mpsc::Sender<ProgressNotification>,
    total: Option<u64>,
}

impl ProgressReporter {
    /// Create a new progress reporter.
    pub fn new(
        token: ProgressToken,
        sender: mpsc::Sender<ProgressNotification>,
        total: Option<u64>,
    ) -> Self {
        Self {
            token,
            sender,
            total,
        }
    }

    /// Report progress.
    pub async fn report(&self, progress: u64, message: Option<&str>) {
        let notification = ProgressNotification::new(
            self.token.clone(),
            progress,
            self.total,
            message.map(String::from),
        );
        let _ = self.sender.send(notification).await;
    }

    /// Report progress with percentage.
    pub async fn report_percent(&self, percent: u64, message: Option<&str>) {
        let progress = if let Some(total) = self.total {
            (percent * total) / 100
        } else {
            percent
        };
        self.report(progress, message).await;
    }

    /// Complete the progress.
    pub async fn complete(&self, message: Option<&str>) {
        if let Some(total) = self.total {
            self.report(total, message).await;
        }
    }
}

/// Progress manager for creating and tracking progress reporters.
pub struct ProgressManager {
    sender: mpsc::Sender<ProgressNotification>,
    receiver: Arc<tokio::sync::Mutex<mpsc::Receiver<ProgressNotification>>>,
    next_id: std::sync::atomic::AtomicI64,
}

impl ProgressManager {
    /// Create a new progress manager.
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(100);
        Self {
            sender,
            receiver: Arc::new(tokio::sync::Mutex::new(receiver)),
            next_id: std::sync::atomic::AtomicI64::new(1),
        }
    }

    /// Create a new progress reporter with a generated token.
    pub fn create_reporter(&self, total: Option<u64>) -> ProgressReporter {
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let token = ProgressToken::Number(id);
        ProgressReporter::new(token, self.sender.clone(), total)
    }

    /// Create a progress reporter with a specific token.
    pub fn create_reporter_with_token(
        &self,
        token: ProgressToken,
        total: Option<u64>,
    ) -> ProgressReporter {
        ProgressReporter::new(token, self.sender.clone(), total)
    }

    /// Get the receiver for progress notifications.
    pub fn receiver(&self) -> Arc<tokio::sync::Mutex<mpsc::Receiver<ProgressNotification>>> {
        self.receiver.clone()
    }
}

impl Default for ProgressManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_progress_reporter() {
        let (tx, mut rx) = mpsc::channel(10);
        let reporter =
            ProgressReporter::new(ProgressToken::String("test".to_string()), tx, Some(100));

        reporter.report(50, Some("Halfway")).await;

        let notification = rx.recv().await.unwrap();
        assert_eq!(notification.params.progress, 50);
        assert_eq!(notification.params.total, Some(100));
        assert_eq!(notification.params.message, Some("Halfway".to_string()));
    }
}
