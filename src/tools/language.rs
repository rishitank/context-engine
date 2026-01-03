//! Language utilities for multi-language symbol detection and file classification.
//!
//! This module provides centralized language detection and symbol extraction
//! across many programming languages.

use serde::{Deserialize, Serialize};

/// A detected symbol in source code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    /// The symbol name
    pub name: String,
    /// The kind of symbol (function, class, struct, etc.)
    pub kind: String,
    /// The 1-based line number where the symbol was found
    pub line: usize,
    /// Optional signature (for functions, methods)
    pub signature: Option<String>,
}

/// Maps a file extension to a canonical language identifier.
///
/// Supports 40+ programming languages and configuration formats.
///
/// # Examples
///
/// ```
/// assert_eq!(extension_to_language("rs"), "rust");
/// assert_eq!(extension_to_language("py"), "python");
/// assert_eq!(extension_to_language("unknown"), "other");
/// ```
pub fn extension_to_language(ext: &str) -> &'static str {
    match ext {
        // Systems programming
        "rs" => "rust",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "hh" => "cpp",
        "go" => "go",
        "zig" => "zig",
        "nim" => "nim",

        // Dynamic/scripting
        "py" | "pyi" | "pyw" => "python",
        "rb" | "rake" | "gemspec" => "ruby",
        "pl" | "pm" | "t" => "perl",
        "php" | "phtml" => "php",
        "lua" => "lua",
        "sh" | "bash" | "zsh" | "fish" | "ksh" => "shell",
        "ps1" | "psm1" => "powershell",

        // JVM languages
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "scala" | "sc" => "scala",
        "groovy" | "gradle" => "groovy",
        "clj" | "cljs" | "cljc" | "edn" => "clojure",

        // .NET languages
        "cs" => "csharp",
        "fs" | "fsi" | "fsx" => "fsharp",
        "vb" => "visualbasic",

        // Web languages
        "js" | "mjs" | "cjs" => "javascript",
        "ts" | "mts" | "cts" => "typescript",
        "tsx" | "jsx" => "react",
        "vue" => "vue",
        "svelte" => "svelte",
        "html" | "htm" | "xhtml" => "html",
        "css" | "scss" | "sass" | "less" | "styl" => "css",

        // Mobile
        "swift" => "swift",
        "m" | "mm" => "objectivec",
        "dart" => "dart",

        // Functional languages
        "hs" | "lhs" => "haskell",
        "ml" | "mli" => "ocaml",
        "ex" | "exs" => "elixir",
        "erl" | "hrl" => "erlang",
        "elm" => "elm",
        // Note: "fs" is already mapped to fsharp above
        "lisp" | "cl" | "lsp" => "lisp",
        "scm" | "ss" => "scheme",
        "rkt" => "racket",

        // Data/Config
        "json" | "jsonc" | "json5" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "xml" | "xsd" | "xsl" | "xslt" => "xml",
        "ini" | "cfg" | "conf" => "config",

        // Query/Markup
        "sql" => "sql",
        "md" | "markdown" | "mdx" => "markdown",
        "rst" => "restructuredtext",
        "tex" | "latex" => "latex",

        // Infrastructure
        "tf" | "tfvars" | "hcl" => "terraform",
        "proto" => "protobuf",
        "graphql" | "gql" => "graphql",
        "dockerfile" => "docker",

        // Statistical/Scientific
        "r" | "R" => "r",
        "jl" => "julia",
        // Note: "m" is already mapped to objectivec above (Objective-C is more common)

        // Other
        "v" | "sv" | "svh" => "verilog",
        "vhd" | "vhdl" => "vhdl",
        "asm" | "s" => "assembly",
        "wasm" | "wat" => "webassembly",
        "sol" => "solidity",
        "move" => "move",
        "cairo" => "cairo",

        _ => "other",
    }
}

/// Normalizes a language hint to a canonical language identifier.
///
/// This function handles cases where users provide file extensions (e.g., "rs", "py")
/// instead of full language names (e.g., "rust", "python"). It also handles
/// common aliases and abbreviations.
///
/// # Examples
///
/// ```
/// use crate::tools::language::normalize_language_hint;
/// assert_eq!(normalize_language_hint("rs"), "rust");
/// assert_eq!(normalize_language_hint("rust"), "rust");
/// assert_eq!(normalize_language_hint("py"), "python");
/// assert_eq!(normalize_language_hint("ts"), "typescript");
/// ```
pub fn normalize_language_hint(hint: &str) -> &'static str {
    let hint_lower = hint.to_lowercase();
    let hint_str = hint_lower.as_str();

    // First, check if it's already a canonical language name
    match hint_str {
        "rust" => "rust",
        "python" => "python",
        "javascript" => "javascript",
        "typescript" => "typescript",
        "go" => "go",
        "java" => "java",
        "kotlin" => "kotlin",
        "scala" => "scala",
        "ruby" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "csharp" => "csharp",
        "fsharp" => "fsharp",
        "cpp" => "cpp",
        "c" => "c",
        "haskell" => "haskell",
        "ocaml" => "ocaml",
        "elixir" => "elixir",
        "erlang" => "erlang",
        "clojure" => "clojure",
        "lua" => "lua",
        "perl" => "perl",
        "shell" => "shell",
        "powershell" => "powershell",
        "sql" => "sql",
        "html" => "html",
        "css" => "css",
        "json" => "json",
        "yaml" => "yaml",
        "toml" => "toml",
        "xml" => "xml",
        "markdown" => "markdown",
        "docker" => "docker",
        "terraform" => "terraform",
        "protobuf" => "protobuf",
        "graphql" => "graphql",
        "react" => "react",
        "vue" => "vue",
        "svelte" => "svelte",
        "dart" => "dart",
        "zig" => "zig",
        "nim" => "nim",
        "julia" => "julia",
        "r" => "r",
        "assembly" => "assembly",
        "verilog" => "verilog",
        "vhdl" => "vhdl",
        "solidity" => "solidity",
        "move" => "move",
        "cairo" => "cairo",
        "objectivec" => "objectivec",
        "visualbasic" => "visualbasic",
        "groovy" => "groovy",
        "config" => "config",
        "restructuredtext" => "restructuredtext",
        "latex" => "latex",
        "webassembly" => "webassembly",
        "make" => "make",
        "cmake" => "cmake",
        "just" => "just",
        "git" => "git",
        "other" => "other",
        // Handle common extensions as hints
        _ => extension_to_language(hint_str),
    }
}

