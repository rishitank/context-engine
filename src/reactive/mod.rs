//! Reactive review system.
//!
//! Session-based PR reviews with parallel execution and real-time updates.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::reviewer::{ReviewConfig, ReviewPipeline};
use crate::service::ContextService;
use crate::types::review::*;

/// A reactive review session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSession {
    /// Session ID
    pub id: String,
    /// Session status
    pub status: SessionStatus,
    /// PR or branch being reviewed
    pub target: String,
    /// Reviews in this session
    pub reviews: Vec<Review>,
    /// Session metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
}

/// Session status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Paused,
    Completed,
    Cancelled,
}

/// Reactive review manager.
pub struct ReactiveReviewManager {
    context_service: Arc<ContextService>,
    sessions: Arc<RwLock<HashMap<String, ReviewSession>>>,
    config: ReviewConfig,
}

impl ReactiveReviewManager {
    /// Create a new reactive review manager.
    pub fn new(context_service: Arc<ContextService>, config: ReviewConfig) -> Self {
        Self {
            context_service,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Start a new review session.
    pub async fn start_session(&self, target: String) -> Result<ReviewSession> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let session = ReviewSession {
            id: id.clone(),
            status: SessionStatus::Active,
            target,
            reviews: Vec::new(),
            metadata: HashMap::new(),
            created_at: now.clone(),
            updated_at: now,
        };

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(id.clone(), session.clone());
        }

        info!("Started review session: {}", id);
        Ok(session)
    }

    /// Get a session by ID.
    pub async fn get_session(&self, id: &str) -> Option<ReviewSession> {
        let sessions = self.sessions.read().await;
        sessions.get(id).cloned()
    }

    /// List all sessions.
    pub async fn list_sessions(&self, status: Option<SessionStatus>) -> Vec<ReviewSession> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .filter(|s| status.is_none_or(|st| s.status == st))
            .cloned()
            .collect()
    }

    /// Add a review to a session.
    pub async fn add_review(&self, session_id: &str, diff: &str) -> Result<Review> {
        let pipeline = ReviewPipeline::new(self.context_service.clone(), self.config.clone());
        let review = pipeline.review_diff(diff, None).await?;

        {
            let mut sessions = self.sessions.write().await;
            let session = sessions
                .get_mut(session_id)
                .ok_or_else(|| Error::ReviewSessionNotFound(session_id.to_string()))?;

            session.reviews.push(review.clone());
            session.updated_at = chrono::Utc::now().to_rfc3339();
        }

        Ok(review)
    }

    /// Complete a session.
    pub async fn complete_session(&self, id: &str) -> Result<ReviewSession> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(id)
            .ok_or_else(|| Error::ReviewSessionNotFound(id.to_string()))?;

        session.status = SessionStatus::Completed;
        session.updated_at = chrono::Utc::now().to_rfc3339();

        info!("Completed review session: {}", id);
        Ok(session.clone())
    }

    /// Cancel a session.
    pub async fn cancel_session(&self, id: &str) -> Result<ReviewSession> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(id)
            .ok_or_else(|| Error::ReviewSessionNotFound(id.to_string()))?;

        session.status = SessionStatus::Cancelled;
        session.updated_at = chrono::Utc::now().to_rfc3339();

        info!("Cancelled review session: {}", id);
        Ok(session.clone())
    }

    /// Get session statistics.
    pub async fn get_stats(&self) -> SessionStats {
        let sessions = self.sessions.read().await;

        let mut stats = SessionStats::default();
        for session in sessions.values() {
            match session.status {
                SessionStatus::Active => stats.active += 1,
                SessionStatus::Completed => stats.completed += 1,
                SessionStatus::Cancelled => stats.cancelled += 1,
                SessionStatus::Paused => stats.paused += 1,
            }
            stats.total_reviews += session.reviews.len();
        }
        stats.total_sessions = sessions.len();

        stats
    }
}

/// Session statistics.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub active: usize,
    pub completed: usize,
    pub cancelled: usize,
    pub paused: usize,
    pub total_reviews: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_status_serialization() {
        let statuses = [
            (SessionStatus::Active, "\"active\""),
            (SessionStatus::Paused, "\"paused\""),
            (SessionStatus::Completed, "\"completed\""),
            (SessionStatus::Cancelled, "\"cancelled\""),
        ];

        for (status, expected) in &statuses {
            let json = serde_json::to_string(status).unwrap();
            assert_eq!(json, *expected);
        }
    }

    #[test]
    fn test_review_session_creation() {
        let session = ReviewSession {
            id: "session-123".to_string(),
            status: SessionStatus::Active,
            target: "feature/test-branch".to_string(),
            reviews: Vec::new(),
            metadata: HashMap::new(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        assert_eq!(session.status, SessionStatus::Active);
        assert!(session.reviews.is_empty());
    }

    #[test]
    fn test_review_session_serialization() {
        let session = ReviewSession {
            id: "test".to_string(),
            status: SessionStatus::Active,
            target: "main".to_string(),
            reviews: Vec::new(),
            metadata: HashMap::new(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&session).unwrap();
        let parsed: ReviewSession = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, "test");
        assert_eq!(parsed.status, SessionStatus::Active);
    }

    #[test]
    fn test_session_stats_default() {
        let stats = SessionStats::default();

        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.active, 0);
        assert_eq!(stats.completed, 0);
        assert_eq!(stats.cancelled, 0);
        assert_eq!(stats.paused, 0);
        assert_eq!(stats.total_reviews, 0);
    }

    #[test]
    fn test_session_stats_serialization() {
        let stats = SessionStats {
            total_sessions: 10,
            active: 3,
            completed: 5,
            cancelled: 1,
            paused: 1,
            total_reviews: 25,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let parsed: SessionStats = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.total_sessions, 10);
        assert_eq!(parsed.total_reviews, 25);
    }

    #[test]
    fn test_session_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("pr_number".to_string(), serde_json::json!(123));
        metadata.insert("author".to_string(), serde_json::json!("test-user"));

        let session = ReviewSession {
            id: "test".to_string(),
            status: SessionStatus::Active,
            target: "PR #123".to_string(),
            reviews: Vec::new(),
            metadata,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        assert_eq!(session.metadata.len(), 2);
        assert_eq!(
            session.metadata.get("pr_number").unwrap(),
            &serde_json::json!(123)
        );
    }
}
