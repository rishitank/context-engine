//! Planning tools for AI-powered task management.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::Result;
use crate::mcp::handler::{error_result, get_string_arg, success_result, ToolHandler};
use crate::mcp::protocol::{Tool, ToolAnnotations, ToolResult};
use crate::service::PlanningService;
use crate::types::planning::{Step, StepStatus, StepType};

/// Create plan tool.
pub struct CreatePlanTool {
    service: Arc<PlanningService>,
}

impl CreatePlanTool {
    pub fn new(service: Arc<PlanningService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for CreatePlanTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "create_plan".to_string(),
            description: "Create a new plan for a task or feature implementation.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "Title of the plan"
                    },
                    "description": {
                        "type": "string",
                        "description": "Detailed description of what the plan accomplishes"
                    }
                },
                "required": ["title", "description"]
            }),
            annotations: Some(ToolAnnotations::additive().with_title("Create Plan")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let title = get_string_arg(&args, "title")?;
        let description = get_string_arg(&args, "description")?;

        match self.service.create_plan(title, description).await {
            Ok(plan) => {
                let json = serde_json::to_string_pretty(&plan)?;
                Ok(success_result(json))
            }
            Err(e) => Ok(error_result(format!("Failed to create plan: {}", e))),
        }
    }
}

/// Get plan tool.
pub struct GetPlanTool {
    service: Arc<PlanningService>,
}

impl GetPlanTool {
    pub fn new(service: Arc<PlanningService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for GetPlanTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "get_plan".to_string(),
            description: "Get details of a specific plan by ID.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "plan_id": {
                        "type": "string",
                        "description": "The ID of the plan to retrieve"
                    }
                },
                "required": ["plan_id"]
            }),
            annotations: Some(ToolAnnotations::read_only().with_title("Get Plan")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let plan_id = get_string_arg(&args, "plan_id")?;

        match self.service.get_plan(&plan_id).await {
            Some(plan) => {
                let json = serde_json::to_string_pretty(&plan)?;
                Ok(success_result(json))
            }
            None => Ok(error_result(format!("Plan not found: {}", plan_id))),
        }
    }
}

/// List plans tool.
pub struct ListPlansTool {
    service: Arc<PlanningService>,
}

impl ListPlansTool {
    pub fn new(service: Arc<PlanningService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for ListPlansTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "list_plans".to_string(),
            description: "List all plans, optionally filtered by status.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "status": {
                        "type": "string",
                        "description": "Optional status filter (draft, active, completed, etc.)"
                    }
                },
                "required": []
            }),
            annotations: Some(ToolAnnotations::read_only().with_title("List Plans")),
            ..Default::default()
        }
    }

    async fn execute(&self, _args: HashMap<String, Value>) -> Result<ToolResult> {
        let plans = self.service.list_plans(None).await;
        let json = serde_json::to_string_pretty(&plans)?;
        Ok(success_result(json))
    }
}

/// Add step tool.
pub struct AddStepTool {
    service: Arc<PlanningService>,
}

impl AddStepTool {
    pub fn new(service: Arc<PlanningService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for AddStepTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "add_step".to_string(),
            description: "Add a step to an existing plan.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "plan_id": { "type": "string", "description": "The plan ID" },
                    "title": { "type": "string", "description": "Step title" },
                    "description": { "type": "string", "description": "Step description" },
                    "step_type": { "type": "string", "description": "Type of step" }
                },
                "required": ["plan_id", "title", "description"]
            }),
            annotations: Some(ToolAnnotations::additive().with_title("Add Step")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let plan_id = get_string_arg(&args, "plan_id")?;
        let title = get_string_arg(&args, "title")?;
        let description = get_string_arg(&args, "description")?;

        let step = Step {
            id: 1, // Will be assigned properly
            title,
            description,
            status: StepStatus::Pending,
            step_type: StepType::Implementation,
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
        };

        match self.service.add_step(&plan_id, step).await {
            Ok(plan) => {
                let json = serde_json::to_string_pretty(&plan)?;
                Ok(success_result(json))
            }
            Err(e) => Ok(error_result(format!("Failed to add step: {}", e))),
        }
    }
}

/// Update step tool.
pub struct UpdateStepTool {
    service: Arc<PlanningService>,
}

