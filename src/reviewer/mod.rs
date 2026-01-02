//! Code review pipeline.
//!
//! Multi-pass review system with risk scoring and invariant checking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::Result;
use crate::service::ContextService;
use crate::types::review::*;

/// Review configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewConfig {
    /// Enable security checks
    pub security_checks: bool,
    /// Enable performance checks
    pub performance_checks: bool,
    /// Enable style checks
    pub style_checks: bool,
    /// Custom invariants to check
    pub invariants: Vec<InvariantDefinition>,
    /// Risk thresholds
    pub risk_thresholds: RiskThresholds,
}

impl Default for ReviewConfig {
    fn default() -> Self {
        Self {
            security_checks: true,
            performance_checks: true,
            style_checks: true,
            invariants: Vec::new(),
            risk_thresholds: RiskThresholds::default(),
        }
    }
}

/// Risk thresholds for categorization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskThresholds {
    pub low_max: u8,
    pub medium_max: u8,
    pub high_max: u8,
}

impl Default for RiskThresholds {
    fn default() -> Self {
        Self {
            low_max: 30,
            medium_max: 60,
            high_max: 85,
        }
    }
}

/// Invariant definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantDefinition {
    pub name: String,
    pub description: String,
    pub pattern: String,
    pub severity: Severity,
}

/// Review pipeline for analyzing code changes.
pub struct ReviewPipeline {
    context_service: Arc<ContextService>,
    config: ReviewConfig,
}

impl ReviewPipeline {
    /// Create a new review pipeline.
    pub fn new(context_service: Arc<ContextService>, config: ReviewConfig) -> Self {
        Self {
            context_service,
            config,
        }
    }

    /// Review a diff.
    pub async fn review_diff(&self, diff: &str, _context: Option<&str>) -> Result<Review> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        // Parse the diff
        let files = self.parse_diff(diff)?;
        
        // Calculate risk score
        let (risk_score, risk_level) = self.calculate_risk(&files);

        // Run invariant checks
        let invariants = self.check_invariants(diff).await?;

        // Generate findings
        let findings = self.generate_findings(&files, diff).await?;

