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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::protocol::ContentBlock;
    use crate::mcp::skills::{Skill, SkillMetadata};
    use std::path::PathBuf;

    fn create_test_registry() -> Arc<RwLock<SkillRegistry>> {
        let mut registry = SkillRegistry::new(PathBuf::from("test_skills"));

        // Add test skills
        registry.add_skill(Skill {
            id: "planning".to_string(),
            metadata: SkillMetadata {
                name: "Planning".to_string(),
                description: "Task planning workflow".to_string(),
                category: Some("workflow".to_string()),
                tags: vec!["tasks".to_string(), "planning".to_string()],
                always_apply: false,
            },
            instructions: "# Planning\n\nPlan your tasks carefully.".to_string(),
            path: PathBuf::from("skills/planning/SKILL.md"),
        });

        registry.add_skill(Skill {
            id: "debugging".to_string(),
            metadata: SkillMetadata {
                name: "Debugging".to_string(),
                description: "Debug code systematically".to_string(),
                category: Some("troubleshooting".to_string()),
                tags: vec!["bugs".to_string(), "errors".to_string()],
                always_apply: false,
            },
            instructions: "# Debugging\n\nFind and fix bugs.".to_string(),
            path: PathBuf::from("skills/debugging/SKILL.md"),
        });

        registry.add_skill(Skill {
            id: "testing".to_string(),
            metadata: SkillMetadata {
                name: "Testing".to_string(),
                description: "Write comprehensive tests".to_string(),
                category: Some("quality".to_string()),
                tags: vec!["unit-tests".to_string(), "integration".to_string()],
                always_apply: false,
            },
            instructions: "# Testing\n\nWrite good tests.".to_string(),
            path: PathBuf::from("skills/testing/SKILL.md"),
        });

        Arc::new(RwLock::new(registry))
    }

    fn extract_text(result: &crate::mcp::protocol::ToolResult) -> String {
        match &result.content[0] {
            ContentBlock::Text { text } => text.clone(),
            _ => panic!("Expected text content"),
        }
    }

    // ========== ListSkillsTool Tests ==========

    #[tokio::test]
    async fn test_list_skills_tool_definition() {
        let registry = create_test_registry();
        let tool = ListSkillsTool::new(registry);
        let def = tool.definition();

        assert_eq!(def.name, "list_skills");
        assert!(def.description.contains("List all available skills"));
        assert!(def.annotations.is_some());
    }

    #[tokio::test]
    async fn test_list_skills_all() {
        let registry = create_test_registry();
        let tool = ListSkillsTool::new(registry);

        let args = HashMap::new();
        let result = tool.execute(args).await.unwrap();

        assert!(!result.is_error);
        let text = extract_text(&result);
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["count"], 3);
        assert!(parsed["skills"].is_array());
        assert!(parsed["hint"].as_str().unwrap().contains("load_skill"));
    }

    #[tokio::test]
    async fn test_list_skills_filter_by_category() {
        let registry = create_test_registry();
        let tool = ListSkillsTool::new(registry);

        let mut args = HashMap::new();
        args.insert("category".to_string(), Value::String("quality".to_string()));

        let result = tool.execute(args).await.unwrap();
        let text = extract_text(&result);
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["count"], 1);
        let skills = parsed["skills"].as_array().unwrap();
        assert_eq!(skills[0]["id"], "testing");
    }

    #[tokio::test]
    async fn test_list_skills_filter_no_match() {
        let registry = create_test_registry();
        let tool = ListSkillsTool::new(registry);

        let mut args = HashMap::new();
        args.insert("category".to_string(), Value::String("nonexistent".to_string()));

        let result = tool.execute(args).await.unwrap();
        let text = extract_text(&result);
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["count"], 0);
    }

    // ========== SearchSkillsTool Tests ==========

    #[tokio::test]
    async fn test_search_skills_tool_definition() {
        let registry = create_test_registry();
        let tool = SearchSkillsTool::new(registry);
        let def = tool.definition();

        assert_eq!(def.name, "search_skills");
        assert!(def.description.contains("Search for available skills"));
    }

    #[tokio::test]
    async fn test_search_skills_by_name() {
        let registry = create_test_registry();
        let tool = SearchSkillsTool::new(registry);

        let mut args = HashMap::new();
        args.insert("query".to_string(), Value::String("debugging".to_string()));

        let result = tool.execute(args).await.unwrap();
        let text = extract_text(&result);
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["count"], 1);
        let skills = parsed["skills"].as_array().unwrap();
        assert_eq!(skills[0]["id"], "debugging");
    }

    #[tokio::test]
    async fn test_search_skills_by_tag() {
        let registry = create_test_registry();
        let tool = SearchSkillsTool::new(registry);

        let mut args = HashMap::new();
        args.insert("query".to_string(), Value::String("bugs".to_string()));

        let result = tool.execute(args).await.unwrap();
        let text = extract_text(&result);
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["count"], 1);
    }

    #[tokio::test]
    async fn test_search_skills_no_results() {
        let registry = create_test_registry();
        let tool = SearchSkillsTool::new(registry);

        let mut args = HashMap::new();
        args.insert("query".to_string(), Value::String("xyz123nonexistent".to_string()));

        let result = tool.execute(args).await.unwrap();
        let text = extract_text(&result);
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["count"], 0);
    }

    #[tokio::test]
    async fn test_search_skills_multiple_results() {
        let registry = create_test_registry();
        let tool = SearchSkillsTool::new(registry);

        // Both "testing" and "debugging" contain "ing"
        let mut args = HashMap::new();
        args.insert("query".to_string(), Value::String("ing".to_string()));

        let result = tool.execute(args).await.unwrap();
        let text = extract_text(&result);
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        // All three skills contain "ing" in their names
        assert!(parsed["count"].as_i64().unwrap() >= 2);
    }

    // ========== LoadSkillTool Tests ==========

    #[tokio::test]
    async fn test_load_skill_tool_definition() {
        let registry = create_test_registry();
        let tool = LoadSkillTool::new(registry);
        let def = tool.definition();

        assert_eq!(def.name, "load_skill");
        assert!(def.description.contains("Load full instructions"));
    }

    #[tokio::test]
    async fn test_load_skill_success() {
        let registry = create_test_registry();
        let tool = LoadSkillTool::new(registry);

        let mut args = HashMap::new();
        args.insert("id".to_string(), Value::String("planning".to_string()));

        let result = tool.execute(args).await.unwrap();
        let text = extract_text(&result);
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["id"], "planning");
        assert_eq!(parsed["name"], "Planning");
        assert!(parsed["instructions"].as_str().unwrap().contains("Plan your tasks"));
    }

    #[tokio::test]
    async fn test_load_skill_not_found() {
        let registry = create_test_registry();
        let tool = LoadSkillTool::new(registry);

        let mut args = HashMap::new();
        args.insert("id".to_string(), Value::String("nonexistent".to_string()));

        let result = tool.execute(args).await.unwrap();
        let text = extract_text(&result);
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert!(parsed["error"].as_str().unwrap().contains("not found"));
        assert!(parsed["available_skills"].is_array());
    }

    #[tokio::test]
    async fn test_load_skill_includes_metadata() {
        let registry = create_test_registry();
        let tool = LoadSkillTool::new(registry);

        let mut args = HashMap::new();
        args.insert("id".to_string(), Value::String("debugging".to_string()));

        let result = tool.execute(args).await.unwrap();
        let text = extract_text(&result);
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["id"], "debugging");
        assert_eq!(parsed["name"], "Debugging");
        assert_eq!(parsed["description"], "Debug code systematically");
        assert_eq!(parsed["category"], "troubleshooting");
        assert!(parsed["tags"].is_array());
        assert!(parsed["instructions"].is_string());
    }
}

