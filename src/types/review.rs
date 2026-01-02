//! Code review types.
//!
//! This module contains all types related to the code review system,
//! including review sessions, findings, invariants, and risk scoring.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete code review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    /// Unique review identifier
    pub id: String,
    /// Review title
    pub title: String,
    /// Review status
    pub status: ReviewStatus,
    /// Files being reviewed
    pub files: Vec<ReviewFile>,
    /// Review findings
    pub findings: Vec<Finding>,
    /// Overall risk score (0-100)
    pub risk_score: u8,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Invariants checked
    pub invariants: Vec<InvariantCheck>,
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
    /// Review metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Pending,
    InProgress,
    Completed,
    Approved,
    ChangesRequested,
    Rejected,
}

/// A file in a review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewFile {
    /// File path
    pub path: String,
    /// Change type
    pub change_type: ChangeType,
    /// Number of additions
    pub additions: u32,
    /// Number of deletions
    pub deletions: u32,
    /// Diff hunks
    pub hunks: Vec<DiffHunk>,
    /// File-level risk score
    pub risk_score: u8,
    /// File-level findings
    pub findings: Vec<Finding>,
}

/// Type of file change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
}

/// A diff hunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    /// Old file start line
    pub old_start: u32,
    /// Old file line count
    pub old_lines: u32,
    /// New file start line
    pub new_start: u32,
    /// New file line count
    pub new_lines: u32,
    /// Hunk content
    pub content: String,
    /// Hunk header
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
}

/// A review finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Finding identifier
    pub id: String,
    /// Finding type
    pub finding_type: FindingType,
    /// Severity level
    pub severity: Severity,
    /// Finding title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// File path
    pub file: String,
    /// Line number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// Line range
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_range: Option<(u32, u32)>,
    /// Suggested fix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    /// Code snippet
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_snippet: Option<String>,
    /// Whether this is actionable
    pub actionable: bool,
    /// Category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// Type of finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingType {
    Bug,
    Security,
    Performance,
    Style,
    Documentation,
    Testing,
    Complexity,
    Duplication,
    BestPractice,
    Accessibility,
    Compatibility,
}

/// Severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Risk level (re-exported from planning for convenience).
pub use super::planning::RiskLevel;

/// An invariant check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantCheck {
    /// Invariant name
    pub name: String,
    /// Invariant description
    pub description: String,
    /// Whether the check passed
    pub passed: bool,
    /// Failure message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_message: Option<String>,
    /// Affected files
    #[serde(default)]
    pub affected_files: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_status_serialization() {
        let status = ReviewStatus::InProgress;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"in_progress\"");

        let parsed: ReviewStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ReviewStatus::InProgress);
    }

    #[test]
    fn test_change_type_serialization() {
        let change = ChangeType::Modified;
        let json = serde_json::to_string(&change).unwrap();
        assert_eq!(json, "\"modified\"");
    }

    #[test]
    fn test_finding_type_variants() {
        let types = vec![
            FindingType::Bug,
            FindingType::Security,
            FindingType::Performance,
            FindingType::Style,
            FindingType::Documentation,
        ];

        for t in types {
            let json = serde_json::to_string(&t).unwrap();
            let parsed: FindingType = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, t);
        }
    }

    #[test]
    fn test_severity_ordering() {
        // Verify severity levels exist
        let severities = vec![
            Severity::Info,
            Severity::Warning,
            Severity::Error,
            Severity::Critical,
        ];

        for s in severities {
            let json = serde_json::to_string(&s).unwrap();
            assert!(!json.is_empty());
        }
    }

    #[test]
    fn test_finding_serialization() {
        let finding = Finding {
            id: "test-123".to_string(),
            finding_type: FindingType::Security,
            severity: Severity::Critical,
            title: "SQL Injection".to_string(),
            description: "Potential SQL injection vulnerability".to_string(),
            file: "src/db.rs".to_string(),
            line: Some(42),
            line_range: Some((40, 45)),
            suggestion: Some("Use parameterized queries".to_string()),
            code_snippet: Some("query(user_input)".to_string()),
            actionable: true,
            category: Some("security".to_string()),
        };

        let json = serde_json::to_string(&finding).unwrap();
        let parsed: Finding = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, finding.id);
        assert_eq!(parsed.title, finding.title);
        assert!(parsed.actionable);
    }

    #[test]
    fn test_invariant_check() {
        let check = InvariantCheck {
            name: "No console.log".to_string(),
            description: "Production code should not contain console.log".to_string(),
            passed: false,
            failure_message: Some("Found console.log in src/app.ts".to_string()),
            affected_files: vec!["src/app.ts".to_string()],
        };

        let json = serde_json::to_string(&check).unwrap();
        let parsed: InvariantCheck = serde_json::from_str(&json).unwrap();

        assert!(!parsed.passed);
        assert_eq!(parsed.affected_files.len(), 1);
    }

    #[test]
    fn test_diff_hunk() {
        let hunk = DiffHunk {
            old_start: 10,
            old_lines: 5,
            new_start: 10,
            new_lines: 7,
            content: "+added line\n-removed line".to_string(),
            header: Some("@@ -10,5 +10,7 @@".to_string()),
        };

        let json = serde_json::to_string(&hunk).unwrap();
        assert!(json.contains("old_start"));
        assert!(json.contains("new_lines"));
    }

    #[test]
    fn test_review_file() {
        let file = ReviewFile {
            path: "src/main.rs".to_string(),
            change_type: ChangeType::Modified,
            additions: 10,
            deletions: 5,
            hunks: vec![],
            risk_score: 25,
            findings: vec![],
        };

        assert_eq!(file.additions, 10);
        assert_eq!(file.deletions, 5);
        assert_eq!(file.risk_score, 25);
    }
}