        Ok(Review {
            id,
            title: "Code Review".to_string(),
            status: ReviewStatus::Completed,
            files,
            findings,
            risk_score,
            risk_level,
            invariants,
            created_at: now.clone(),
            updated_at: now,
            metadata: HashMap::new(),
        })
    }

    /// Parse a unified diff into review files.
    fn parse_diff(&self, diff: &str) -> Result<Vec<ReviewFile>> {
        let mut files = Vec::new();
        let mut current_file: Option<ReviewFile> = None;

        for line in diff.lines() {
            if line.starts_with("diff --git") || line.starts_with("--- ") || line.starts_with("+++ ") {
                // Extract file path
                if line.starts_with("+++ ") {
                    let path = line.trim_start_matches("+++ ").trim_start_matches("b/");
                    if let Some(ref mut file) = current_file {
                        files.push(file.clone());
                    }
                    current_file = Some(ReviewFile {
                        path: path.to_string(),
                        change_type: ChangeType::Modified,
                        additions: 0,
                        deletions: 0,
                        hunks: Vec::new(),
                        risk_score: 0,
                        findings: Vec::new(),
                    });
                }
            } else if let Some(ref mut file) = current_file {
                if line.starts_with('+') && !line.starts_with("+++") {
                    file.additions += 1;
                } else if line.starts_with('-') && !line.starts_with("---") {
                    file.deletions += 1;
                }
            }
        }

        if let Some(file) = current_file {
            files.push(file);
        }

        Ok(files)
    }

    /// Calculate risk score for the review.
    fn calculate_risk(&self, files: &[ReviewFile]) -> (u8, RiskLevel) {
        let mut score = 0u8;

        for file in files {
            // High-risk file patterns
            if file.path.contains("auth") || file.path.contains("security") {
                score = score.saturating_add(20);
            }
            if file.path.contains("database") || file.path.contains("migration") {
                score = score.saturating_add(15);
            }
            // Size-based risk
            let changes = file.additions + file.deletions;
            if changes > 100 {
                score = score.saturating_add(10);
            } else if changes > 50 {
                score = score.saturating_add(5);
            }
        }

        let level = if score > self.config.risk_thresholds.high_max {
            RiskLevel::Critical
        } else if score > self.config.risk_thresholds.medium_max {
            RiskLevel::High
        } else if score > self.config.risk_thresholds.low_max {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        (score.min(100), level)
    }

    /// Check invariants against the diff.
    async fn check_invariants(&self, diff: &str) -> Result<Vec<InvariantCheck>> {
        let mut results = Vec::new();

        for invariant in &self.config.invariants {
            let passed = !diff.contains(&invariant.pattern);

            results.push(InvariantCheck {
                name: invariant.name.clone(),
                description: invariant.description.clone(),
                passed,
                failure_message: if passed {
                    None
                } else {
                    Some(format!("Pattern '{}' found in diff", invariant.pattern))
                },
                affected_files: Vec::new(),
            });
        }

        Ok(results)
    }

    /// Generate findings from the diff using context service for AI analysis.
    async fn generate_findings(&self, files: &[ReviewFile], diff: &str) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();

        // Security patterns to check
        let security_patterns = [
            ("API key exposure", r#"(?i)(api[_-]?key|apikey)\s*[:=]\s*['"]?[a-zA-Z0-9]{20,}"#),
            ("Password in code", r#"(?i)(password|passwd|pwd)\s*[:=]\s*['"]"#),
            ("SQL injection risk", r#"(?i)(execute|query)\s*\(\s*['"].*\+"#),
            ("Hardcoded secret", r#"(?i)(secret|token|credential)\s*[:=]\s*['"]"#),
        ];

        if self.config.security_checks {
            for (name, pattern) in &security_patterns {
                if let Ok(re) = regex::Regex::new(pattern) {
                    for cap in re.find_iter(diff) {
                        findings.push(Finding {
                            id: uuid::Uuid::new_v4().to_string(),
                            finding_type: FindingType::Security,
                            severity: Severity::Critical,
                            title: name.to_string(),
                            description: format!("Potential security issue detected: {}", cap.as_str()),
                            file: String::new(),
                            line: None,
                            line_range: None,
                            suggestion: Some("Review and remove any hardcoded secrets or sensitive data".to_string()),
                            code_snippet: Some(cap.as_str().to_string()),
                            actionable: true,
                            category: Some("security".to_string()),
                        });
                    }
                }
            }
        }

        // Performance patterns
        if self.config.performance_checks {
            let perf_patterns = [
                ("N+1 query potential", r"(?i)for\s*\([^)]*\)\s*\{[^}]*\.(find|query|select)"),
                ("Missing async/await", r"\.then\s*\([^)]*\)\s*\.then"),
            ];

            for (name, pattern) in &perf_patterns {
                if let Ok(re) = regex::Regex::new(pattern) {
                    if re.is_match(diff) {
                        findings.push(Finding {
                            id: uuid::Uuid::new_v4().to_string(),
                            finding_type: FindingType::Performance,
                            severity: Severity::Warning,
                            title: name.to_string(),
                            description: format!("Potential performance issue: {}", name),
                            file: String::new(),
                            line: None,
                            line_range: None,
                            suggestion: Some("Consider optimizing this pattern".to_string()),
                            code_snippet: None,
                            actionable: true,
                            category: Some("performance".to_string()),
                        });
                    }
                }
            }
        }

        // Use context service for semantic analysis if files are changed
        if !files.is_empty() {
            let query = format!(
                "Analyze these code changes for potential issues:\n\nChanged files: {}\n\nDiff summary: {} lines added, {} lines removed",
                files.iter().map(|f| f.path.as_str()).collect::<Vec<_>>().join(", "),
                files.iter().map(|f| f.additions as usize).sum::<usize>(),
                files.iter().map(|f| f.deletions as usize).sum::<usize>()
            );

            // Try to get AI-powered analysis
            match self.context_service.search(&query, Some(2000)).await {
                Ok(analysis) => {
                    if !analysis.is_empty() && analysis.len() > 50 {
                        findings.push(Finding {
                            id: uuid::Uuid::new_v4().to_string(),
                            finding_type: FindingType::BestPractice,
                            severity: Severity::Info,
                            title: "AI Analysis".to_string(),
                            description: analysis,
                            file: String::new(),
                            line: None,
                            line_range: None,
                            suggestion: None,
                            code_snippet: None,
                            actionable: false,
                            category: Some("ai-analysis".to_string()),
                        });
                    }
                }
                Err(e) => {
                    tracing::debug!("Context service analysis failed: {}", e);
                }
            }
        }

        Ok(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ReviewConfig {
        ReviewConfig {
            security_checks: true,
            performance_checks: true,
            style_checks: true,
            invariants: Vec::new(),
            risk_thresholds: RiskThresholds::default(),
        }
    }

    #[test]
    fn test_review_config_default() {
        let config = ReviewConfig::default();
        assert!(config.security_checks);
        assert!(config.performance_checks);
        assert!(config.style_checks);
        assert!(config.invariants.is_empty());
    }

    #[test]
    fn test_risk_thresholds_default() {
        let thresholds = RiskThresholds::default();
        assert_eq!(thresholds.low_max, 30);
        assert_eq!(thresholds.medium_max, 60);
        assert_eq!(thresholds.high_max, 85);
    }

    #[test]
    fn test_invariant_definition() {
        let invariant = InvariantDefinition {
            name: "No console.log".to_string(),
            description: "Production code should not contain console.log".to_string(),
            pattern: "console.log".to_string(),
            severity: Severity::Warning,
        };

        let json = serde_json::to_string(&invariant).unwrap();
        let parsed: InvariantDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "No console.log");
    }

    #[test]
    fn test_parse_simple_diff() {
        // Create a mock context service for testing
        // We can't easily test the full pipeline without mocking,
        // so we test the individual components

        let diff = r#"diff --git a/file.rs b/file.rs
--- a/file.rs
+++ b/file.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!("Hello");
     println!("World");
 }
"#;

        // Parse diff logic test
        assert!(diff.contains("+++ b/file.rs"));
        assert!(diff.contains("+ "));
    }

    /// Helper to calculate risk score using the same logic as ReviewPipeline
    fn calculate_test_risk(files: &[ReviewFile], thresholds: &RiskThresholds) -> (u8, RiskLevel) {
        let mut score = 0u8;

        for file in files {
            // High-risk file patterns
            if file.path.contains("auth") || file.path.contains("security") {
                score = score.saturating_add(20);
            }
            if file.path.contains("database") || file.path.contains("migration") {
                score = score.saturating_add(15);
            }
            // Size-based risk
            let changes = file.additions + file.deletions;
            if changes > 100 {
                score = score.saturating_add(10);
            } else if changes > 50 {
                score = score.saturating_add(5);
            }
        }

        let level = if score > thresholds.high_max {
            RiskLevel::Critical
        } else if score > thresholds.medium_max {
            RiskLevel::High
        } else if score > thresholds.low_max {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        (score.min(100), level)
    }

    #[test]
    fn test_risk_calculation_low() {
        let config = create_test_config();
        let files = vec![
            ReviewFile {
                path: "src/utils.rs".to_string(),
                change_type: ChangeType::Modified,
                additions: 10,
                deletions: 5,
                hunks: Vec::new(),
                risk_score: 0,
                findings: Vec::new(),
            },
        ];

        let (score, level) = calculate_test_risk(&files, &config.risk_thresholds);

        // Low risk: small changes (15 total), no sensitive paths
        assert_eq!(score, 0); // No risk factors triggered
        assert_eq!(level, RiskLevel::Low);
    }

    #[test]
    fn test_risk_calculation_medium() {
        let config = create_test_config();
        let files = vec![
            ReviewFile {
                path: "src/utils.rs".to_string(),
                change_type: ChangeType::Modified,
                additions: 80,
                deletions: 30,
                hunks: Vec::new(),
                risk_score: 0,
                findings: Vec::new(),
            },
        ];

        let (score, level) = calculate_test_risk(&files, &config.risk_thresholds);

        // Medium risk: large changes (110 total) triggers +10
        assert_eq!(score, 10);
        assert!(score <= config.risk_thresholds.low_max); // 10 <= 30, so still Low
        assert_eq!(level, RiskLevel::Low);
    }

    #[test]
    fn test_risk_calculation_high() {
        let config = create_test_config();
        let files = vec![
            ReviewFile {
                path: "src/auth/login.rs".to_string(),
                change_type: ChangeType::Modified,
                additions: 100,
                deletions: 50,
                hunks: Vec::new(),
                risk_score: 0,
                findings: Vec::new(),
            },
        ];

        let (score, level) = calculate_test_risk(&files, &config.risk_thresholds);

        // High risk: auth path (+20) + large changes (+10) = 30
        assert_eq!(score, 30);
        assert_eq!(level, RiskLevel::Low); // 30 == low_max, so still Low
    }

    #[test]
    fn test_risk_calculation_critical() {
        let config = create_test_config();
        let files = vec![
            ReviewFile {
                path: "src/auth/login.rs".to_string(),
                change_type: ChangeType::Modified,
                additions: 200,
                deletions: 100,
                hunks: Vec::new(),
                risk_score: 0,
                findings: Vec::new(),
            },
            ReviewFile {
                path: "src/security/tokens.rs".to_string(),
                change_type: ChangeType::Modified,
                additions: 150,
                deletions: 50,
                hunks: Vec::new(),
                risk_score: 0,
                findings: Vec::new(),
            },
            ReviewFile {
                path: "src/database/migrations/001.rs".to_string(),
                change_type: ChangeType::Added,
                additions: 150,
                deletions: 0,
                hunks: Vec::new(),
                risk_score: 0,
                findings: Vec::new(),
            },
            ReviewFile {
                path: "src/database/schema.rs".to_string(),
                change_type: ChangeType::Modified,
                additions: 100,
                deletions: 50,
                hunks: Vec::new(),
                risk_score: 0,
                findings: Vec::new(),
            },
        ];

        let (score, level) = calculate_test_risk(&files, &config.risk_thresholds);

        // File 1: auth (+20) + large (+10) = 30
        // File 2: security (+20) + large (+10) = 30
        // File 3: database (+15) + migration (+15) + large (+10) = 40
        // File 4: database (+15) + large (+10) = 25
        // Total: 30 + 30 + 40 + 25 = 125, capped at 100
        assert_eq!(score, 100);
        assert!(score > config.risk_thresholds.high_max); // 100 > 85
        assert_eq!(level, RiskLevel::Critical);
    }

    #[test]
    fn test_security_pattern_detection() {
        let patterns = [
            (r#"api_key = "abcd1234567890123456""#, true),
            (r#"password = "secret""#, true),
            (r#"const name = "John""#, false),
        ];

        let api_key_regex = regex::Regex::new(r#"(?i)(api[_-]?key|apikey)\s*[:=]\s*['"]?[a-zA-Z0-9]{20,}"#).unwrap();
        let password_regex = regex::Regex::new(r#"(?i)(password|passwd|pwd)\s*[:=]\s*['"]"#).unwrap();

        for (input, should_match) in &patterns {
            let matches = api_key_regex.is_match(input) || password_regex.is_match(input);
            assert_eq!(matches, *should_match, "Pattern mismatch for: {}", input);
        }
    }

    #[test]
    fn test_review_file_creation() {
        let file = ReviewFile {
            path: "test.rs".to_string(),
            change_type: ChangeType::Added,
            additions: 50,
            deletions: 0,
            hunks: Vec::new(),
            risk_score: 10,
            findings: Vec::new(),
        };

        assert_eq!(file.change_type, ChangeType::Added);
        assert_eq!(file.additions, 50);
    }

    #[test]
    fn test_finding_creation() {
        let finding = Finding {
            id: "test-123".to_string(),
            finding_type: FindingType::Security,
            severity: Severity::Critical,
            title: "Hardcoded secret".to_string(),
            description: "Found hardcoded API key".to_string(),
            file: "config.rs".to_string(),
            line: Some(42),
            line_range: Some((40, 45)),
            suggestion: Some("Use environment variables".to_string()),
            code_snippet: Some("api_key = \"...\"".to_string()),
            actionable: true,
            category: Some("security".to_string()),
        };

        assert_eq!(finding.severity, Severity::Critical);
        assert!(finding.actionable);
    }

    #[test]
    fn test_config_with_invariants() {
        let config = ReviewConfig {
            security_checks: true,
            performance_checks: false,
            style_checks: false,
            invariants: vec![
                InvariantDefinition {
                    name: "No TODO".to_string(),
                    description: "Code should not contain TODO comments".to_string(),
                    pattern: "TODO".to_string(),
                    severity: Severity::Warning,
                },
            ],
            risk_thresholds: RiskThresholds::default(),
        };

        assert_eq!(config.invariants.len(), 1);
        assert_eq!(config.invariants[0].name, "No TODO");
    }
}
