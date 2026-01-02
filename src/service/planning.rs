//! Planning service for AI-powered task planning.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::types::planning::*;

/// Storage for plans.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct PlanStore {
    plans: HashMap<String, Plan>,
}

/// Planning service for managing plans.
pub struct PlanningService {
    store: Arc<RwLock<PlanStore>>,
    storage_path: PathBuf,
}

impl PlanningService {
    /// Create a new planning service.
    pub async fn new(workspace: &Path) -> Result<Self> {
        let storage_path = workspace.join(".context-engine").join("plans.json");
        
        // Create directory if needed
        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Load existing store
        let store = if storage_path.exists() {
            let content = fs::read_to_string(&storage_path).await?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            PlanStore::default()
        };

        Ok(Self {
            store: Arc::new(RwLock::new(store)),
            storage_path,
        })
    }

    /// Save the store to disk.
    async fn save(&self) -> Result<()> {
        let store = self.store.read().await;
        let content = serde_json::to_string_pretty(&*store)?;
        fs::write(&self.storage_path, content).await?;
        Ok(())
    }

    /// Create a new plan.
    pub async fn create_plan(&self, title: String, description: String) -> Result<Plan> {
        let now = chrono::Utc::now().to_rfc3339();
        let id = Uuid::new_v4().to_string();

        let plan = Plan {
            id: id.clone(),
            title,
            description,
            status: PlanStatus::Draft,
            steps: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
            metadata: HashMap::new(),
            risk_assessment: None,
            estimated_duration: None,
            tags: Vec::new(),
        };

        {
            let mut store = self.store.write().await;
            store.plans.insert(id.clone(), plan.clone());
        }

        self.save().await?;
        info!("Created plan: {}", id);
        
        Ok(plan)
    }

    /// Get a plan by ID.
    pub async fn get_plan(&self, id: &str) -> Option<Plan> {
        let store = self.store.read().await;
        store.plans.get(id).cloned()
    }

    /// List all plans.
    pub async fn list_plans(&self, status: Option<PlanStatus>) -> Vec<Plan> {
        let store = self.store.read().await;
        
        store.plans
            .values()
            .filter(|p| status.map_or(true, |s| p.status == s))
            .cloned()
            .collect()
    }

    /// Add a step to a plan.
    pub async fn add_step(&self, plan_id: &str, step: Step) -> Result<Plan> {
        let mut store = self.store.write().await;
        
        let plan = store.plans.get_mut(plan_id)
            .ok_or_else(|| Error::PlanNotFound(plan_id.to_string()))?;

        plan.steps.push(step);
        plan.updated_at = chrono::Utc::now().to_rfc3339();
        
        let plan = plan.clone();
        drop(store);
        
        self.save().await?;
        Ok(plan)
    }

    /// Update step status.
    pub async fn update_step_status(
        &self,
        plan_id: &str,
        step_id: u32,
        status: StepStatus,
    ) -> Result<Plan> {
        let mut store = self.store.write().await;
        
        let plan = store.plans.get_mut(plan_id)
            .ok_or_else(|| Error::PlanNotFound(plan_id.to_string()))?;

        let step = plan.steps.iter_mut()
            .find(|s| s.id == step_id)
            .ok_or_else(|| Error::StepNotFound(step_id))?;

        step.status = status;
        if status == StepStatus::Completed {
            step.completed_at = Some(chrono::Utc::now().to_rfc3339());
        }
        
        plan.updated_at = chrono::Utc::now().to_rfc3339();
        
        let plan = plan.clone();
        drop(store);
        
        self.save().await?;
        Ok(plan)
    }

    /// Delete a plan.
    pub async fn delete_plan(&self, id: &str) -> Result<bool> {
        let removed = {
            let mut store = self.store.write().await;
            store.plans.remove(id).is_some()
        };

        if removed {
            self.save().await?;
            info!("Deleted plan: {}", id);
        }

        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_service() -> (PlanningService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let service = PlanningService::new(temp_dir.path()).await.unwrap();
        (service, temp_dir)
    }

    #[tokio::test]
    async fn test_create_plan() {
        let (service, _temp) = create_test_service().await;

        let plan = service.create_plan(
            "Test Plan".to_string(),
            "A test plan description".to_string(),
        ).await.unwrap();

        assert_eq!(plan.title, "Test Plan");
        assert_eq!(plan.description, "A test plan description");
        assert_eq!(plan.status, PlanStatus::Draft);
        assert!(plan.steps.is_empty());
    }

    #[tokio::test]
    async fn test_get_plan() {
        let (service, _temp) = create_test_service().await;

        let created = service.create_plan("Test".to_string(), "Desc".to_string()).await.unwrap();
        let retrieved = service.get_plan(&created.id).await.unwrap();

        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.title, "Test");
    }