/// Checks if a file's language matches a language hint.
///
/// This function handles the case where users provide either file extensions
/// (e.g., "rs") or full language names (e.g., "rust") as hints.
///
/// # Examples
///
/// ```
/// use crate::tools::language::language_matches_hint;
/// assert!(language_matches_hint("rust", "rs"));
/// assert!(language_matches_hint("rust", "rust"));
/// assert!(language_matches_hint("python", "py"));
/// assert!(!language_matches_hint("rust", "python"));
/// ```
pub fn language_matches_hint(file_lang: &str, hint: &str) -> bool {
    // Direct match
    if file_lang == hint {
        return true;
    }

    // Normalize the hint and compare
    let normalized_hint = normalize_language_hint(hint);
    if file_lang == normalized_hint {
        return true;
    }

    // Check if the file language contains the hint (for partial matches)
    if file_lang.contains(hint) {
        return true;
    }

    false
}

/// Maps an extensionless filename to a language category.
///
/// Recognizes common configuration and build files without extensions.
/// Returns `None` if the filename is not recognized.
pub fn filename_to_language(name: &str) -> Option<&'static str> {
    match name {
        // Build systems
        "Makefile" | "makefile" | "GNUmakefile" => Some("make"),
        "CMakeLists.txt" => Some("cmake"),
        "Rakefile" | "Gemfile" => Some("ruby"),
        "Justfile" | "justfile" => Some("just"),

        // Containers/Infra
        "Dockerfile" | "Containerfile" => Some("docker"),
        "docker-compose.yml" | "docker-compose.yaml" => Some("docker-compose"),
        "Vagrantfile" => Some("ruby"),

        // CI/CD
        "Jenkinsfile" => Some("groovy"),
        ".travis.yml" => Some("yaml"),
        ".gitlab-ci.yml" => Some("yaml"),

        // Git
        ".gitignore" | ".gitattributes" | ".gitmodules" => Some("git"),

        // Environment/Config
        ".env" | ".env.local" | ".env.development" | ".env.production" => Some("env"),
        ".editorconfig" => Some("editorconfig"),
        ".prettierrc" | ".eslintrc" => Some("json"),

        // Package managers
        "Cargo.toml" => Some("toml"),
        "pyproject.toml" | "setup.py" | "setup.cfg" => Some("python"),
        "package.json" | "package-lock.json" => Some("json"),
        "requirements.txt" | "Pipfile" => Some("python"),
        "go.mod" | "go.sum" => Some("go"),
        "pom.xml" | "build.gradle" | "build.gradle.kts" => Some("java"),
        "Podfile" | "Podfile.lock" => Some("ruby"),
        "Cartfile" => Some("swift"),

        // Shell config
        ".bashrc" | ".bash_profile" | ".zshrc" | ".profile" => Some("shell"),

        _ => None,
    }
}

// ===== Symbol Detection =====

/// Detect symbols in a line of code based on the file extension.
///
/// Supports: Rust, Python, TypeScript/JavaScript, Go, Java, Ruby, C/C++, C#,
/// Swift, Kotlin, Scala, PHP, Elixir, Haskell, and more.
pub fn detect_symbol(line: &str, ext: &str, line_num: usize) -> Option<Symbol> {
    match ext {
        "rs" => detect_rust_symbol(line, line_num),
        "py" | "pyi" => detect_python_symbol(line, line_num),
        "ts" | "tsx" | "js" | "jsx" | "mjs" | "mts" => detect_ts_symbol(line, line_num),
        "go" => detect_go_symbol(line, line_num),
        "java" => detect_java_symbol(line, line_num),
        "rb" | "rake" => detect_ruby_symbol(line, line_num),
        "c" | "h" | "cpp" | "cc" | "hpp" | "cxx" => detect_c_cpp_symbol(line, line_num),
        "cs" => detect_csharp_symbol(line, line_num),
        "swift" => detect_swift_symbol(line, line_num),
        "kt" | "kts" => detect_kotlin_symbol(line, line_num),
        "scala" | "sc" => detect_scala_symbol(line, line_num),
        "php" => detect_php_symbol(line, line_num),
        "ex" | "exs" => detect_elixir_symbol(line, line_num),
        "hs" | "lhs" => detect_haskell_symbol(line, line_num),
        "lua" => detect_lua_symbol(line, line_num),
        "dart" => detect_dart_symbol(line, line_num),
        "clj" | "cljs" | "cljc" => detect_clojure_symbol(line, line_num),
        "sh" | "bash" | "zsh" | "fish" => detect_shell_symbol(line, line_num),
        _ => None,
    }
}

