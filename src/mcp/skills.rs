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

        let end_marker = content[3..].find("---");
        match end_marker {
            Some(end_pos) => {
                let yaml_content = &content[3..end_pos + 3].trim();
                let instructions = content[end_pos + 6..].trim().to_string();

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
}

