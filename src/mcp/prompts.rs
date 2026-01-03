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
    /// Creates a new registry populated with the built-in prompts.
    ///
    /// # Examples
    ///
    /// ```
    /// let registry = crate::mcp::prompts::PromptRegistry::new();
    /// assert!(!registry.list().is_empty());
    /// ```
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register_builtin_prompts();
        registry
    }

    /// Populates the registry with the built-in prompt definitions used by the application.
    ///
    /// Registers three prompts — "code_review", "explain_code", and "write_tests" — each with their
    /// argument metadata and template text (including conditional sections and variable placeholders).
    ///
    /// # Examples
    ///
    /// ```
    /// let registry = crate::mcp::prompts::PromptRegistry::new();
    /// let names: Vec<_> = registry.list().into_iter().map(|p| p.name).collect();
    /// assert!(names.contains(&"code_review".to_string()));
    /// ```
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

    /// Adds or updates a prompt and its template in the registry.
    ///
    /// The provided `prompt` is stored under its `name`; if a prompt with the same name
    /// already exists it will be replaced along with its template.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut registry = PromptRegistry::new();
    /// let prompt = Prompt {
    ///     name: "example".to_string(),
    ///     description: "An example prompt".to_string(),
    ///     arguments: vec![],
    /// };
    /// let template = PromptTemplate { template: "Hello {{name}}".to_string() };
    /// registry.register(prompt, template);
    /// assert!(registry.list().iter().any(|p| p.name == "example"));
    /// ```
    pub fn register(&mut self, prompt: Prompt, template: PromptTemplate) {
        self.prompts.insert(prompt.name.clone(), (prompt, template));
    }

    /// Retrieve all registered prompts.
    ///
    /// Returns a vector containing a clone of each registered `Prompt`. The order of prompts is not guaranteed.
    ///
    /// # Examples
    ///
    /// ```
    /// let registry = PromptRegistry::new();
    /// let prompts = registry.list();
    /// assert!(prompts.iter().any(|p| p.name == "code_review"));
    /// ```
    pub fn list(&self) -> Vec<Prompt> {
        self.prompts.values().map(|(p, _)| p.clone()).collect()
    }

    /// Retrieve a registered prompt by name and render its template using the provided arguments.
    ///
    /// The template supports conditional blocks of the form `{{#if var}}...{{/if}}` (the block is included only when `var` is present and not empty) and simple `{{variable}}` substitutions. Any remaining unsubstituted placeholders are removed from the output. Returns `None` if no prompt with the given name exists. On success the result contains the prompt description and a single user-role message with the rendered text.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// let registry = PromptRegistry::new();
    /// let mut args = HashMap::new();
    /// args.insert("code".to_string(), "fn main() {}".to_string());
    /// let res = registry.get("code_review", &args);
    /// assert!(res.is_some());
    /// ```
    pub fn get(&self, name: &str, arguments: &HashMap<String, String>) -> Option<GetPromptResult> {
        self.prompts.get(name).map(|(prompt, template)| {
            let mut text = template.template.clone();

            // First, handle all conditionals (before simple substitution)
            // Find all {{#if var}}...{{/if}} blocks and process them
            loop {
                let if_start = text.find("{{#if ");
                if if_start.is_none() {
                    break;
                }
                let start = if_start.unwrap();

                // Find the variable name
                let var_start = start + 6; // "{{#if " is 6 chars
                let var_end = match text[var_start..].find("}}") {
                    Some(pos) => var_start + pos,
                    None => break,
                };
                let var_name = text[var_start..var_end].trim();

                // Find the matching {{/if}}
                let block_start = var_end + 2; // skip "}}"
                let endif_pos = match text[block_start..].find("{{/if}}") {
                    Some(pos) => block_start + pos,
                    None => break,
                };
                let content = &text[block_start..endif_pos];
                let block_end = endif_pos + 7; // "{{/if}}" is 7 chars

                // Check if the variable is provided and non-empty
                let should_include = arguments
                    .get(var_name)
                    .map(|v| !v.is_empty())
                    .unwrap_or(false);

                if should_include {
                    // Keep the content, remove the markers
                    text = format!("{}{}{}", &text[..start], content, &text[block_end..]);
                } else {
                    // Remove the entire block including markers
                    text = format!("{}{}", &text[..start], &text[block_end..]);
                }
            }

            // Simple template substitution for {{variable}}
            for (key, value) in arguments {
                text = text.replace(&format!("{{{{{}}}}}", key), value);
            }

            // Replace any remaining unsubstituted placeholders with empty string
            // This handles optional arguments that weren't provided
            let placeholder_re = regex::Regex::new(r"\{\{[a-zA-Z_][a-zA-Z0-9_]*\}\}").ok();
            if let Some(re) = placeholder_re {
                text = re.replace_all(&text, "").to_string();
            }

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

    #[test]
    fn test_get_nonexistent_prompt() {
        let registry = PromptRegistry::new();
        let args = HashMap::new();
        let result = registry.get("nonexistent_prompt", &args);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_prompt_missing_required_args() {
        let registry = PromptRegistry::new();
        // code_review requires 'code' and 'language' but we provide neither
        let args = HashMap::new();
        let result = registry.get("code_review", &args);
        assert!(result.is_some());
        // The template placeholders should be removed (empty string replacement)
        let text = match &result.unwrap().messages[0].content {
            PromptContent::Text { text } => text.clone(),
            _ => panic!("Expected text content"),
        };
        // Should not contain unsubstituted placeholders
        assert!(!text.contains("{{code}}"));
        assert!(!text.contains("{{language}}"));
    }

    #[test]
    fn test_get_prompt_empty_arg_values() {
        let registry = PromptRegistry::new();
        let mut args = HashMap::new();
        args.insert("code".to_string(), "".to_string());
        args.insert("language".to_string(), "".to_string());

        let result = registry.get("code_review", &args);
        assert!(result.is_some());
        // Empty values should still work (conditionals should hide content)
    }

    #[test]
    fn test_conditional_with_value() {
        let registry = PromptRegistry::new();
        let mut args = HashMap::new();
        args.insert("code".to_string(), "fn test() {}".to_string());
        args.insert("language".to_string(), "rust".to_string());
        args.insert("focus".to_string(), "security".to_string());

        let result = registry.get("code_review", &args);
        assert!(result.is_some());
        let text = match &result.unwrap().messages[0].content {
            PromptContent::Text { text } => text.clone(),
            _ => panic!("Expected text content"),
        };
        // With focus provided, the conditional content should be included
        assert!(text.contains("security"));
    }

    #[test]
    fn test_conditional_without_value() {
        let registry = PromptRegistry::new();
        let mut args = HashMap::new();
        args.insert("code".to_string(), "fn test() {}".to_string());
        args.insert("language".to_string(), "rust".to_string());
        // Don't provide 'focus' - conditional should be removed

        let result = registry.get("code_review", &args);
        assert!(result.is_some());
        let text = match &result.unwrap().messages[0].content {
            PromptContent::Text { text } => text.clone(),
            _ => panic!("Expected text content"),
        };
        // Without focus, the conditional content should be removed
        assert!(!text.contains("{{#if"));
        assert!(!text.contains("{{/if}}"));
    }
}