/// Extract a symbol name from a line after a keyword prefix.
fn extract_name(line: &str, prefix: &str) -> String {
    let rest = line.split(prefix).nth(1).unwrap_or("");
    rest.chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// Extract a name with generics support (e.g., `Foo<T>` -> `Foo`).
fn extract_name_before_generic(line: &str, prefix: &str) -> String {
    let rest = line.split(prefix).nth(1).unwrap_or("");
    rest.chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

// ===== Rust =====

fn detect_rust_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    let trimmed = line.trim_start();

    // Functions
    if trimmed.starts_with("pub fn ")
        || trimmed.starts_with("fn ")
        || trimmed.starts_with("pub async fn ")
        || trimmed.starts_with("async fn ")
        || trimmed.starts_with("pub(crate) fn ")
        || trimmed.starts_with("pub(super) fn ")
    {
        let name = extract_name(trimmed, "fn ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: if trimmed.contains("async") {
                    "async_function".to_string()
                } else {
                    "function".to_string()
                },
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Structs
    if trimmed.starts_with("pub struct ") || trimmed.starts_with("struct ") {
        let name = extract_name_before_generic(trimmed, "struct ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "struct".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Enums
    if trimmed.starts_with("pub enum ") || trimmed.starts_with("enum ") {
        let name = extract_name_before_generic(trimmed, "enum ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "enum".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Traits
    if trimmed.starts_with("pub trait ") || trimmed.starts_with("trait ") {
        let name = extract_name_before_generic(trimmed, "trait ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "trait".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Impl blocks
    if trimmed.starts_with("impl ") || trimmed.starts_with("impl<") {
        // Handle impl<T> Trait for Type and impl Type patterns
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            // Find the type being implemented
            let name = if trimmed.contains(" for ") {
                // impl Trait for Type
                trimmed.split(" for ").nth(1).and_then(|s| {
                    s.split_whitespace()
                        .next()
                        .map(|n| n.trim_end_matches(['<', '{']))
                })
            } else {
                // impl Type or impl<T> Type<T>
                parts
                    .get(1)
                    .map(|s| s.trim_start_matches('<').split('<').next().unwrap_or(*s))
            };

            if let Some(n) = name {
                let clean_name: String = n
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !clean_name.is_empty() {
                    return Some(Symbol {
                        name: clean_name,
                        kind: "impl".to_string(),
                        line: line_num,
                        signature: Some(line.to_string()),
                    });
                }
            }
        }
    }

    // Type aliases
    if trimmed.starts_with("pub type ") || trimmed.starts_with("type ") {
        let name = extract_name(trimmed, "type ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "type_alias".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Constants
    if trimmed.starts_with("pub const ") || trimmed.starts_with("const ") {
        let name = extract_name(trimmed, "const ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "constant".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Statics
    if trimmed.starts_with("pub static ") || trimmed.starts_with("static ") {
        let name = extract_name(trimmed, "static ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "static".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Macros
    if trimmed.starts_with("macro_rules! ") {
        let name = extract_name(trimmed, "macro_rules! ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "macro".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    None
}

// ===== Python =====

fn detect_python_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    let trimmed = line.trim_start();

    // Functions
    if trimmed.starts_with("def ") {
        let name = extract_name(trimmed, "def ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "function".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Async functions
    if trimmed.starts_with("async def ") {
        let name = extract_name(trimmed, "async def ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "async_function".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Classes
    if trimmed.starts_with("class ") {
        let name = extract_name(trimmed, "class ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "class".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    None
}

// ===== TypeScript/JavaScript =====

fn detect_ts_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    let trimmed = line.trim_start();

    // Function declarations (avoid false positives from comments/strings)
    let is_function_decl = trimmed.starts_with("function ")
        || trimmed.starts_with("export function ")
        || trimmed.starts_with("async function ")
        || trimmed.starts_with("export async function ")
        || trimmed.starts_with("export default function ");

    if is_function_decl {
        let name = extract_name(trimmed, "function ");
        if !name.is_empty() {
            let kind = if trimmed.contains("async ") {
                "async_function"
            } else {
                "function"
            };
            return Some(Symbol {
                name,
                kind: kind.to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Class declarations
    if trimmed.starts_with("class ")
        || trimmed.starts_with("export class ")
        || trimmed.starts_with("abstract class ")
        || trimmed.starts_with("export abstract class ")
        || trimmed.starts_with("export default class ")
    {
        let name = extract_name(trimmed, "class ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "class".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Interface declarations
    if trimmed.starts_with("interface ") || trimmed.starts_with("export interface ") {
        let name = extract_name(trimmed, "interface ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "interface".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Type aliases
    if trimmed.starts_with("type ") || trimmed.starts_with("export type ") {
        let name = extract_name(trimmed, "type ");
        if !name.is_empty() && !trimmed.contains("typeof") {
            return Some(Symbol {
                name,
                kind: "type".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Enum declarations
    if trimmed.starts_with("enum ") || trimmed.starts_with("export enum ") {
        let name = extract_name(trimmed, "enum ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "enum".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Arrow function constants (top-level only)
    if (trimmed.starts_with("const ") || trimmed.starts_with("export const "))
        && (trimmed.contains(" = (") || trimmed.contains(" = async ("))
        && trimmed.contains("=>")
    {
        let name = extract_name(trimmed, "const ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "arrow_function".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    None
}

// ===== Go =====

fn detect_go_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    let trimmed = line.trim_start();

    // Functions and methods
    if trimmed.starts_with("func ") {
        let rest = trimmed.strip_prefix("func ").unwrap_or("");

        let name = if rest.starts_with('(') {
            // Method: func (r *Receiver) MethodName(...)
            rest.split(')')
                .nth(1)
                .and_then(|s| s.trim().split('(').next())
                .map(|s| s.trim().to_string())
        } else {
            // Function: func FuncName(...)
            rest.split('(').next().map(|s| s.trim().to_string())
        };

        if let Some(n) = name {
            if !n.is_empty() {
                return Some(Symbol {
                    name: n,
                    kind: "function".to_string(),
                    line: line_num,
                    signature: Some(line.to_string()),
                });
            }
        }
    }

    // Structs
    if trimmed.starts_with("type ") && trimmed.contains(" struct") {
        let name = extract_name(trimmed, "type ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "struct".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Interfaces
    if trimmed.starts_with("type ") && trimmed.contains(" interface") {
        let name = extract_name(trimmed, "type ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "interface".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Type aliases
    if trimmed.starts_with("type ")
        && !trimmed.contains(" struct")
        && !trimmed.contains(" interface")
    {
        let name = extract_name(trimmed, "type ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "type_alias".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Constants
    if trimmed.starts_with("const ") && trimmed.contains('=') {
        let name = extract_name(trimmed, "const ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "constant".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Variables
    if trimmed.starts_with("var ") && trimmed.contains('=') {
        let name = extract_name(trimmed, "var ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "variable".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    None
}

// ===== Java =====

fn detect_java_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    let trimmed = line.trim_start();

    // Skip annotations
    if trimmed.starts_with('@') {
        return None;
    }

    // Class declarations
    if (trimmed.contains("class ") && trimmed.contains('{'))
        || (trimmed.contains("class ") && !trimmed.contains('('))
    {
        if let Some(idx) = trimmed.find("class ") {
            let rest = &trimmed[idx + 6..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(Symbol {
                    name,
                    kind: "class".to_string(),
                    line: line_num,
                    signature: None,
                });
            }
        }
    }

    // Interface declarations
    if trimmed.contains("interface ") && !trimmed.contains('(') {
        if let Some(idx) = trimmed.find("interface ") {
            let rest = &trimmed[idx + 10..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(Symbol {
                    name,
                    kind: "interface".to_string(),
                    line: line_num,
                    signature: None,
                });
            }
        }
    }

    // Enum declarations
    if trimmed.contains("enum ") && !trimmed.contains('(') {
        if let Some(idx) = trimmed.find("enum ") {
            let rest = &trimmed[idx + 5..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(Symbol {
                    name,
                    kind: "enum".to_string(),
                    line: line_num,
                    signature: None,
                });
            }
        }
    }

    // Method declarations (public/private/protected ... type name(...))
    if trimmed.contains('(')
        && trimmed.contains(')')
        && !trimmed.starts_with("if")
        && !trimmed.starts_with("while")
        && !trimmed.starts_with("for")
    {
        // Look for method pattern: modifiers + return_type + name(
        let parts: Vec<&str> = trimmed.split('(').collect();
        if !parts.is_empty() {
            let before_paren = parts[0].trim();
            let tokens: Vec<&str> = before_paren.split_whitespace().collect();
            if tokens.len() >= 2 {
                let last = tokens.last().unwrap();
                let name: String = last
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !name.is_empty()
                    && name
                        .chars()
                        .next()
                        .map(|c| c.is_lowercase())
                        .unwrap_or(false)
                {
                    return Some(Symbol {
                        name,
                        kind: "method".to_string(),
                        line: line_num,
                        signature: Some(line.to_string()),
                    });
                }
            }
        }
    }

    None
}

// ===== Ruby =====

fn detect_ruby_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    let trimmed = line.trim_start();

    // Methods
    if trimmed.starts_with("def ") {
        let name = extract_name(trimmed, "def ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "method".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Classes
    if trimmed.starts_with("class ") {
        let name = extract_name(trimmed, "class ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "class".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Modules
    if trimmed.starts_with("module ") {
        let name = extract_name(trimmed, "module ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "module".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    None
}

// ===== C/C++ =====

fn detect_c_cpp_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    let trimmed = line.trim_start();

    // Skip preprocessor directives
    if trimmed.starts_with('#') {
        return None;
    }

    // Class declarations (C++)
    if trimmed.starts_with("class ") && !trimmed.contains(';') {
        let name = extract_name(trimmed, "class ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "class".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Struct declarations
    if trimmed.starts_with("struct ") && !trimmed.contains(';') {
        let name = extract_name(trimmed, "struct ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "struct".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Enum declarations
    if trimmed.starts_with("enum ") && !trimmed.contains(';') {
        let rest = if trimmed.contains("enum class ") {
            trimmed.strip_prefix("enum class ").unwrap_or("")
        } else {
            trimmed.strip_prefix("enum ").unwrap_or("")
        };
        let name: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "enum".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Namespace (C++)
    if trimmed.starts_with("namespace ") && !trimmed.contains(';') {
        let name = extract_name(trimmed, "namespace ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "namespace".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Template (C++)
    if trimmed.starts_with("template") {
        return Some(Symbol {
            name: "template".to_string(),
            kind: "template".to_string(),
            line: line_num,
            signature: Some(line.to_string()),
        });
    }

    // Function declarations (simplified: type name(...) {)
    if trimmed.contains('(')
        && (trimmed.ends_with('{') || trimmed.ends_with(')'))
        && !trimmed.contains(';')
    {
        let parts: Vec<&str> = trimmed.split('(').collect();
        if !parts.is_empty() {
            let before_paren = parts[0].trim();
            let tokens: Vec<&str> = before_paren.split_whitespace().collect();
            if !tokens.is_empty() {
                let last = tokens.last().unwrap();
                // Handle pointer/reference decorations
                let name: String = last
                    .trim_start_matches('*')
                    .trim_start_matches('&')
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !name.is_empty()
                    && name != "if"
                    && name != "while"
                    && name != "for"
                    && name != "switch"
                {
                    return Some(Symbol {
                        name,
                        kind: "function".to_string(),
                        line: line_num,
                        signature: Some(line.to_string()),
                    });
                }
            }
        }
    }

    None
}

// ===== C# =====

fn detect_csharp_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    let trimmed = line.trim_start();

    // Skip attributes
    if trimmed.starts_with('[') {
        return None;
    }

    // Class declarations
    if trimmed.contains("class ") && !trimmed.contains(';') {
        if let Some(idx) = trimmed.find("class ") {
            let rest = &trimmed[idx + 6..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(Symbol {
                    name,
                    kind: "class".to_string(),
                    line: line_num,
                    signature: None,
                });
            }
        }
    }

    // Interface declarations
    if trimmed.contains("interface ") && !trimmed.contains(';') {
        if let Some(idx) = trimmed.find("interface ") {
            let rest = &trimmed[idx + 10..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(Symbol {
                    name,
                    kind: "interface".to_string(),
                    line: line_num,
                    signature: None,
                });
            }
        }
    }

    // Struct declarations
    if trimmed.contains("struct ") && !trimmed.contains(';') {
        if let Some(idx) = trimmed.find("struct ") {
            let rest = &trimmed[idx + 7..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(Symbol {
                    name,
                    kind: "struct".to_string(),
                    line: line_num,
                    signature: None,
                });
            }
        }
    }

    // Enum declarations
    if trimmed.contains("enum ") && !trimmed.contains(';') {
        if let Some(idx) = trimmed.find("enum ") {
            let rest = &trimmed[idx + 5..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(Symbol {
                    name,
                    kind: "enum".to_string(),
                    line: line_num,
                    signature: None,
                });
            }
        }
    }

    // Namespace
    if trimmed.starts_with("namespace ") {
        let name = extract_name(trimmed, "namespace ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "namespace".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    None
}

// ===== Swift =====

fn detect_swift_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    let trimmed = line.trim_start();

    // Functions
    if trimmed.starts_with("func ") || trimmed.contains(" func ") {
        let name = extract_name(trimmed, "func ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "function".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Classes
    if trimmed.starts_with("class ") || trimmed.contains(" class ") {
        let name = extract_name(trimmed, "class ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "class".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Structs
    if trimmed.starts_with("struct ") || trimmed.contains(" struct ") {
        let name = extract_name(trimmed, "struct ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "struct".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Enums
    if trimmed.starts_with("enum ") || trimmed.contains(" enum ") {
        let name = extract_name(trimmed, "enum ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "enum".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Protocols
    if trimmed.starts_with("protocol ") || trimmed.contains(" protocol ") {
        let name = extract_name(trimmed, "protocol ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "protocol".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Extensions
    if trimmed.starts_with("extension ") {
        let name = extract_name(trimmed, "extension ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "extension".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    None
}

// ===== Kotlin =====

fn detect_kotlin_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    let trimmed = line.trim_start();

    // Functions
    if trimmed.starts_with("fun ") || trimmed.contains(" fun ") {
        let name = extract_name(trimmed, "fun ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "function".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Data classes (check before generic class to avoid false match)
    if trimmed.contains("data class ") {
        let name = extract_name(trimmed, "data class ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "data_class".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Sealed classes (check before generic class to avoid false match)
    if trimmed.contains("sealed class ") {
        let name = extract_name(trimmed, "sealed class ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "sealed_class".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Classes (generic - after specific class types)
    if trimmed.contains("class ") && !trimmed.contains(';') {
        let name = extract_name(trimmed, "class ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "class".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Objects (singletons)
    if trimmed.starts_with("object ") || trimmed.contains(" object ") {
        let name = extract_name(trimmed, "object ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "object".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Interfaces
    if trimmed.starts_with("interface ") || trimmed.contains(" interface ") {
        let name = extract_name(trimmed, "interface ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "interface".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    None
}

// ===== Scala =====

fn detect_scala_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    let trimmed = line.trim_start();

    // Definitions
    if trimmed.starts_with("def ") || trimmed.contains(" def ") {
        let name = extract_name(trimmed, "def ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "function".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Classes
    if trimmed.contains("class ") && !trimmed.contains(';') {
        let name = extract_name(trimmed, "class ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "class".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Objects
    if trimmed.starts_with("object ") || trimmed.contains(" object ") {
        let name = extract_name(trimmed, "object ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "object".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Traits
    if trimmed.starts_with("trait ") || trimmed.contains(" trait ") {
        let name = extract_name(trimmed, "trait ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "trait".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Case classes
    if trimmed.contains("case class ") {
        let name = extract_name(trimmed, "case class ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "case_class".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    None
}

// ===== PHP =====

fn detect_php_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    let trimmed = line.trim_start();

    // Functions
    if trimmed.starts_with("function ") || trimmed.contains(" function ") {
        let name = extract_name(trimmed, "function ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "function".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Classes
    if trimmed.contains("class ") && !trimmed.contains(';') {
        let name = extract_name(trimmed, "class ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "class".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Interfaces
    if trimmed.starts_with("interface ") || trimmed.contains(" interface ") {
        let name = extract_name(trimmed, "interface ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "interface".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Traits
    if trimmed.starts_with("trait ") {
        let name = extract_name(trimmed, "trait ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "trait".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    None
}

// ===== Elixir =====

fn detect_elixir_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    let trimmed = line.trim_start();

    // Functions
    if trimmed.starts_with("def ") || trimmed.starts_with("defp ") {
        let keyword = if trimmed.starts_with("defp ") {
            "defp "
        } else {
            "def "
        };
        let name = extract_name(trimmed, keyword);
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: if keyword == "defp " {
                    "private_function"
                } else {
                    "function"
                }
                .to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Modules
    if trimmed.starts_with("defmodule ") {
        let rest = trimmed.strip_prefix("defmodule ").unwrap_or("");
        let name: String = rest.split_whitespace().next().unwrap_or("").to_string();
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "module".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Macros
    if trimmed.starts_with("defmacro ") || trimmed.starts_with("defmacrop ") {
        let keyword = if trimmed.starts_with("defmacrop ") {
            "defmacrop "
        } else {
            "defmacro "
        };
        let name = extract_name(trimmed, keyword);
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "macro".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    None
}

// ===== Haskell =====

fn detect_haskell_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    let trimmed = line.trim_start();

    // Skip if it's indented (likely part of a definition body)
    if line.starts_with(' ') || line.starts_with('\t') {
        return None;
    }

    // Type signatures (name :: Type)
    if trimmed.contains(" :: ") {
        let name: String = trimmed
            .split(" :: ")
            .next()
            .unwrap_or("")
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '\'')
            .collect();
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "type_signature".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Data types
    if trimmed.starts_with("data ") {
        let name = extract_name(trimmed, "data ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "data".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Newtypes
    if trimmed.starts_with("newtype ") {
        let name = extract_name(trimmed, "newtype ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "newtype".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Type aliases
    if trimmed.starts_with("type ") {
        let name = extract_name(trimmed, "type ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "type_alias".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Classes
    if trimmed.starts_with("class ") {
        let name = extract_name(trimmed, "class ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "class".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Instances
    if trimmed.starts_with("instance ") {
        return Some(Symbol {
            name: "instance".to_string(),
            kind: "instance".to_string(),
            line: line_num,
            signature: Some(line.to_string()),
        });
    }

    // Module declarations
    if trimmed.starts_with("module ") {
        let rest = trimmed.strip_prefix("module ").unwrap_or("");
        let name: String = rest.split_whitespace().next().unwrap_or("").to_string();
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "module".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    None
}

// ===== Lua =====

fn detect_lua_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    let trimmed = line.trim_start();

    // Functions
    if trimmed.starts_with("function ") || trimmed.starts_with("local function ") {
        let keyword = if trimmed.starts_with("local function ") {
            "local function "
        } else {
            "function "
        };
        let rest = trimmed.strip_prefix(keyword).unwrap_or("");
        let name: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.' || *c == ':')
            .collect();
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "function".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    None
}

// ===== Dart =====

fn detect_dart_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    let trimmed = line.trim_start();

    // Classes
    if trimmed.starts_with("class ") || trimmed.contains(" class ") {
        let name = extract_name(trimmed, "class ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "class".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Abstract classes
    if trimmed.starts_with("abstract class ") {
        let name = extract_name(trimmed, "abstract class ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "abstract_class".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Mixins
    if trimmed.starts_with("mixin ") {
        let name = extract_name(trimmed, "mixin ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "mixin".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Enums
    if trimmed.starts_with("enum ") {
        let name = extract_name(trimmed, "enum ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "enum".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Extension
    if trimmed.starts_with("extension ") {
        let name = extract_name(trimmed, "extension ");
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "extension".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Top-level functions (simple heuristic)
    if trimmed.contains('(')
        && !trimmed.starts_with("if")
        && !trimmed.starts_with("while")
        && !trimmed.starts_with("for")
    {
        let parts: Vec<&str> = trimmed.split('(').collect();
        if !parts.is_empty() {
            let before = parts[0].trim();
            let tokens: Vec<&str> = before.split_whitespace().collect();
            if tokens.len() >= 2 {
                let last = tokens.last().unwrap();
                let name: String = last
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !name.is_empty()
                    && name
                        .chars()
                        .next()
                        .map(|c| c.is_lowercase())
                        .unwrap_or(false)
                {
                    return Some(Symbol {
                        name,
                        kind: "function".to_string(),
                        line: line_num,
                        signature: Some(line.to_string()),
                    });
                }
            }
        }
    }

    None
}

// ===== Clojure =====

fn detect_clojure_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    let trimmed = line.trim_start();

    // Functions
    if trimmed.starts_with("(defn ") || trimmed.starts_with("(defn- ") {
        let keyword = if trimmed.starts_with("(defn- ") {
            "(defn- "
        } else {
            "(defn "
        };
        let rest = trimmed.strip_prefix(keyword).unwrap_or("");
        let name: String = rest
            .chars()
            .take_while(|c| !c.is_whitespace() && *c != '[' && *c != '(')
            .collect();
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: if keyword == "(defn- " {
                    "private_function"
                } else {
                    "function"
                }
                .to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Macros
    if trimmed.starts_with("(defmacro ") {
        let rest = trimmed.strip_prefix("(defmacro ").unwrap_or("");
        let name: String = rest
            .chars()
            .take_while(|c| !c.is_whitespace() && *c != '[' && *c != '(')
            .collect();
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "macro".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Protocols
    if trimmed.starts_with("(defprotocol ") {
        let rest = trimmed.strip_prefix("(defprotocol ").unwrap_or("");
        let name: String = rest.chars().take_while(|c| !c.is_whitespace()).collect();
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "protocol".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Records
    if trimmed.starts_with("(defrecord ") {
        let rest = trimmed.strip_prefix("(defrecord ").unwrap_or("");
        let name: String = rest
            .chars()
            .take_while(|c| !c.is_whitespace() && *c != '[')
            .collect();
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "record".to_string(),
                line: line_num,
                signature: None,
            });
        }
    }

    // Multimethods
    if trimmed.starts_with("(defmulti ") {
        let rest = trimmed.strip_prefix("(defmulti ").unwrap_or("");
        let name: String = rest.chars().take_while(|c| !c.is_whitespace()).collect();
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "multimethod".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    None
}

// ===== Shell =====

fn detect_shell_symbol(line: &str, line_num: usize) -> Option<Symbol> {
    let trimmed = line.trim_start();

    // Function definitions (both styles)
    // Style 1: function name() { or function name {
    if trimmed.starts_with("function ") {
        let rest = trimmed.strip_prefix("function ").unwrap_or("");
        let name: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "function".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    // Style 2: name() {
    if trimmed.contains("()") && (trimmed.ends_with('{') || trimmed.ends_with("{ ")) {
        let name: String = trimmed
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            return Some(Symbol {
                name,
                kind: "function".to_string(),
                line: line_num,
                signature: Some(line.to_string()),
            });
        }
    }

    None
}

// ============================================================================
// Definition Patterns for go_to_definition
// ============================================================================

/// Returns regex patterns for finding symbol definitions in a given language.
/// Each pattern should capture the symbol name in a way that can be matched.
pub fn get_definition_patterns(language: &str, symbol: &str) -> Vec<String> {
    let escaped = regex::escape(symbol);

    match language {
        "rust" => vec![
            format!(r"fn\s+{}\s*[<(]", escaped),
            format!(r"struct\s+{}\s*[<{{]", escaped),
            format!(r"enum\s+{}\s*[<{{]", escaped),
            format!(r"trait\s+{}\s*[<{{:]", escaped),
            format!(r"impl\s+.*{}\s*[<{{]", escaped),
            format!(r"type\s+{}\s*[<=]", escaped),
            format!(r"const\s+{}\s*:", escaped),
            format!(r"static\s+{}\s*:", escaped),
            format!(r"mod\s+{}\s*[{{;]", escaped),
            format!(r"macro_rules!\s+{}", escaped),
        ],
        "python" => vec![
            format!(r"def\s+{}\s*\(", escaped),
            format!(r"async\s+def\s+{}\s*\(", escaped),
            format!(r"class\s+{}\s*[:\(]", escaped),
            format!(r"{}\s*=", escaped),
        ],
        "typescript" | "javascript" => vec![
            format!(r"function\s+{}\s*[<(]", escaped),
            format!(r"async\s+function\s+{}\s*[<(]", escaped),
            format!(r"class\s+{}\s*[<{{]", escaped),
            format!(r"interface\s+{}\s*[<{{]", escaped),
            format!(r"type\s+{}\s*[<=]", escaped),
            format!(r"enum\s+{}\s*{{", escaped),
            format!(r"const\s+{}\s*[=:]", escaped),
            format!(r"let\s+{}\s*[=:]", escaped),
            format!(r"var\s+{}\s*[=:]", escaped),
            format!(r"export\s+(default\s+)?function\s+{}\s*[<(]", escaped),
            format!(r"export\s+(default\s+)?class\s+{}\s*[<{{]", escaped),
            format!(r"export\s+interface\s+{}\s*[<{{]", escaped),
            format!(r"export\s+type\s+{}\s*[<=]", escaped),
            format!(r"{}\s*:\s*function", escaped),
            format!(r"{}\s*=\s*\(", escaped),
            format!(r"{}\s*=\s*async\s*\(", escaped),
        ],
        "go" => vec![
            format!(r"func\s+{}\s*\(", escaped),
            format!(r"func\s+\([^)]+\)\s+{}\s*\(", escaped),
            format!(r"type\s+{}\s+struct", escaped),
            format!(r"type\s+{}\s+interface", escaped),
            format!(r"type\s+{}\s+=", escaped),
            format!(r"const\s+{}\s*=", escaped),
            format!(r"var\s+{}\s+", escaped),
        ],
        "java" => vec![
            format!(r"class\s+{}\s*[<{{]", escaped),
            format!(r"interface\s+{}\s*[<{{]", escaped),
            format!(r"enum\s+{}\s*{{", escaped),
            format!(r"\s+{}\s*\([^)]*\)\s*{{", escaped),
            format!(r"\s+{}\s*\([^)]*\)\s*throws", escaped),
        ],
        "ruby" => vec![
            format!(r"def\s+{}\s*[\(;]?", escaped),
            format!(r"class\s+{}\s*[<;]?", escaped),
            format!(r"module\s+{}", escaped),
        ],
        "c" | "cpp" | "c++" => vec![
            format!(r"\s+{}\s*\([^)]*\)\s*{{", escaped),
            format!(r"class\s+{}\s*[<{{:]", escaped),
            format!(r"struct\s+{}\s*{{", escaped),
            format!(r"enum\s+(class\s+)?{}\s*{{", escaped),
            format!(r"namespace\s+{}\s*{{", escaped),
            format!(r"typedef\s+.*{}\s*;", escaped),
            format!(r"#define\s+{}", escaped),
        ],
        "csharp" | "c#" => vec![
            format!(r"class\s+{}\s*[<{{:]", escaped),
            format!(r"interface\s+{}\s*[<{{:]", escaped),
            format!(r"struct\s+{}\s*[<{{:]", escaped),
            format!(r"enum\s+{}\s*{{", escaped),
            format!(r"namespace\s+{}", escaped),
            format!(r"\s+{}\s*\([^)]*\)\s*{{", escaped),
        ],
        "swift" => vec![
            format!(r"func\s+{}\s*[<(]", escaped),
            format!(r"class\s+{}\s*[<{{:]", escaped),
            format!(r"struct\s+{}\s*[<{{:]", escaped),
            format!(r"enum\s+{}\s*[<{{:]", escaped),
            format!(r"protocol\s+{}\s*[<{{:]", escaped),
            format!(r"extension\s+{}", escaped),
        ],
        "kotlin" => vec![
            format!(r"fun\s+{}\s*[<(]", escaped),
            format!(r"class\s+{}\s*[<({{:]", escaped),
            format!(r"data\s+class\s+{}\s*[<(]", escaped),
            format!(r"sealed\s+class\s+{}", escaped),
            format!(r"object\s+{}\s*[{{:]", escaped),
            format!(r"interface\s+{}\s*[<{{:]", escaped),
        ],
        "scala" => vec![
            format!(r"def\s+{}\s*[<\[(]", escaped),
            format!(r"class\s+{}\s*[<\[({{]", escaped),
            format!(r"case\s+class\s+{}\s*[<\[(]", escaped),
            format!(r"object\s+{}\s*[{{]", escaped),
            format!(r"trait\s+{}\s*[<{{]", escaped),
        ],
        "php" => vec![
            format!(r"function\s+{}\s*\(", escaped),
            format!(r"class\s+{}\s*[{{]", escaped),
            format!(r"interface\s+{}\s*[{{]", escaped),
            format!(r"trait\s+{}\s*[{{]", escaped),
        ],
        "elixir" => vec![
            format!(r"def\s+{}\s*[\(,]", escaped),
            format!(r"defp\s+{}\s*[\(,]", escaped),
            format!(r"defmodule\s+{}", escaped),
            format!(r"defmacro\s+{}\s*[\(,]", escaped),
        ],
        "haskell" => vec![
            format!(r"{}\s+::", escaped),
            format!(r"data\s+{}", escaped),
            format!(r"newtype\s+{}", escaped),
            format!(r"type\s+{}", escaped),
            format!(r"class\s+.*{}", escaped),
        ],
        "lua" => vec![
            format!(r"function\s+{}\s*\(", escaped),
            format!(r"local\s+function\s+{}\s*\(", escaped),
            format!(r"{}\s*=\s*function", escaped),
        ],
        "dart" => vec![
            format!(r"class\s+{}\s*[<{{]", escaped),
            format!(r"abstract\s+class\s+{}\s*[<{{]", escaped),
            format!(r"mixin\s+{}\s*[<{{]", escaped),
            format!(r"enum\s+{}\s*{{", escaped),
            format!(r"\s+{}\s*\([^)]*\)\s*{{", escaped),
        ],
        "clojure" => vec![
            format!(r"\(defn\s+{}", escaped),
            format!(r"\(defn-\s+{}", escaped),
            format!(r"\(defmacro\s+{}", escaped),
            format!(r"\(defprotocol\s+{}", escaped),
            format!(r"\(defrecord\s+{}", escaped),
        ],
        "shell" | "bash" | "sh" | "zsh" => vec![
            format!(r"function\s+{}", escaped),
            format!(r"{}\s*\(\)\s*{{", escaped),
        ],
        _ => vec![
            // Generic fallback patterns
            format!(r"(fn|func|function|def)\s+{}\s*[\(<]", escaped),
            format!(
                r"(class|struct|interface|trait|type)\s+{}\s*[<{{:]",
                escaped
            ),
        ],
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_to_language() {
        assert_eq!(extension_to_language("rs"), "rust");
        assert_eq!(extension_to_language("py"), "python");
        assert_eq!(extension_to_language("ts"), "typescript");
        assert_eq!(extension_to_language("tsx"), "react"); // tsx/jsx are React
        assert_eq!(extension_to_language("jsx"), "react");
        assert_eq!(extension_to_language("js"), "javascript");
        assert_eq!(extension_to_language("go"), "go");
        assert_eq!(extension_to_language("java"), "java");
        assert_eq!(extension_to_language("rb"), "ruby");
        assert_eq!(extension_to_language("cpp"), "cpp");
        assert_eq!(extension_to_language("cs"), "csharp");
        assert_eq!(extension_to_language("swift"), "swift");
        assert_eq!(extension_to_language("kt"), "kotlin");
        assert_eq!(extension_to_language("scala"), "scala");
        assert_eq!(extension_to_language("php"), "php");
        assert_eq!(extension_to_language("ex"), "elixir");
        assert_eq!(extension_to_language("hs"), "haskell");
        assert_eq!(extension_to_language("lua"), "lua");
        assert_eq!(extension_to_language("dart"), "dart");
        assert_eq!(extension_to_language("clj"), "clojure");
        assert_eq!(extension_to_language("sh"), "shell");
        assert_eq!(extension_to_language("unknown"), "other");
    }

    #[test]
    fn test_filename_to_language() {
        assert_eq!(filename_to_language("Makefile"), Some("make"));
        assert_eq!(filename_to_language("Dockerfile"), Some("docker"));
        assert_eq!(filename_to_language("Jenkinsfile"), Some("groovy"));
        assert_eq!(filename_to_language("Vagrantfile"), Some("ruby"));
        assert_eq!(filename_to_language("Gemfile"), Some("ruby"));
        assert_eq!(filename_to_language("Rakefile"), Some("ruby"));
        assert_eq!(filename_to_language(".gitignore"), Some("git"));
        assert_eq!(filename_to_language(".bashrc"), Some("shell"));
        assert_eq!(filename_to_language(".zshrc"), Some("shell"));
        assert_eq!(filename_to_language("random_file"), None);
    }

    #[test]
    fn test_detect_rust_symbol() {
        let sym = detect_symbol("pub fn process_data(input: &str) -> Result<()> {", "rs", 1);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "process_data");
        assert_eq!(s.kind, "function");

        let sym = detect_symbol("pub struct Config {", "rs", 2);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "Config");
        assert_eq!(s.kind, "struct");

        let sym = detect_symbol("pub enum Status {", "rs", 3);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "Status");
        assert_eq!(s.kind, "enum");

        let sym = detect_symbol("pub trait Handler {", "rs", 4);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "Handler");
        assert_eq!(s.kind, "trait");
    }

    #[test]
    fn test_detect_python_symbol() {
        let sym = detect_symbol("def process_data(input):", "py", 1);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "process_data");
        assert_eq!(s.kind, "function");

        let sym = detect_symbol("async def fetch_data():", "py", 2);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "fetch_data");
        assert_eq!(s.kind, "async_function");

        let sym = detect_symbol("class DataProcessor:", "py", 3);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "DataProcessor");
        assert_eq!(s.kind, "class");
    }

    #[test]
    fn test_detect_typescript_symbol() {
        let sym = detect_symbol("function processData(input: string): void {", "ts", 1);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "processData");
        assert_eq!(s.kind, "function");

        let sym = detect_symbol(
            "export async function fetchData(): Promise<void> {",
            "ts",
            2,
        );
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "fetchData");
        assert_eq!(s.kind, "async_function"); // Async functions are distinguished

        let sym = detect_symbol("class DataProcessor {", "ts", 3);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "DataProcessor");
        assert_eq!(s.kind, "class");

        let sym = detect_symbol("interface Config {", "ts", 4);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "Config");
        assert_eq!(s.kind, "interface");

        let sym = detect_symbol("type Result = string | number;", "ts", 5);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "Result");
        assert_eq!(s.kind, "type");
    }

    #[test]
    fn test_detect_go_symbol() {
        let sym = detect_symbol("func ProcessData(input string) error {", "go", 1);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "ProcessData");
        assert_eq!(s.kind, "function");

        let sym = detect_symbol("func (s *Server) Handle(req Request) {", "go", 2);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "Handle");
        assert_eq!(s.kind, "function"); // Methods are also detected as functions

        let sym = detect_symbol("type Config struct {", "go", 3);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "Config");
        assert_eq!(s.kind, "struct");

        let sym = detect_symbol("type Handler interface {", "go", 4);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "Handler");
        assert_eq!(s.kind, "interface");
    }

    #[test]
    fn test_detect_java_symbol() {
        let sym = detect_symbol("public class DataProcessor {", "java", 1);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "DataProcessor");
        assert_eq!(s.kind, "class");

        let sym = detect_symbol("public interface Handler {", "java", 2);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "Handler");
        assert_eq!(s.kind, "interface");

        let sym = detect_symbol("public enum Status {", "java", 3);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "Status");
        assert_eq!(s.kind, "enum");
    }

    #[test]
    fn test_detect_ruby_symbol() {
        let sym = detect_symbol("def process_data(input)", "rb", 1);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "process_data");
        assert_eq!(s.kind, "method");

        let sym = detect_symbol("class DataProcessor", "rb", 2);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "DataProcessor");
        assert_eq!(s.kind, "class");

        let sym = detect_symbol("module Helpers", "rb", 3);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "Helpers");
        assert_eq!(s.kind, "module");
    }

    #[test]
    fn test_detect_kotlin_symbol() {
        let sym = detect_symbol("fun processData(input: String): Result {", "kt", 1);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "processData");
        assert_eq!(s.kind, "function");

        let sym = detect_symbol("data class Config(val name: String)", "kt", 2);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "Config");
        assert_eq!(s.kind, "data_class");

        let sym = detect_symbol("sealed class Result {", "kt", 3);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "Result");
        assert_eq!(s.kind, "sealed_class");
    }

    #[test]
    fn test_detect_elixir_symbol() {
        let sym = detect_symbol("def process_data(input) do", "ex", 1);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "process_data");
        assert_eq!(s.kind, "function");

        let sym = detect_symbol("defp private_helper(x) do", "ex", 2);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "private_helper");
        assert_eq!(s.kind, "private_function");

        let sym = detect_symbol("defmodule MyApp.DataProcessor do", "ex", 3);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "MyApp.DataProcessor");
        assert_eq!(s.kind, "module");
    }

    #[test]
    fn test_detect_clojure_symbol() {
        let sym = detect_symbol("(defn process-data [input]", "clj", 1);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "process-data");
        assert_eq!(s.kind, "function");

        let sym = detect_symbol("(defn- private-helper [x]", "clj", 2);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "private-helper");
        assert_eq!(s.kind, "private_function");

        let sym = detect_symbol("(defprotocol Handler", "clj", 3);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "Handler");
        assert_eq!(s.kind, "protocol");
    }

    #[test]
    fn test_detect_shell_symbol() {
        let sym = detect_symbol("function process_data() {", "sh", 1);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "process_data");
        assert_eq!(s.kind, "function");

        let sym = detect_symbol("my_function() {", "sh", 2);
        assert!(sym.is_some());
        let s = sym.unwrap();
        assert_eq!(s.name, "my_function");
        assert_eq!(s.kind, "function");
    }

    #[test]
    fn test_get_definition_patterns() {
        let patterns = get_definition_patterns("rust", "process_data");
        assert!(!patterns.is_empty());
        assert!(patterns.iter().any(|p| p.contains("fn")));

        let patterns = get_definition_patterns("python", "process_data");
        assert!(!patterns.is_empty());
        assert!(patterns.iter().any(|p| p.contains("def")));

        let patterns = get_definition_patterns("typescript", "processData");
        assert!(!patterns.is_empty());
        assert!(patterns.iter().any(|p| p.contains("function")));

        let patterns = get_definition_patterns("unknown_lang", "symbol");
        assert!(!patterns.is_empty()); // Should return fallback patterns
    }

    #[test]
    fn test_normalize_language_hint() {
        // Extension to canonical name
        assert_eq!(normalize_language_hint("rs"), "rust");
        assert_eq!(normalize_language_hint("py"), "python");
        assert_eq!(normalize_language_hint("ts"), "typescript");
        assert_eq!(normalize_language_hint("js"), "javascript");
        assert_eq!(normalize_language_hint("go"), "go");
        assert_eq!(normalize_language_hint("kt"), "kotlin");
        assert_eq!(normalize_language_hint("cs"), "csharp");

        // Already canonical names
        assert_eq!(normalize_language_hint("rust"), "rust");
        assert_eq!(normalize_language_hint("python"), "python");
        assert_eq!(normalize_language_hint("typescript"), "typescript");

        // Case insensitive
        assert_eq!(normalize_language_hint("RS"), "rust");
        assert_eq!(normalize_language_hint("Rust"), "rust");
        assert_eq!(normalize_language_hint("PYTHON"), "python");
    }

    #[test]
    fn test_language_matches_hint() {
        // Direct match
        assert!(language_matches_hint("rust", "rust"));
        assert!(language_matches_hint("python", "python"));

        // Extension hint matches canonical name
        assert!(language_matches_hint("rust", "rs"));
        assert!(language_matches_hint("python", "py"));
        assert!(language_matches_hint("typescript", "ts"));
        assert!(language_matches_hint("javascript", "js"));

        // Partial match (contains)
        assert!(language_matches_hint("typescript", "script"));

        // Non-matches
        assert!(!language_matches_hint("rust", "python"));
        assert!(!language_matches_hint("rust", "py"));
        assert!(!language_matches_hint("javascript", "ts"));
    }
}
