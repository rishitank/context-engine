//! MCP Prompt Templates
//!
//! Pre-defined prompts that guide AI assistants in common tasks.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A prompt argument definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// A prompt template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub arguments: Vec<PromptArgument>,
}

/// A prompt message (the actual content).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: PromptContent,
}

/// Prompt content types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PromptContent {
    Text {
        text: String,
    },
    Resource {
        uri: String,
        mime_type: Option<String>,
    },
}

/// Result of prompts/list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPromptsResult {
    pub prompts: Vec<Prompt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Result of prompts/get.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptResult {
    pub description: Option<String>,
    pub messages: Vec<PromptMessage>,
}

/// Prompt registry.
#[derive(Debug, Clone, Default)]
pub struct PromptRegistry {
    prompts: HashMap<String, (Prompt, PromptTemplate)>,
}

/// Template for generating prompt messages.
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    pub template: String,
}

impl PromptRegistry {
    /// Create a new registry with built-in prompts.
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register_builtin_prompts();
        registry
    }

    /// Register built-in prompts.
    fn register_builtin_prompts(&mut self) {
        // Code Review Prompt
        self.register(
            Prompt {
                name: "code_review".to_string(),
                description: "Review code for quality, bugs, and best practices".to_string(),
                arguments: vec![
                    PromptArgument {
                        name: "code".to_string(),
                        description: "The code to review".to_string(),
                        required: true,
                    },
                    PromptArgument {
                        name: "language".to_string(),
                        description: "Programming language (optional, auto-detected)".to_string(),
                        required: false,
                    },
                    PromptArgument {
                        name: "focus".to_string(),
                        description: "Areas to focus on (security, performance, style)".to_string(),
                        required: false,
                    },
                ],
            },
            PromptTemplate {
                template: r#"Please review the following code:

```{{language}}
{{code}}
```

{{#if focus}}Focus areas: {{focus}}{{/if}}

Analyze for:
1. Potential bugs or errors
2. Security vulnerabilities
3. Performance issues
4. Code style and best practices
5. Suggestions for improvement"#
                    .to_string(),
            },
        );

        // Explain Code Prompt
        self.register(
            Prompt {
                name: "explain_code".to_string(),
                description: "Explain what a piece of code does".to_string(),
                arguments: vec![
                    PromptArgument {
                        name: "code".to_string(),
                        description: "The code to explain".to_string(),
                        required: true,
                    },
                    PromptArgument {
                        name: "level".to_string(),
                        description: "Explanation level: beginner, intermediate, advanced"
                            .to_string(),
                        required: false,
                    },
                ],
            },
            PromptTemplate {
                template:
                    r#"Please explain the following code{{#if level}} at a {{level}} level{{/if}}:

```
{{code}}
```

Explain:
1. What the code does overall
2. How it works step by step
3. Any important patterns or techniques used"#
                        .to_string(),
            },
        );

        // Write Tests Prompt
        self.register(
            Prompt {
                name: "write_tests".to_string(),
                description: "Generate test cases for code".to_string(),
                arguments: vec![
                    PromptArgument {
                        name: "code".to_string(),
                        description: "The code to test".to_string(),
                        required: true,
                    },
                    PromptArgument {
                        name: "framework".to_string(),
                        description: "Test framework (jest, pytest, cargo test, etc.)".to_string(),
                        required: false,
                    },
                ],
            },
            PromptTemplate {
                template: r#"Generate comprehensive tests for the following code{{#if framework}} using {{framework}}{{/if}}:

```
{{code}}
```

Include:
1. Unit tests for each function/method
2. Edge cases and boundary conditions
3. Error handling tests
4. Integration tests if applicable"#.to_string(),
            },
        );
    }

    /// Register a prompt.
    pub fn register(&mut self, prompt: Prompt, template: PromptTemplate) {
        self.prompts.insert(prompt.name.clone(), (prompt, template));
    }

    /// List all prompts.
    pub fn list(&self) -> Vec<Prompt> {
        self.prompts.values().map(|(p, _)| p.clone()).collect()
    }

    /// Get a prompt by name with arguments substituted.
    pub fn get(&self, name: &str, arguments: &HashMap<String, String>) -> Option<GetPromptResult> {
        self.prompts.get(name).map(|(prompt, template)| {
            let mut text = template.template.clone();

            // Simple template substitution
            for (key, value) in arguments {
                text = text.replace(&format!("{{{{{}}}}}", key), value);
            }

            // Handle conditionals (very simple implementation)
            // {{#if var}}content{{/if}}
            for (key, value) in arguments {
                let if_pattern = format!("{{{{#if {}}}}}", key);
                let endif_pattern = "{{/if}}";

                if let Some(start) = text.find(&if_pattern) {
                    if let Some(end) = text[start..].find(endif_pattern) {
                        let content = &text[start + if_pattern.len()..start + end];
                        if !value.is_empty() {
                            text = text
                                .replace(&text[start..start + end + endif_pattern.len()], content);
                        } else {
                            text =
                                text.replace(&text[start..start + end + endif_pattern.len()], "");
                        }
                    }
                }
            }

            // Clean up remaining template markers
            text = text
                .lines()
                .filter(|line| !line.contains("{{#if") && !line.contains("{{/if}}"))
                .collect::<Vec<_>>()
                .join("\n");

            GetPromptResult {
                description: Some(prompt.description.clone()),
                messages: vec![PromptMessage {
                    role: "user".to_string(),
                    content: PromptContent::Text { text },
                }],
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_prompts() {
        let registry = PromptRegistry::new();
        let prompts = registry.list();
        assert!(!prompts.is_empty());
        assert!(prompts.iter().any(|p| p.name == "code_review"));
    }

    #[test]
    fn test_get_prompt() {
        let registry = PromptRegistry::new();
        let mut args = HashMap::new();
        args.insert("code".to_string(), "fn main() {}".to_string());
        args.insert("language".to_string(), "rust".to_string());

        let result = registry.get("code_review", &args);
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.messages.len(), 1);
    }
}