    #[tokio::test]
    async fn test_get_nonexistent_plan() {
        let (service, _temp) = create_test_service().await;
        let result = service.get_plan("nonexistent").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list_plans() {
        let (service, _temp) = create_test_service().await;

        service.create_plan("Plan 1".to_string(), "Desc 1".to_string()).await.unwrap();
        service.create_plan("Plan 2".to_string(), "Desc 2".to_string()).await.unwrap();

        let plans = service.list_plans(None).await;
        assert_eq!(plans.len(), 2);
    }

    #[tokio::test]
    async fn test_list_plans_by_status() {
        let (service, _temp) = create_test_service().await;

        service.create_plan("Plan 1".to_string(), "Desc 1".to_string()).await.unwrap();
        service.create_plan("Plan 2".to_string(), "Desc 2".to_string()).await.unwrap();

        let drafts = service.list_plans(Some(PlanStatus::Draft)).await;
        assert_eq!(drafts.len(), 2);

        let active = service.list_plans(Some(PlanStatus::Active)).await;
        assert!(active.is_empty());
    }

    #[tokio::test]
    async fn test_add_step() {
        let (service, _temp) = create_test_service().await;

        let plan = service.create_plan("Test".to_string(), "Desc".to_string()).await.unwrap();

        let step = Step {
            id: 1,
            title: "Step 1".to_string(),
            description: "First step".to_string(),
            status: StepStatus::Pending,
            step_type: StepType::Implementation,
            dependencies: Vec::new(),
            affected_files: Vec::new(),
            estimated_duration: None,
            actual_duration: None,
            completed_at: None,
            error: None,
            output: None,
            requires_approval: false,
            approval: None,
            rollback: None,
            validation: Vec::new(),
        };

        let updated = service.add_step(&plan.id, step).await.unwrap();
        assert_eq!(updated.steps.len(), 1);
        assert_eq!(updated.steps[0].title, "Step 1");
    }

    #[tokio::test]
    async fn test_update_step_status() {
        let (service, _temp) = create_test_service().await;

        let plan = service.create_plan("Test".to_string(), "Desc".to_string()).await.unwrap();

        let step = Step {
            id: 1,
            title: "Step 1".to_string(),
            description: "First step".to_string(),
            status: StepStatus::Pending,
            step_type: StepType::Implementation,
            dependencies: Vec::new(),
            affected_files: Vec::new(),
            estimated_duration: None,
            actual_duration: None,
            completed_at: None,
            error: None,
            output: None,
            requires_approval: false,
            approval: None,
            rollback: None,
            validation: Vec::new(),
        };

        service.add_step(&plan.id, step).await.unwrap();

        let updated = service.update_step_status(&plan.id, 1, StepStatus::InProgress).await.unwrap();
        assert_eq!(updated.steps[0].status, StepStatus::InProgress);

        let completed = service.update_step_status(&plan.id, 1, StepStatus::Completed).await.unwrap();
        assert_eq!(completed.steps[0].status, StepStatus::Completed);
        assert!(completed.steps[0].completed_at.is_some());
    }

    #[tokio::test]
    async fn test_delete_plan() {
        let (service, _temp) = create_test_service().await;

        let plan = service.create_plan("Test".to_string(), "Desc".to_string()).await.unwrap();
        assert!(service.get_plan(&plan.id).await.is_some());

        let deleted = service.delete_plan(&plan.id).await.unwrap();
        assert!(deleted);
        assert!(service.get_plan(&plan.id).await.is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_plan() {
        let (service, _temp) = create_test_service().await;
        let deleted = service.delete_plan("nonexistent").await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();

        // Create and populate
        let plan_id = {
            let service = PlanningService::new(temp_dir.path()).await.unwrap();
            let plan = service.create_plan("Persistent".to_string(), "Test".to_string()).await.unwrap();
            plan.id
        };

        // Reload and verify
        {
            let service = PlanningService::new(temp_dir.path()).await.unwrap();
            let plan = service.get_plan(&plan_id).await;
            assert!(plan.is_some());
            assert_eq!(plan.unwrap().title, "Persistent");
        }
    }

    #[tokio::test]
    async fn test_add_step_to_nonexistent_plan() {
        let (service, _temp) = create_test_service().await;

        let step = Step {
            id: 1,
            title: "Step".to_string(),
            description: "Desc".to_string(),
            status: StepStatus::Pending,
            step_type: StepType::Implementation,
            dependencies: Vec::new(),
            affected_files: Vec::new(),
            estimated_duration: None,
            actual_duration: None,
            completed_at: None,
            error: None,
            output: None,
            requires_approval: false,
            approval: None,
            rollback: None,
            validation: Vec::new(),
        };

        let result = service.add_step("nonexistent", step).await;
        assert!(result.is_err());
    }
}