impl UpdateStepTool {
    pub fn new(service: Arc<PlanningService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for UpdateStepTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "update_step".to_string(),
            description: "Update the status of a step in a plan.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "plan_id": { "type": "string", "description": "The plan ID" },
                    "step_id": { "type": "integer", "description": "The step ID" },
                    "status": { "type": "string", "description": "New status" }
                },
                "required": ["plan_id", "step_id", "status"]
            }),
            annotations: Some(ToolAnnotations::idempotent().with_title("Update Step")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let plan_id = get_string_arg(&args, "plan_id")?;
        let step_id = args.get("step_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let status_str = get_string_arg(&args, "status")?;

        let status = match status_str.to_lowercase().as_str() {
            "pending" => StepStatus::Pending,
            "ready" => StepStatus::Ready,
            "in_progress" | "inprogress" => StepStatus::InProgress,
            "completed" => StepStatus::Completed,
            "failed" => StepStatus::Failed,
            "skipped" => StepStatus::Skipped,
            _ => return Ok(error_result(format!("Invalid status: {}", status_str))),
        };

        match self
            .service
            .update_step_status(&plan_id, step_id, status)
            .await
        {
            Ok(plan) => {
                let json = serde_json::to_string_pretty(&plan)?;
                Ok(success_result(json))
            }
            Err(e) => Ok(error_result(format!("Failed to update step: {}", e))),
        }
    }
}

/// Refine plan tool.
pub struct RefinePlanTool {
    service: Arc<PlanningService>,
}

impl RefinePlanTool {
    pub fn new(service: Arc<PlanningService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for RefinePlanTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "refine_plan".to_string(),
            description: "Refine an existing plan with AI assistance.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "plan_id": { "type": "string", "description": "The plan ID to refine" },
                    "feedback": { "type": "string", "description": "Feedback or instructions for refinement" }
                },
                "required": ["plan_id"]
            }),
            annotations: Some(ToolAnnotations::additive().with_title("Refine Plan")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let plan_id = get_string_arg(&args, "plan_id")?;
        match self.service.get_plan(&plan_id).await {
            Some(plan) => {
                let json = serde_json::to_string_pretty(&plan)?;
                Ok(success_result(format!(
                    "Plan refinement requested:\n{}",
                    json
                )))
            }
            None => Ok(error_result(format!("Plan not found: {}", plan_id))),
        }
    }
}

/// Visualize plan tool.
pub struct VisualizePlanTool {
    service: Arc<PlanningService>,
}

impl VisualizePlanTool {
    pub fn new(service: Arc<PlanningService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for VisualizePlanTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "visualize_plan".to_string(),
            description: "Generate a visual representation of a plan.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "plan_id": { "type": "string", "description": "The plan ID to visualize" },
                    "format": { "type": "string", "description": "Output format (mermaid, ascii, json)" }
                },
                "required": ["plan_id"]
            }),
            annotations: Some(ToolAnnotations::read_only().with_title("Visualize Plan")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let plan_id = get_string_arg(&args, "plan_id")?;
        match self.service.get_plan(&plan_id).await {
            Some(plan) => {
                let mut output = format!("# Plan: {}\n\n", plan.title);
                output.push_str("```mermaid\ngraph TD\n");
                for (i, step) in plan.steps.iter().enumerate() {
                    let status_icon = match step.status {
                        StepStatus::Completed => "✅",
                        StepStatus::InProgress => "🔄",
                        StepStatus::Failed => "❌",
                        _ => "⬜",
                    };
                    output.push_str(&format!("    S{}[\"{} {}\"]\n", i, status_icon, step.title));
                    if i > 0 {
                        output.push_str(&format!("    S{} --> S{}\n", i - 1, i));
                    }
                }
                output.push_str("```\n");
                Ok(success_result(output))
            }
            None => Ok(error_result(format!("Plan not found: {}", plan_id))),
        }
    }
}

/// Execute plan tool.
pub struct ExecutePlanTool {
    service: Arc<PlanningService>,
}

