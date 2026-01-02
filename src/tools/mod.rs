//! MCP tool implementations.
//!
//! This module contains all 49 MCP tools organized by category:
//!
//! - `retrieval` - Codebase search and context retrieval (6 tools)
//! - `index` - Index management tools (5 tools)
//! - `planning` - AI-powered task planning (20 tools)
//! - `memory` - Persistent memory storage (4 tools)
//! - `review` - Code review tools (14 tools)

pub mod index;
pub mod memory;
pub mod planning;
pub mod retrieval;
pub mod review;

use std::sync::Arc;

use crate::mcp::handler::McpHandler;
use crate::service::{ContextService, MemoryService, PlanningService};

/// Register all tools with the handler.
pub fn register_all_tools(
    handler: &mut McpHandler,
    context_service: Arc<ContextService>,
    memory_service: Arc<MemoryService>,
    planning_service: Arc<PlanningService>,
) {
    // Retrieval tools (6)
    handler.register(retrieval::CodebaseRetrievalTool::new(
        context_service.clone(),
    ));
    handler.register(retrieval::SearchCodeTool::new(context_service.clone()));
    handler.register(retrieval::GetFileTool::new(context_service.clone()));
    handler.register(retrieval::GetContextTool::new(context_service.clone()));
    handler.register(retrieval::EnhancePromptTool::new(context_service.clone()));
    handler.register(retrieval::ToolManifestTool::new());

    // Index tools (5)
    handler.register(index::IndexWorkspaceTool::new(context_service.clone()));
    handler.register(index::IndexStatusTool::new(context_service.clone()));
    handler.register(index::ReindexWorkspaceTool::new(context_service.clone()));
    handler.register(index::ClearIndexTool::new(context_service.clone()));
    handler.register(index::RefreshIndexTool::new(context_service.clone()));

    // Memory tools (4)
    handler.register(memory::StoreMemoryTool::new(memory_service.clone()));
    handler.register(memory::RetrieveMemoryTool::new(memory_service.clone()));
    handler.register(memory::ListMemoryTool::new(memory_service.clone()));
    handler.register(memory::DeleteMemoryTool::new(memory_service.clone()));

    // Planning tools (20)
    handler.register(planning::CreatePlanTool::new(planning_service.clone()));
    handler.register(planning::GetPlanTool::new(planning_service.clone()));
    handler.register(planning::ListPlansTool::new(planning_service.clone()));
    handler.register(planning::AddStepTool::new(planning_service.clone()));
    handler.register(planning::UpdateStepTool::new(planning_service.clone()));
    handler.register(planning::RefinePlanTool::new(planning_service.clone()));
    handler.register(planning::VisualizePlanTool::new(planning_service.clone()));
    handler.register(planning::ExecutePlanTool::new(planning_service.clone()));
    handler.register(planning::SavePlanTool::new(planning_service.clone()));
    handler.register(planning::LoadPlanTool::new(planning_service.clone()));
    handler.register(planning::DeletePlanTool::new(planning_service.clone()));
    handler.register(planning::StartStepTool::new(planning_service.clone()));
    handler.register(planning::CompleteStepTool::new(planning_service.clone()));
    handler.register(planning::FailStepTool::new(planning_service.clone()));
    handler.register(planning::ViewProgressTool::new(planning_service.clone()));
    handler.register(planning::ViewHistoryTool::new(planning_service.clone()));
    handler.register(planning::RequestApprovalTool::new(planning_service.clone()));
    handler.register(planning::RespondApprovalTool::new(planning_service.clone()));
    handler.register(planning::ComparePlanVersionsTool::new(
        planning_service.clone(),
    ));
    handler.register(planning::RollbackPlanTool::new(planning_service.clone()));

    // Review tools (14)
    handler.register(review::ReviewDiffTool::new(context_service.clone()));
    handler.register(review::AnalyzeRiskTool::new(context_service.clone()));
    handler.register(review::ReviewChangesTool::new(context_service.clone()));
    handler.register(review::ReviewGitDiffTool::new(context_service.clone()));
    handler.register(review::ReviewAutoTool::new(context_service.clone()));
    handler.register(review::CheckInvariantsTool::new(context_service.clone()));
    handler.register(review::RunStaticAnalysisTool::new(context_service.clone()));
    handler.register(review::ScrubSecretsTool::new());
    handler.register(review::ValidateContentTool::new());
    handler.register(review::GetReviewStatusTool::new());
    handler.register(review::ReactiveReviewPRTool::new(context_service.clone()));
    handler.register(review::PauseReviewTool::new());
    handler.register(review::ResumeReviewTool::new());
    handler.register(review::GetReviewTelemetryTool::new());
}
