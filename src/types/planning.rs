//! Planning and task management types.
//!
//! This module contains all types related to the AI-powered planning system,
//! including plans, steps, dependencies, and execution state.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete plan with steps and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    /// Unique plan identifier
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Plan status
    pub status: PlanStatus,
    /// Ordered list of steps
    pub steps: Vec<Step>,
    /// Creation timestamp (ISO 8601)
    pub created_at: String,
    /// Last update timestamp (ISO 8601)
    pub updated_at: String,
    /// Plan metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Risk assessment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_assessment: Option<RiskAssessment>,
    /// Estimated total duration in minutes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_duration: Option<u32>,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Status of a plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Draft,
    Active,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// A single step in a plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// Step number (1-based)
    pub id: u32,
    /// Step title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Step status
    pub status: StepStatus,
    /// Step type
    pub step_type: StepType,
    /// Dependencies (step IDs that must complete first)
    #[serde(default)]
    pub dependencies: Vec<u32>,
    /// Files affected by this step
    #[serde(default)]
    pub affected_files: Vec<String>,
    /// Estimated duration in minutes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_duration: Option<u32>,
    /// Actual duration in minutes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_duration: Option<u32>,
    /// Completion timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Output/result of the step
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    /// Whether this step requires human approval
    #[serde(default)]
    pub requires_approval: bool,
    /// Approval status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval: Option<ApprovalStatus>,
    /// Rollback instructions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollback: Option<String>,
    /// Validation criteria
    #[serde(default)]
    pub validation: Vec<ValidationCriterion>,
}

/// Status of a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Ready,
    InProgress,
    Completed,
    Failed,
    Skipped,
    Blocked,
    AwaitingApproval,
}

/// Type of step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    Analysis,
    Implementation,
    Testing,
    Review,
    Deployment,
    Documentation,
    Refactoring,
    Configuration,
    Manual,
}

/// Approval status for a step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalStatus {
    /// Whether approved
    pub approved: bool,
    /// Approver identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,
    /// Approval timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<String>,
    /// Approval notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Validation criterion for a step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCriterion {
    /// Criterion description
    pub description: String,
    /// Whether validated
    pub validated: bool,
    /// Validation method
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
}

/// Risk assessment for a plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Overall risk level
    pub level: RiskLevel,
    /// Risk score (0-100)
    pub score: u8,
    /// Individual risk factors
    pub factors: Vec<RiskFactor>,
    /// Mitigation strategies
    pub mitigations: Vec<String>,
}

/// Risk level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// A single risk factor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Factor name
    pub name: String,
    /// Factor description
    pub description: String,
    /// Impact level
    pub impact: RiskLevel,
    /// Likelihood
    pub likelihood: RiskLevel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_status_serialization() {
        let status = PlanStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"active\"");

        let parsed: PlanStatus = serde_json::from_str("\"completed\"").unwrap();
        assert_eq!(parsed, PlanStatus::Completed);
    }

    #[test]
    fn test_step_status_serialization() {
        let status = StepStatus::InProgress;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"in_progress\"");
    }

    #[test]
    fn test_risk_level_serialization() {
        let levels = vec![
            (RiskLevel::Low, "\"low\""),
            (RiskLevel::Medium, "\"medium\""),
            (RiskLevel::High, "\"high\""),
            (RiskLevel::Critical, "\"critical\""),
        ];

        for (level, expected) in levels {
            let json = serde_json::to_string(&level).unwrap();
            assert_eq!(json, expected);
        }
    }

    #[test]
    fn test_step_creation() {
        let step = Step {
            id: 1,
            title: "Initialize project".to_string(),
            description: "Set up the project structure".to_string(),
            status: StepStatus::Pending,
            step_type: StepType::Implementation,
            dependencies: vec![],
            affected_files: vec![],
            estimated_duration: Some(30),
            actual_duration: None,
            completed_at: None,
            error: None,
            output: None,
            requires_approval: false,
            approval: None,
            rollback: None,
            validation: vec![],
        };

        assert_eq!(step.id, 1);
        assert!(step.dependencies.is_empty());
        assert_eq!(step.estimated_duration, Some(30));
    }

    #[test]
    fn test_plan_with_steps() {
        let plan = Plan {
            id: "plan-123".to_string(),
            title: "Refactor Module".to_string(),
            description: "Refactor the authentication module".to_string(),
            status: PlanStatus::Draft,
            steps: vec![
                Step {
                    id: 1,
                    title: "Analyze".to_string(),
                    description: "Analyze current code".to_string(),
                    status: StepStatus::Pending,
                    step_type: StepType::Analysis,
                    dependencies: vec![],
                    affected_files: vec![],
                    estimated_duration: None,
                    actual_duration: None,
                    completed_at: None,
                    error: None,
                    output: None,
                    requires_approval: false,
                    approval: None,
                    rollback: None,
                    validation: vec![],
                },
            ],
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            metadata: HashMap::new(),
            risk_assessment: None,
            estimated_duration: Some(60),
            tags: vec!["refactor".to_string()],
        };

        let json = serde_json::to_string(&plan).unwrap();
        let parsed: Plan = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, plan.id);
        assert_eq!(parsed.steps.len(), 1);
        assert_eq!(parsed.tags.len(), 1);
    }

    #[test]
    fn test_risk_assessment() {
        let assessment = RiskAssessment {
            level: RiskLevel::Medium,
            score: 45,
            factors: vec![
                RiskFactor {
                    name: "Complexity".to_string(),
                    description: "High cyclomatic complexity".to_string(),
                    impact: RiskLevel::Medium,
                    likelihood: RiskLevel::High,
                },
            ],
            mitigations: vec!["Add comprehensive tests".to_string()],
        };

        let json = serde_json::to_string(&assessment).unwrap();
        let parsed: RiskAssessment = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.score, 45);
        assert_eq!(parsed.factors.len(), 1);
    }

    #[test]
    fn test_step_type_serialization() {
        let types = vec![
            StepType::Analysis,
            StepType::Implementation,
            StepType::Testing,
            StepType::Review,
            StepType::Deployment,
        ];

        for t in types {
            let json = serde_json::to_string(&t).unwrap();
            let parsed: StepType = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, t);
        }
    }
}