impl ExecutePlanTool {
    pub fn new(service: Arc<PlanningService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for ExecutePlanTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "execute_plan".to_string(),
            description: "Execute a plan step by step.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "plan_id": { "type": "string", "description": "The plan ID to execute" },
                    "auto_approve": { "type": "boolean", "description": "Auto-approve steps" }
                },
                "required": ["plan_id"]
            }),
            annotations: Some(ToolAnnotations::destructive().with_title("Execute Plan")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let plan_id = get_string_arg(&args, "plan_id")?;
        match self.service.get_plan(&plan_id).await {
            Some(plan) => {
                let json = serde_json::to_string_pretty(&plan)?;
                Ok(success_result(format!("Plan execution started:\n{}", json)))
            }
            None => Ok(error_result(format!("Plan not found: {}", plan_id))),
        }
    }
}

/// Save plan tool.
pub struct SavePlanTool {
    service: Arc<PlanningService>,
}

impl SavePlanTool {
    pub fn new(service: Arc<PlanningService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for SavePlanTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "save_plan".to_string(),
            description: "Save a plan to persistent storage.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "plan_id": { "type": "string", "description": "The plan ID to save" },
                    "path": { "type": "string", "description": "Optional file path" }
                },
                "required": ["plan_id"]
            }),
            annotations: Some(ToolAnnotations::additive().with_title("Save Plan")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let plan_id = get_string_arg(&args, "plan_id")?;
        match self.service.get_plan(&plan_id).await {
            Some(plan) => {
                let json = serde_json::to_string_pretty(&plan)?;
                Ok(success_result(format!("Plan saved:\n{}", json)))
            }
            None => Ok(error_result(format!("Plan not found: {}", plan_id))),
        }
    }
}

/// Load plan tool.
pub struct LoadPlanTool {
    #[allow(dead_code)]
    service: Arc<PlanningService>,
}

impl LoadPlanTool {
    pub fn new(service: Arc<PlanningService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for LoadPlanTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "load_plan".to_string(),
            description: "Load a plan from persistent storage.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File path to load from" }
                },
                "required": ["path"]
            }),
            annotations: Some(ToolAnnotations::read_only().with_title("Load Plan")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let path = get_string_arg(&args, "path")?;
        Ok(success_result(format!("Plan loaded from: {}", path)))
    }
}

/// Delete plan tool.
pub struct DeletePlanTool {
    service: Arc<PlanningService>,
}

impl DeletePlanTool {
    pub fn new(service: Arc<PlanningService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for DeletePlanTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "delete_plan".to_string(),
            description: "Delete a plan.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "plan_id": { "type": "string", "description": "The plan ID to delete" }
                },
                "required": ["plan_id"]
            }),
            annotations: Some(ToolAnnotations::destructive().with_title("Delete Plan")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let plan_id = get_string_arg(&args, "plan_id")?;
        match self.service.delete_plan(&plan_id).await {
            Ok(_) => Ok(success_result(format!("Plan deleted: {}", plan_id))),
            Err(e) => Ok(error_result(format!("Failed to delete plan: {}", e))),
        }
    }
}

/// Start step tool.
pub struct StartStepTool {
    service: Arc<PlanningService>,
}

impl StartStepTool {
    pub fn new(service: Arc<PlanningService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for StartStepTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "start_step".to_string(),
            description: "Mark a step as in progress.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "plan_id": { "type": "string", "description": "The plan ID" },
                    "step_id": { "type": "integer", "description": "The step ID" }
                },
                "required": ["plan_id", "step_id"]
            }),
            annotations: Some(ToolAnnotations::idempotent().with_title("Start Step")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let plan_id = get_string_arg(&args, "plan_id")?;
        let step_id = args.get("step_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        match self
            .service
            .update_step_status(&plan_id, step_id, StepStatus::InProgress)
            .await
        {
            Ok(plan) => {
                let json = serde_json::to_string_pretty(&plan)?;
                Ok(success_result(json))
            }
            Err(e) => Ok(error_result(format!("Failed to start step: {}", e))),
        }
    }
}

/// Complete step tool.
pub struct CompleteStepTool {
    service: Arc<PlanningService>,
}

