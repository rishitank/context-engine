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
    /// Constructs a JSON-RPC progress notification containing the provided token, progress value, optional total, and optional message.
    ///
    /// # Examples
    ///
    /// ```
    /// let note = ProgressNotification::new(
    ///     ProgressToken::String("op-1".into()),
    ///     50,
    ///     Some(100),
    ///     Some("in progress".into()),
    /// );
    /// assert_eq!(note.jsonrpc, "2.0");
    /// assert_eq!(note.method, "notifications/progress");
    /// assert_eq!(note.params.progress, 50);
    /// assert_eq!(note.params.total, Some(100));
    /// assert_eq!(note.params.message.as_deref(), Some("in progress"));
    /// ```
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
    /// Constructs a ProgressReporter bound to a progress token, a sender channel, and an optional total.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::mpsc;
    /// use crate::mcp::progress::{ProgressReporter, ProgressToken};
    ///
    /// let (tx, _rx) = mpsc::channel(1);
    /// let reporter = ProgressReporter::new(ProgressToken::Number(1), tx, Some(100));
    /// ```
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

    /// Send a progress notification for this reporter.
    ///
    /// The optional `message`, if provided, is included in the notification. Send failures are ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// # use futures::executor::block_on;
    /// # use crate::mcp::progress::{ProgressManager, ProgressToken};
    /// let manager = ProgressManager::new();
    /// let reporter = manager.create_reporter(Some(100));
    /// block_on(async {
    ///     reporter.report(42, Some("halfway")).await;
    /// });
    /// ```
    pub async fn report(&self, progress: u64, message: Option<&str>) {
        let notification = ProgressNotification::new(
            self.token.clone(),
            progress,
            self.total,
            message.map(String::from),
        );
        let _ = self.sender.send(notification).await;
    }

    /// Converts a percentage into an absolute progress value (using the reporter's `total` when present) and emits that progress notification.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tokio::sync::mpsc;
    /// # use crate::mcp::progress::{ProgressManager};
    /// # #[tokio::test]
    /// # async fn example_report_percent() {
    /// let manager = ProgressManager::new();
    /// let reporter = manager.create_reporter(Some(200));
    /// reporter.report_percent(50, Some("Halfway")).await;
    /// # }
    /// ```
    pub async fn report_percent(&self, percent: u64, message: Option<&str>) {
        let progress = if let Some(total) = self.total {
            (percent * total) / 100
        } else {
            percent
        };
        self.report(progress, message).await;
    }

    /// Report completion for this reporter by sending a notification with progress set to the reporter's total, if one is configured.
    ///
    /// If the reporter has no configured total, no notification is sent.
    ///
    /// # Parameters
    ///
    /// - `message`: Optional message to include with the completion notification.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::mpsc;
    /// use crate::mcp::progress::{ProgressReporter, ProgressToken};
    ///
    /// // Create a reporter with a total of 100 and send completion.
    /// let rt = tokio::runtime::Runtime::new().unwrap();
    /// let (tx, _rx) = mpsc::channel(10);
    /// let reporter = ProgressReporter::new(ProgressToken::Number(1), tx, Some(100));
    /// rt.block_on(reporter.complete(Some("finished")));
    /// ```
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
    /// Creates a new ProgressManager configured to emit progress notifications.
    ///
    /// # Examples
    ///
    /// ```
    /// let mgr = ProgressManager::new();
    /// // obtain a receiver to consume notifications
    /// let _recv = mgr.receiver();
    /// ```
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(100);
        Self {
            sender,
            receiver: Arc::new(tokio::sync::Mutex::new(receiver)),
            next_id: std::sync::atomic::AtomicI64::new(1),
        }
    }

    /// Creates a new ProgressReporter that uses a generated numeric token.
    ///
    /// The generated token is a sequential numeric identifier unique to this ProgressManager instance.
    ///
    /// # Parameters
    ///
    /// - `total`: Optional total number of work units for the operation; if provided, percentage-based reporting
    ///   will be computed against this value.
    ///
    /// # Returns
    ///
    /// A `ProgressReporter` bound to this manager's sender, using a newly generated numeric `ProgressToken`.
    ///
    /// # Examples
    ///
    /// ```
    /// let manager = ProgressManager::new();
    /// let reporter = manager.create_reporter(Some(100));
    /// // `reporter` can now be used to emit progress updates.
    /// ```
    pub fn create_reporter(&self, total: Option<u64>) -> ProgressReporter {
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let token = ProgressToken::Number(id);
        ProgressReporter::new(token, self.sender.clone(), total)
    }

    /// Creates a ProgressReporter bound to the given token and optional total.
    ///
    /// The returned reporter will send progress notifications tagged with `token`
    /// using the manager's internal channel.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::mcp::progress::{ProgressManager, ProgressToken};
    ///
    /// let manager = ProgressManager::new();
    /// let reporter = manager.create_reporter_with_token(ProgressToken::String("op".into()), Some(100));
    /// ```
    pub fn create_reporter_with_token(
        &self,
        token: ProgressToken,
        total: Option<u64>,
    ) -> ProgressReporter {
        ProgressReporter::new(token, self.sender.clone(), total)
    }

    /// Returns a clone of the shared receiver handle for progress notifications.
    ///
    /// The returned `Arc<tokio::sync::Mutex<mpsc::Receiver<ProgressNotification>>>` can be cloned and used by consumers to lock and receive progress notifications.
    ///
    /// # Examples
    ///
    /// ```
    /// let manager = ProgressManager::new();
    /// let rx = manager.receiver();
    /// // `rx` is a clone of the manager's shared receiver handle
    /// assert!(Arc::strong_count(&rx) >= 1);
    /// ```
    pub fn receiver(&self) -> Arc<tokio::sync::Mutex<mpsc::Receiver<ProgressNotification>>> {
        self.receiver.clone()
    }
}

