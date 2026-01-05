//! Agent Skills support for MCP.
//!
//! This module provides Agent Skills support following the open standard (agentskills.io).
//! Skills are exposed to MCP clients via:
//! 1. MCP Prompts (prompts/list, prompts/get) - native MCP support
//! 2. Tool Search Tool pattern (search_skills, load_skill) - for broader compatibility
//!
//! Skills are loaded from SKILL.md files in the skills directory.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::error::{Error, Result};

/// Skill metadata from SKILL.md frontmatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub always_apply: bool,
}

/// A parsed skill with metadata and instructions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Unique identifier (directory name).
    pub id: String,
    /// Skill metadata from frontmatter.
    pub metadata: SkillMetadata,
    /// Full instructions (markdown body after frontmatter).
    pub instructions: String,
    /// Path to the SKILL.md file.
    pub path: PathBuf,
}

/// Skill registry that loads and manages skills.
#[derive(Debug, Clone, Default)]
pub struct SkillRegistry {
    skills: HashMap<String, Skill>,
    skills_dir: PathBuf,
}

impl SkillRegistry {
    /// Creates a new skill registry with the given skills directory.
    pub fn new(skills_dir: PathBuf) -> Self {
        Self {
            skills: HashMap::new(),
            skills_dir,
        }
    }

    /// Loads all skills from the skills directory.
    pub async fn load_skills(&mut self) -> Result<()> {
        if !self.skills_dir.exists() {
            // Create default skills directory if it doesn't exist
            fs::create_dir_all(&self.skills_dir).await?;
        }

        let mut entries = fs::read_dir(&self.skills_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                let skill_file = path.join("SKILL.md");
                if skill_file.exists() {
                    if let Ok(skill) = self.parse_skill(&skill_file).await {
                        self.skills.insert(skill.id.clone(), skill);
                    }
                }
            }
        }