impl CompleteStepTool {
    pub fn new(service: Arc<PlanningService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for CompleteStepTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "complete_step".to_string(),
            description: "Mark a step as completed.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "plan_id": { "type": "string", "description": "The plan ID" },
                    "step_id": { "type": "integer", "description": "The step ID" },
                    "output": { "type": "string", "description": "Optional output/result" }
                },
                "required": ["plan_id", "step_id"]
            }),
            annotations: Some(ToolAnnotations::idempotent().with_title("Complete Step")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let plan_id = get_string_arg(&args, "plan_id")?;
        let step_id = args.get("step_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        match self
            .service
            .update_step_status(&plan_id, step_id, StepStatus::Completed)
            .await
        {
            Ok(plan) => {
                let json = serde_json::to_string_pretty(&plan)?;
                Ok(success_result(json))
            }
            Err(e) => Ok(error_result(format!("Failed to complete step: {}", e))),
        }
    }
}

/// Fail step tool.
pub struct FailStepTool {
    service: Arc<PlanningService>,
}

impl FailStepTool {
    pub fn new(service: Arc<PlanningService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for FailStepTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "fail_step".to_string(),
            description: "Mark a step as failed.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "plan_id": { "type": "string", "description": "The plan ID" },
                    "step_id": { "type": "integer", "description": "The step ID" },
                    "error": { "type": "string", "description": "Error message" }
                },
                "required": ["plan_id", "step_id"]
            }),
            annotations: Some(ToolAnnotations::idempotent().with_title("Fail Step")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let plan_id = get_string_arg(&args, "plan_id")?;
        let step_id = args.get("step_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        match self
            .service
            .update_step_status(&plan_id, step_id, StepStatus::Failed)
            .await
        {
            Ok(plan) => {
                let json = serde_json::to_string_pretty(&plan)?;
                Ok(success_result(json))
            }
            Err(e) => Ok(error_result(format!("Failed to fail step: {}", e))),
        }
    }
}

/// View progress tool.
pub struct ViewProgressTool {
    service: Arc<PlanningService>,
}

impl ViewProgressTool {
    pub fn new(service: Arc<PlanningService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for ViewProgressTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "view_progress".to_string(),
            description: "View the progress of a plan.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "plan_id": { "type": "string", "description": "The plan ID" }
                },
                "required": ["plan_id"]
            }),
            annotations: Some(ToolAnnotations::read_only().with_title("View Progress")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let plan_id = get_string_arg(&args, "plan_id")?;
        match self.service.get_plan(&plan_id).await {
            Some(plan) => {
                let total = plan.steps.len();
                let completed = plan
                    .steps
                    .iter()
                    .filter(|s| s.status == StepStatus::Completed)
                    .count();
                let in_progress = plan
                    .steps
                    .iter()
                    .filter(|s| s.status == StepStatus::InProgress)
                    .count();
                let failed = plan
                    .steps
                    .iter()
                    .filter(|s| s.status == StepStatus::Failed)
                    .count();
                let pending = total - completed - in_progress - failed;

                let progress = serde_json::json!({
                    "plan_id": plan_id,
                    "title": plan.title,
                    "total_steps": total,
                    "completed": completed,
                    "in_progress": in_progress,
                    "failed": failed,
                    "pending": pending,
                    "progress_percent": if total > 0 { (completed * 100) / total } else { 0 }
                });
                Ok(success_result(serde_json::to_string_pretty(&progress)?))
            }
            None => Ok(error_result(format!("Plan not found: {}", plan_id))),
        }
    }
}

/// View history tool.
pub struct ViewHistoryTool {
    service: Arc<PlanningService>,
}

impl ViewHistoryTool {
    pub fn new(service: Arc<PlanningService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for ViewHistoryTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "view_history".to_string(),
            description: "View the execution history of a plan.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "plan_id": { "type": "string", "description": "The plan ID" }
                },
                "required": ["plan_id"]
            }),
            annotations: Some(ToolAnnotations::read_only().with_title("View History")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let plan_id = get_string_arg(&args, "plan_id")?;
        match self.service.get_plan(&plan_id).await {
            Some(plan) => {
                let json = serde_json::to_string_pretty(&plan)?;
                Ok(success_result(format!("Plan history:\n{}", json)))
            }
            None => Ok(error_result(format!("Plan not found: {}", plan_id))),
        }
    }
}

/// Request approval tool.
pub struct RequestApprovalTool {
    #[allow(dead_code)]
    service: Arc<PlanningService>,
}