impl Default for ProgressManager {
    /// Creates a ProgressManager initialized with its standard channel and token counter.
    ///
    /// # Examples
    ///
    /// ```
    /// let mgr = crate::mcp::progress::ProgressManager::default();
    /// let _recv = mgr.receiver();
    /// ```
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

    #[tokio::test]
    async fn test_progress_reporter_percent() {
        let (tx, mut rx) = mpsc::channel(10);
        let reporter = ProgressReporter::new(ProgressToken::Number(1), tx, Some(200));

        reporter.report_percent(50, Some("Half done")).await;

        let notification = rx.recv().await.unwrap();
        assert_eq!(notification.params.progress, 100); // 50% of 200
        assert_eq!(notification.params.total, Some(200));
    }

    #[tokio::test]
    async fn test_progress_reporter_complete() {
        let (tx, mut rx) = mpsc::channel(10);
        let reporter = ProgressReporter::new(ProgressToken::Number(2), tx, Some(100));

        reporter.complete(Some("Done!")).await;

        let notification = rx.recv().await.unwrap();
        assert_eq!(notification.params.progress, 100);
        assert_eq!(notification.params.message, Some("Done!".to_string()));
    }

    #[test]
    fn test_progress_token_serialization() {
        let token_str = ProgressToken::String("test-token".to_string());
        let token_num = ProgressToken::Number(42);

        let json_str = serde_json::to_string(&token_str).unwrap();
        let json_num = serde_json::to_string(&token_num).unwrap();

        assert_eq!(json_str, "\"test-token\"");
        assert_eq!(json_num, "42");

        let parsed_str: ProgressToken = serde_json::from_str(&json_str).unwrap();
        let parsed_num: ProgressToken = serde_json::from_str(&json_num).unwrap();

        assert_eq!(parsed_str, token_str);
        assert_eq!(parsed_num, token_num);
    }

    #[test]
    fn test_progress_notification_structure() {
        let notification = ProgressNotification::new(
            ProgressToken::String("op-1".to_string()),
            25,
            Some(100),
            Some("Processing...".to_string()),
        );

        assert_eq!(notification.jsonrpc, "2.0");
        assert_eq!(notification.method, "notifications/progress");
        assert_eq!(notification.params.progress, 25);
        assert_eq!(notification.params.total, Some(100));
    }

    #[test]
    fn test_progress_manager_create_reporter() {
        let manager = ProgressManager::new();

        let reporter1 = manager.create_reporter(Some(100));
        let reporter2 = manager.create_reporter(Some(200));

        // Reporters should have different tokens
        assert_ne!(reporter1.token, reporter2.token);
    }

    #[test]
    fn test_progress_manager_with_custom_token() {
        let manager = ProgressManager::new();
        let custom_token = ProgressToken::String("custom".to_string());

        let reporter = manager.create_reporter_with_token(custom_token.clone(), Some(50));
        assert_eq!(reporter.token, custom_token);
    }

    #[test]
    fn test_progress_params_serialization() {
        let params = ProgressParams {
            progress_token: ProgressToken::Number(1),
            progress: 50,
            total: Some(100),
            message: Some("Working...".to_string()),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"progressToken\":1"));
        assert!(json.contains("\"progress\":50"));
        assert!(json.contains("\"total\":100"));
        assert!(json.contains("\"message\":\"Working...\""));
    }
}
