---
name: documentation
description: Documentation generation and maintenance workflow
category: quality
tags:
  - documentation
  - docs
  - readme
  - api-docs
  - comments
always_apply: false
---

# Documentation Skill

Use this skill when you need to create, update, or improve documentation.

## When to Use

- Creating README files
- Writing API documentation
- Adding code comments
- Creating user guides
- Documenting architecture decisions
- Writing changelog entries

## Documentation Workflow

### Phase 1: Understand the Code

1. **Get project overview**:
   ```
   workspace_stats()
   ```

2. **Understand structure**:
   ```
   file_outline(path: "main/entry/point.rs")
   dependency_graph(file_path: "core/module.rs")
   ```

3. **Read existing docs**:
   ```
   get_file(path: "README.md")
   get_file(path: "docs/API.md")
   ```

### Phase 2: Gather Information

4. **Find public APIs**:
   ```
   codebase_retrieval(query: "public functions and exports")
   search_code(query: "pub fn OR export")
   ```

5. **Check existing usage**:
   ```
   search_tests_for(symbol: "main_function")
   ```

6. **Review git history**:
   ```
   git_log(path: ".", max_commits: 20)
   ```

### Phase 3: Write Documentation

7. **Follow existing style**: Match the project's documentation conventions

8. **Structure content**:
   - Overview/Introduction
   - Installation/Setup
   - Usage/Examples
   - API Reference
   - Configuration
   - Troubleshooting

9. **Include examples**: Use real code from tests when possible

### Phase 4: Verify

10. **Check accuracy**: Verify examples compile/run

11. **Review for completeness**:
    ```
    codebase_retrieval(query: "undocumented public functions")
    ```

## Documentation Types

### README.md
- Project description
- Quick start guide
- Installation instructions
- Basic usage examples
- Links to detailed docs

### API Reference
- Function signatures
- Parameter descriptions
- Return values
- Error conditions
- Usage examples

### Code Comments
- Why, not what
- Complex algorithm explanations
- Edge case handling
- TODO/FIXME with context

### Architecture Docs
- System overview
- Component relationships
- Data flow
- Design decisions

### Changelog
- Version number
- Date
- Added/Changed/Fixed/Removed
- Breaking changes highlighted

## Best Practices

### Be Concise
- Lead with the most important information
- Use bullet points for lists
- Include code examples

### Be Accurate
- Test all code examples
- Update docs with code changes
- Remove outdated information

### Be Consistent
- Follow existing style
- Use same terminology
- Match formatting conventions

### Be Complete
- Document all public APIs
- Include error handling
- Cover edge cases

## Output Format

When creating documentation:

1. **Type**: What kind of documentation
2. **Location**: Where it should live
3. **Content**: The documentation itself
4. **Verification**: How to verify accuracy

