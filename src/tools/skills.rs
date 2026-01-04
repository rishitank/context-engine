//! Skills tools for MCP.
//!
//! Implements the Tool Search Tool pattern for exposing Agent Skills to MCP clients.
//! - `search_skills`: Find skills by query (returns metadata only)
//! - `load_skill`: Load full skill instructions by ID

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::Result;
use crate::mcp::handler::{get_optional_string_arg, get_string_arg, success_result, ToolHandler};
use crate::mcp::protocol::{Tool, ToolAnnotations, ToolResult};
use crate::mcp::skills::SkillRegistry;

/// Search skills tool - finds skills by query.
pub struct SearchSkillsTool {
    registry: Arc<RwLock<SkillRegistry>>,
}

impl SearchSkillsTool {
    pub fn new(registry: Arc<RwLock<SkillRegistry>>) -> Self {
        Self { registry }
    }
}

#[async_trait]
impl ToolHandler for SearchSkillsTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "search_skills".to_string(),
            description: "Search for available skills by query. Returns skill metadata (name, description, tags) without full instructions. Use load_skill to get full instructions for a specific skill.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query to find relevant skills (matches name, description, tags, category)"
                    }
                },
                "required": ["query"]
            }),
            annotations: Some(ToolAnnotations::read_only().with_title("Search Skills")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let query = get_string_arg(&args, "query")?;
        let registry = self.registry.read().await;
        let skills = registry.search(&query);

        let results: Vec<serde_json::Value> = skills
            .iter()
            .map(|skill| {
                serde_json::json!({
                    "id": skill.id,
                    "name": skill.metadata.name,
                    "description": skill.metadata.description,
                    "category": skill.metadata.category,
                    "tags": skill.metadata.tags,
                    "always_apply": skill.metadata.always_apply
                })
            })
            .collect();

        let response = serde_json::json!({
            "skills": results,
            "count": results.len(),
            "hint": "Use load_skill(id) to get full instructions for a skill"
        });

        Ok(success_result(serde_json::to_string_pretty(&response)?))
    }
}

/// List skills tool - lists all available skills.
pub struct ListSkillsTool {
    registry: Arc<RwLock<SkillRegistry>>,
}

impl ListSkillsTool {
    pub fn new(registry: Arc<RwLock<SkillRegistry>>) -> Self {
        Self { registry }
    }
}

#[async_trait]
impl ToolHandler for ListSkillsTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "list_skills".to_string(),
            description: "List all available skills with their metadata. Use this to discover what skills are available.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "category": {
                        "type": "string",
                        "description": "Optional category filter"
                    }
                }
            }),
            annotations: Some(ToolAnnotations::read_only().with_title("List Skills")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let category_filter = get_optional_string_arg(&args, "category");
        let registry = self.registry.read().await;
        let all_skills = registry.list();

        let skills: Vec<_> = all_skills
            .iter()
            .filter(|s| {
                category_filter
                    .as_ref()
                    .is_none_or(|cat| s.metadata.category.as_ref().is_some_and(|c| c == cat))
            })
            .map(|skill| {
                serde_json::json!({
                    "id": skill.id,
                    "name": skill.metadata.name,
                    "description": skill.metadata.description,
                    "category": skill.metadata.category,
                    "tags": skill.metadata.tags,
                    "always_apply": skill.metadata.always_apply
                })
            })
            .collect();

        let response = serde_json::json!({
            "skills": skills,
            "count": skills.len(),
            "hint": "Use load_skill(id) to get full instructions for a skill"
        });

        Ok(success_result(serde_json::to_string_pretty(&response)?))
    }
}

/// Load skill tool - loads full skill instructions.
pub struct LoadSkillTool {
    registry: Arc<RwLock<SkillRegistry>>,
}

impl LoadSkillTool {
    pub fn new(registry: Arc<RwLock<SkillRegistry>>) -> Self {
        Self { registry }
    }
}

#[async_trait]
impl ToolHandler for LoadSkillTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "load_skill".to_string(),
            description: "Load full instructions for a skill by ID. Use search_skills or list_skills first to find the skill ID.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "The skill ID to load (e.g., 'planning', 'code_review')"
                    }
                },
                "required": ["id"]
            }),
            annotations: Some(ToolAnnotations::read_only().with_title("Load Skill")),
            ..Default::default()
        }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<ToolResult> {
        let id = get_string_arg(&args, "id")?;
        let registry = self.registry.read().await;

        match registry.get(&id) {
            Some(skill) => {
                let response = serde_json::json!({
                    "id": skill.id,
                    "name": skill.metadata.name,
                    "description": skill.metadata.description,
                    "category": skill.metadata.category,
                    "tags": skill.metadata.tags,
                    "instructions": skill.instructions
                });
                Ok(success_result(serde_json::to_string_pretty(&response)?))
            }
            None => {
                let available: Vec<_> = registry.list().iter().map(|s| &s.id).collect();
                let response = serde_json::json!({
                    "error": format!("Skill '{}' not found", id),
                    "available_skills": available
                });
                Ok(success_result(serde_json::to_string_pretty(&response)?))
            }
        }
    }
}

