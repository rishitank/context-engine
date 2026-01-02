//! Retry logic with exponential backoff.

use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use tracing::debug;

use crate::error::Error;

/// Parameters for exponential backoff.
#[derive(Debug, Clone)]
pub struct BackoffParams {
    /// Initial delay in milliseconds
    pub initial_ms: u64,
    /// Multiplier for each retry
    pub mult: f64,
    /// Maximum delay in milliseconds
    pub max_ms: u64,
    /// Maximum number of tries (None = unlimited)
    pub max_tries: Option<u32>,
    /// Maximum total time in milliseconds (None = unlimited)
    pub max_total_ms: Option<u64>,
}

impl Default for BackoffParams {
    fn default() -> Self {
        Self {
            initial_ms: 100,
            mult: 2.0,
            max_ms: 30_000,
            max_tries: None,
            max_total_ms: None,
        }
    }
}

impl BackoffParams {
    /// Create params for chat operations (more lenient).
    pub fn for_chat() -> Self {
        Self {
            initial_ms: 500,
            mult: 2.0,
            max_ms: 60_000,
            max_tries: Some(5),
            max_total_ms: Some(300_000), // 5 minutes
        }
    }

    /// Create params for indexing operations.
    pub fn for_indexing() -> Self {
        Self {
            initial_ms: 3_000,
            mult: 1.5,
            max_ms: 60_000,
            max_tries: None,
            max_total_ms: Some(600_000), // 10 minutes
        }
    }
}

/// Retry a function with exponential backoff.
///
/// # Arguments
///
/// * `f` - The async function to retry
/// * `can_retry` - Function to check if an error is retriable
/// * `params` - Backoff parameters
/// * `debug` - Enable debug logging
///
/// # Returns
///
/// The result of the function, or the last error if all retries failed.
pub async fn retry_with_backoff<F, Fut, T, E, R>(
    mut f: F,
    can_retry: R,
    params: &BackoffParams,
    enable_debug: bool,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    R: Fn(&E) -> bool,
    E: std::fmt::Debug,
{
    let start_time = std::time::Instant::now();
    let mut backoff_ms = 0u64;
    let mut tries = 0u32;

    loop {
        match f().await {
            Ok(result) => {
                if tries > 0 && enable_debug {
                    debug!("Operation succeeded after {} transient failures", tries);
                }
                return Ok(result);
            }
            Err(e) => {
                tries += 1;

                // Check max tries
                if let Some(max) = params.max_tries {
                    if tries >= max {
                        return Err(e);
                    }
                }

                // Check if retriable
                if !can_retry(&e) {
                    return Err(e);
                }

                // Calculate backoff
                backoff_ms = if backoff_ms == 0 {
                    params.initial_ms
                } else {
                    ((backoff_ms as f64) * params.mult).min(params.max_ms as f64) as u64
                };

                // Check max total time
                if let Some(max_total) = params.max_total_ms {
                    let elapsed = start_time.elapsed().as_millis() as u64;
                    if elapsed + backoff_ms > max_total {
                        return Err(e);
                    }
                }

                if enable_debug {
                    debug!(
                        "Operation failed with error {:?}, retrying in {} ms; retries = {}",
                        e, backoff_ms, tries
                    );
                }

                sleep(Duration::from_millis(backoff_ms)).await;
            }
        }
    }
}

/// Retry with default retriable error check.
pub async fn retry_api<F, Fut, T>(f: F, params: &BackoffParams, debug: bool) -> crate::Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = crate::Result<T>>,
{
    retry_with_backoff(f, |e: &Error| e.is_retriable(), params, debug).await
}

/// Retry with chat-specific retriable error check.
pub async fn retry_chat<F, Fut, T>(f: F, params: &BackoffParams, debug: bool) -> crate::Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = crate::Result<T>>,
{
    retry_with_backoff(f, |e: &Error| e.is_chat_retriable(), params, debug).await
}