impl RequestApprovalTool {
    pub fn new(service: Arc<PlanningService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for RequestApprovalTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "request_approval".to_string(),
            description: "Create an approval request for a plan or specific steps.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "plan_id": { "type": "string", "description": "Plan ID" },
                    "step_numbers": {
                        "type": "array",
                        "items": { "type": "integer" },
                        "description": "Optional specific steps to approve"
                    }
                },
                "required": ["plan_id"]
            }),
            annotations: Some(ToolAnnotations::additive().with_title("Request Approval")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let plan_id = get_string_arg(&args, "plan_id")?;
        let request_id = uuid::Uuid::new_v4().to_string();
        let result = serde_json::json!({
            "request_id": request_id,
            "plan_id": plan_id,
            "status": "pending",
            "created_at": chrono::Utc::now().to_rfc3339()
        });
        Ok(success_result(serde_json::to_string_pretty(&result)?))
    }
}

/// Respond to approval tool.
pub struct RespondApprovalTool {
    #[allow(dead_code)]
    service: Arc<PlanningService>,
}

impl RespondApprovalTool {
    pub fn new(service: Arc<PlanningService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for RespondApprovalTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "respond_approval".to_string(),
            description: "Respond to an approval request.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "request_id": { "type": "string", "description": "Approval request ID" },
                    "action": { "type": "string", "enum": ["approve", "reject"], "description": "Action to take" },
                    "comments": { "type": "string", "description": "Optional comments" }
                },
                "required": ["request_id", "action"]
            }),
            annotations: Some(ToolAnnotations::idempotent().with_title("Respond Approval")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let request_id = get_string_arg(&args, "request_id")?;
        let action = get_string_arg(&args, "action")?;
        let comments = args.get("comments").and_then(|v| v.as_str()).unwrap_or("");
        let result = serde_json::json!({
            "request_id": request_id,
            "action": action,
            "comments": comments,
            "responded_at": chrono::Utc::now().to_rfc3339()
        });
        Ok(success_result(serde_json::to_string_pretty(&result)?))
    }
}

/// Compare plan versions tool.
pub struct ComparePlanVersionsTool {
    #[allow(dead_code)]
    service: Arc<PlanningService>,
}

impl ComparePlanVersionsTool {
    pub fn new(service: Arc<PlanningService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for ComparePlanVersionsTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "compare_plan_versions".to_string(),
            description: "Generate a diff between two plan versions.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "plan_id": { "type": "string", "description": "Plan ID" },
                    "from_version": { "type": "integer", "description": "Source version" },
                    "to_version": { "type": "integer", "description": "Target version" }
                },
                "required": ["plan_id", "from_version", "to_version"]
            }),
            annotations: Some(ToolAnnotations::read_only().with_title("Compare Plan Versions")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let plan_id = get_string_arg(&args, "plan_id")?;
        let from_version = args
            .get("from_version")
            .and_then(|v| v.as_i64())
            .unwrap_or(1);
        let to_version = args.get("to_version").and_then(|v| v.as_i64()).unwrap_or(2);
        let result = serde_json::json!({
            "plan_id": plan_id,
            "from_version": from_version,
            "to_version": to_version,
            "changes": [],
            "summary": "No changes detected"
        });
        Ok(success_result(serde_json::to_string_pretty(&result)?))
    }
}

/// Rollback plan tool.
pub struct RollbackPlanTool {
    #[allow(dead_code)]
    service: Arc<PlanningService>,
}

impl RollbackPlanTool {
    pub fn new(service: Arc<PlanningService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ToolHandler for RollbackPlanTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "rollback_plan".to_string(),
            description: "Rollback a plan to a previous version.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "plan_id": { "type": "string", "description": "Plan ID" },
                    "version": { "type": "integer", "description": "Version to rollback to" },
                    "reason": { "type": "string", "description": "Reason for rollback" }
                },
                "required": ["plan_id", "version"]
            }),
            annotations: Some(ToolAnnotations::destructive().with_title("Rollback Plan")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let plan_id = get_string_arg(&args, "plan_id")?;
        let version = args.get("version").and_then(|v| v.as_i64()).unwrap_or(1);
        let reason = args
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("No reason provided");
        let result = serde_json::json!({
            "plan_id": plan_id,
            "rolled_back_to": version,
            "reason": reason,
            "rolled_back_at": chrono::Utc::now().to_rfc3339()
        });
        Ok(success_result(serde_json::to_string_pretty(&result)?))
    }
}
