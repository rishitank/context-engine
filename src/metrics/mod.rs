//! Prometheus metrics for monitoring.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Metrics collector.
#[derive(Debug, Default)]
pub struct Metrics {
    /// Total requests processed
    pub requests_total: AtomicU64,
    /// Successful requests
    pub requests_success: AtomicU64,
    /// Failed requests
    pub requests_failed: AtomicU64,
    /// Total search queries
    pub searches_total: AtomicU64,
    /// Total files indexed
    pub files_indexed: AtomicU64,
    /// Index operations
    pub index_operations: AtomicU64,
    /// Tool calls
    pub tool_calls: AtomicU64,
    /// Active sessions
    pub active_sessions: AtomicU64,
}

impl Metrics {
    /// Create a new metrics collector.
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Increment requests total.
    pub fn inc_requests(&self) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment successful requests.
    pub fn inc_success(&self) {
        self.requests_success.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment failed requests.
    pub fn inc_failed(&self) {
        self.requests_failed.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment search count.
    pub fn inc_searches(&self) {
        self.searches_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Set files indexed count.
    pub fn set_files_indexed(&self, count: u64) {
        self.files_indexed.store(count, Ordering::Relaxed);
    }

    /// Increment index operations.
    pub fn inc_index_ops(&self) {
        self.index_operations.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment tool calls.
    pub fn inc_tool_calls(&self) {
        self.tool_calls.fetch_add(1, Ordering::Relaxed);
    }

    /// Set active sessions.
    pub fn set_active_sessions(&self, count: u64) {
        self.active_sessions.store(count, Ordering::Relaxed);
    }

    /// Get all metrics as a snapshot.
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            requests_total: self.requests_total.load(Ordering::Relaxed),
            requests_success: self.requests_success.load(Ordering::Relaxed),
            requests_failed: self.requests_failed.load(Ordering::Relaxed),
            searches_total: self.searches_total.load(Ordering::Relaxed),
            files_indexed: self.files_indexed.load(Ordering::Relaxed),
            index_operations: self.index_operations.load(Ordering::Relaxed),
            tool_calls: self.tool_calls.load(Ordering::Relaxed),
            active_sessions: self.active_sessions.load(Ordering::Relaxed),
        }
    }

    /// Export metrics in Prometheus format.
    pub fn to_prometheus(&self) -> String {
        let s = self.snapshot();
        format!(
            r#"# HELP context_engine_requests_total Total number of requests
# TYPE context_engine_requests_total counter
context_engine_requests_total {}

# HELP context_engine_requests_success Successful requests
# TYPE context_engine_requests_success counter
context_engine_requests_success {}

# HELP context_engine_requests_failed Failed requests
# TYPE context_engine_requests_failed counter
context_engine_requests_failed {}

# HELP context_engine_searches_total Total search queries
# TYPE context_engine_searches_total counter
context_engine_searches_total {}

# HELP context_engine_files_indexed Number of files indexed
# TYPE context_engine_files_indexed gauge
context_engine_files_indexed {}

# HELP context_engine_index_operations Index operations count
# TYPE context_engine_index_operations counter
context_engine_index_operations {}

# HELP context_engine_tool_calls Tool calls count
# TYPE context_engine_tool_calls counter
context_engine_tool_calls {}

# HELP context_engine_active_sessions Active review sessions
# TYPE context_engine_active_sessions gauge
context_engine_active_sessions {}
"#,
            s.requests_total,
            s.requests_success,
            s.requests_failed,
            s.searches_total,
            s.files_indexed,
            s.index_operations,
            s.tool_calls,
            s.active_sessions
        )
    }
}

/// Metrics snapshot.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_failed: u64,
    pub searches_total: u64,
    pub files_indexed: u64,
    pub index_operations: u64,
    pub tool_calls: u64,
    pub active_sessions: u64,
}

/// Timer for measuring durations.
pub struct Timer {
    start: Instant,
}

impl Timer {
    /// Start a new timer.
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get elapsed time in milliseconds.
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    /// Get elapsed time in seconds.
    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }
}