        Ok(())
    }

    /// Parses a SKILL.md file into a Skill.
    async fn parse_skill(&self, path: &Path) -> Result<Skill> {
        let content = fs::read_to_string(path).await?;
        let id = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Parse YAML frontmatter
        let (metadata, instructions) = Self::parse_frontmatter(&content)?;

        Ok(Skill {
            id,
            metadata,
            instructions,
            path: path.to_path_buf(),
        })
    }

    /// Parses YAML frontmatter from markdown content.
    fn parse_frontmatter(content: &str) -> Result<(SkillMetadata, String)> {
        let content = content.trim();
        if !content.starts_with("---") {
            return Err(Error::Internal(
                "SKILL.md must start with YAML frontmatter (---)".to_string(),
            ));
        }

        // Search for end marker after the opening "---"
        let after_opening = &content[3..];
        let end_marker = after_opening.find("---");

        match end_marker {
            Some(end_pos) => {
                // Bounds check: ensure we have enough content
                // end_pos is relative to after_opening, so yaml_content is [0..end_pos]
                let yaml_content = after_opening[..end_pos].trim();

                // The instructions start after the closing "---" (3 chars)
                // Calculate the absolute position: 3 (opening) + end_pos + 3 (closing)
                let instructions_start = end_pos + 3;
                let instructions = if instructions_start <= after_opening.len() {
                    after_opening[instructions_start..].trim().to_string()
                } else {
                    String::new()
                };

                let metadata: SkillMetadata = serde_yaml::from_str(yaml_content)?;

                Ok((metadata, instructions))
            }
            None => Err(Error::Internal(
                "SKILL.md frontmatter not properly closed (missing ---)".to_string(),
            )),
        }
    }

    /// Lists all loaded skills (metadata only, for search).
    pub fn list(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }

    /// Gets a skill by ID.
    pub fn get(&self, id: &str) -> Option<&Skill> {
        self.skills.get(id)
    }

    /// Searches skills by query (matches name, description, tags).
    pub fn search(&self, query: &str) -> Vec<&Skill> {
        let query_lower = query.to_lowercase();
        self.skills
            .values()
            .filter(|skill| {
                skill.metadata.name.to_lowercase().contains(&query_lower)
                    || skill
                        .metadata
                        .description
                        .to_lowercase()
                        .contains(&query_lower)
                    || skill
                        .metadata
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
                    || skill
                        .metadata
                        .category
                        .as_ref()
                        .is_some_and(|c| c.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Adds a skill directly (for testing).
    #[cfg(test)]
    pub fn add_skill(&mut self, skill: Skill) {
        self.skills.insert(skill.id.clone(), skill);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_skill(
        id: &str,
        name: &str,
        description: &str,
        category: Option<&str>,
        tags: Vec<&str>,
    ) -> Skill {
        Skill {
            id: id.to_string(),
            metadata: SkillMetadata {
                name: name.to_string(),
                description: description.to_string(),
                category: category.map(|s| s.to_string()),
                tags: tags.iter().map(|s| s.to_string()).collect(),
                always_apply: false,
            },
            instructions: format!("# {} Instructions\n\nThis is the {} skill.", name, id),
            path: PathBuf::from(format!("skills/{}/SKILL.md", id)),
        }
    }

    #[test]
    fn test_skill_registry_new() {
        let registry = SkillRegistry::new(PathBuf::from("skills"));
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_skill_registry_add_and_get() {
        let mut registry = SkillRegistry::new(PathBuf::from("skills"));
        let skill = create_test_skill(
            "test",
            "Test Skill",
            "A test skill",
            Some("testing"),
            vec!["test", "unit"],
        );

        registry.add_skill(skill);

        assert_eq!(registry.list().len(), 1);
        let retrieved = registry.get("test").unwrap();
        assert_eq!(retrieved.id, "test");
        assert_eq!(retrieved.metadata.name, "Test Skill");
    }

    #[test]
    fn test_skill_registry_get_nonexistent() {
        let registry = SkillRegistry::new(PathBuf::from("skills"));
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_skill_registry_search_by_name() {
        let mut registry = SkillRegistry::new(PathBuf::from("skills"));
        registry.add_skill(create_test_skill(
            "debug",
            "Debugging",
            "Debug workflow",
            Some("troubleshoot"),
            vec![],
        ));
        registry.add_skill(create_test_skill(
            "review",
            "Code Review",
            "Review code",
            Some("quality"),
            vec![],
        ));

        let results = registry.search("debug");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "debug");
    }

    #[test]
    fn test_skill_registry_search_by_description() {
        let mut registry = SkillRegistry::new(PathBuf::from("skills"));
        registry.add_skill(create_test_skill(
            "test1",
            "Skill 1",
            "workflow for testing",
            None,
            vec![],
        ));
        registry.add_skill(create_test_skill(
            "test2",
            "Skill 2",
            "other purpose",
            None,
            vec![],
        ));

        let results = registry.search("workflow");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "test1");
    }

    #[test]
    fn test_skill_registry_search_by_tag() {
        let mut registry = SkillRegistry::new(PathBuf::from("skills"));
        registry.add_skill(create_test_skill(
            "s1",
            "S1",
            "Desc",
            None,
            vec!["python", "testing"],
        ));
        registry.add_skill(create_test_skill(
            "s2",
            "S2",
            "Desc",
            None,
            vec!["rust", "coding"],
        ));

        let results = registry.search("python");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "s1");
    }

    #[test]
    fn test_skill_registry_search_by_category() {
        let mut registry = SkillRegistry::new(PathBuf::from("skills"));
        registry.add_skill(create_test_skill(
            "s1",
            "S1",
            "Desc",
            Some("quality"),
            vec![],
        ));
        registry.add_skill(create_test_skill(
            "s2",
            "S2",
            "Desc",
            Some("workflow"),
            vec![],
        ));

        let results = registry.search("quality");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "s1");
    }

    #[test]
    fn test_skill_registry_search_case_insensitive() {
        let mut registry = SkillRegistry::new(PathBuf::from("skills"));
        registry.add_skill(create_test_skill(
            "test",
            "DEBUGGING",
            "Find BUGS",
            Some("QUALITY"),
            vec!["ERROR"],
        ));

        assert_eq!(registry.search("debugging").len(), 1);
        assert_eq!(registry.search("bugs").len(), 1);
        assert_eq!(registry.search("quality").len(), 1);
        assert_eq!(registry.search("error").len(), 1);
    }

    #[test]
    fn test_skill_registry_search_no_results() {
        let mut registry = SkillRegistry::new(PathBuf::from("skills"));
        registry.add_skill(create_test_skill(
            "test",
            "Test",
            "Description",
            None,
            vec![],
        ));

        let results = registry.search("nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_skill_registry_search_multiple_results() {
        let mut registry = SkillRegistry::new(PathBuf::from("skills"));
        registry.add_skill(create_test_skill(
            "s1",
            "Code Review",
            "Review",
            Some("quality"),
            vec![],
        ));
        registry.add_skill(create_test_skill(
            "s2",
            "Code Analysis",
            "Analyze",
            Some("quality"),
            vec![],
        ));
        registry.add_skill(create_test_skill(
            "s3",
            "Other",
            "Other",
            Some("other"),
            vec![],
        ));

        let results = registry.search("code");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_parse_frontmatter_valid() {
        let content = r#"---
name: test
description: A test skill
category: testing
tags:
  - unit
  - test
always_apply: false
---

# Test Skill

Instructions here."#;

        let (metadata, instructions) = SkillRegistry::parse_frontmatter(content).unwrap();
        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.description, "A test skill");
        assert_eq!(metadata.category, Some("testing".to_string()));
        assert_eq!(metadata.tags, vec!["unit", "test"]);
        assert!(!metadata.always_apply);
        assert!(instructions.contains("# Test Skill"));
    }

    #[test]
    fn test_parse_frontmatter_minimal() {
        let content = r#"---
name: minimal
description: Minimal skill
---

Content"#;

        let (metadata, instructions) = SkillRegistry::parse_frontmatter(content).unwrap();
        assert_eq!(metadata.name, "minimal");
        assert_eq!(metadata.description, "Minimal skill");
        assert!(metadata.category.is_none());
        assert!(metadata.tags.is_empty());
        assert!(!metadata.always_apply);
        assert_eq!(instructions, "Content");
    }

    #[test]
    fn test_parse_frontmatter_no_start_marker() {
        let content = "No frontmatter here";
        let result = SkillRegistry::parse_frontmatter(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_frontmatter_no_end_marker() {
        let content = "---\nname: test\ndescription: test\n";
        let result = SkillRegistry::parse_frontmatter(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_frontmatter_invalid_yaml() {
        let content = r#"---
name: test
description: [invalid yaml
---

Content"#;
        let result = SkillRegistry::parse_frontmatter(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_frontmatter_always_apply_true() {
        let content = r#"---
name: auto
description: Auto apply skill
always_apply: true
---

Content"#;

        let (metadata, _) = SkillRegistry::parse_frontmatter(content).unwrap();
        assert!(metadata.always_apply);
    }

    #[tokio::test]
    async fn test_load_skills_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        // Create a test skill
        let skill_dir = skills_dir.join("test_skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
name: Test Skill
description: A test skill for testing
category: testing
tags:
  - test
---

# Test Skill

Test instructions."#,
        )
        .unwrap();

        let mut registry = SkillRegistry::new(skills_dir);
        registry.load_skills().await.unwrap();

        assert_eq!(registry.list().len(), 1);
        let skill = registry.get("test_skill").unwrap();
        assert_eq!(skill.metadata.name, "Test Skill");
        assert_eq!(skill.metadata.description, "A test skill for testing");
    }

    #[tokio::test]
    async fn test_load_skills_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join("nonexistent_skills");

        let mut registry = SkillRegistry::new(skills_dir.clone());
        registry.load_skills().await.unwrap();

        assert!(skills_dir.exists());
        assert!(registry.list().is_empty());
    }

    #[tokio::test]
    async fn test_load_skills_ignores_invalid() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        // Create a valid skill
        let valid_dir = skills_dir.join("valid");
        std::fs::create_dir_all(&valid_dir).unwrap();
        std::fs::write(
            valid_dir.join("SKILL.md"),
            r#"---
name: Valid
description: Valid skill
---

Content"#,
        )
        .unwrap();

        // Create an invalid skill (missing frontmatter)
        let invalid_dir = skills_dir.join("invalid");
        std::fs::create_dir_all(&invalid_dir).unwrap();
        std::fs::write(invalid_dir.join("SKILL.md"), "No frontmatter").unwrap();

        // Create a directory without SKILL.md
        let empty_dir = skills_dir.join("empty");
        std::fs::create_dir_all(&empty_dir).unwrap();

        let mut registry = SkillRegistry::new(skills_dir);
        registry.load_skills().await.unwrap();

        // Should only load the valid skill
        assert_eq!(registry.list().len(), 1);
        assert!(registry.get("valid").is_some());
    }

    #[tokio::test]
    async fn test_load_skills_multiple() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        for i in 1..=3 {
            let skill_dir = skills_dir.join(format!("skill{}", i));
            std::fs::create_dir_all(&skill_dir).unwrap();
            std::fs::write(
                skill_dir.join("SKILL.md"),
                format!(
                    r#"---
name: Skill {}
description: Description {}
---

Content {}"#,
                    i, i, i
                ),
            )
            .unwrap();
        }

        let mut registry = SkillRegistry::new(skills_dir);
        registry.load_skills().await.unwrap();

        assert_eq!(registry.list().len(), 3);
    }

    #[test]
    fn test_skill_metadata_serialization() {
        let metadata = SkillMetadata {
            name: "Test".to_string(),
            description: "Test description".to_string(),
            category: Some("quality".to_string()),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            always_apply: true,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: SkillMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, metadata.name);
        assert_eq!(deserialized.description, metadata.description);
        assert_eq!(deserialized.category, metadata.category);
        assert_eq!(deserialized.tags, metadata.tags);
        assert_eq!(deserialized.always_apply, metadata.always_apply);
    }

    #[test]
    fn test_skill_serialization() {
        let skill = create_test_skill("test", "Test", "Desc", Some("cat"), vec!["tag"]);

        let json = serde_json::to_string(&skill).unwrap();
        let deserialized: Skill = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, skill.id);
        assert_eq!(deserialized.metadata.name, skill.metadata.name);
        assert_eq!(deserialized.instructions, skill.instructions);
    }
}
