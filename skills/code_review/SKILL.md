---
name: code_review
description: Comprehensive code review workflow for analyzing diffs, identifying risks, checking invariants, and ensuring code quality.
category: quality
tags:
  - code-review
  - quality
  - security
  - best-practices
always_apply: false
---

# Code Review Skill

Use this skill when reviewing code changes, analyzing risks, or ensuring code quality.

## When to Use

- Reviewing a pull request or diff
- Analyzing risk of proposed changes
- Checking for security vulnerabilities
- Validating code against project standards
- Generating review summaries

## Workflow

### 1. Review the Diff

Start by reviewing the actual code changes:

```
review_diff(
  diff: "...",  # The unified diff content
  context: "Adding new authentication endpoint"
)
```

### 2. Analyze Risk

Assess the risk level of the changes:

```
analyze_risk(
  files: ["src/auth/login.rs", "src/db/users.rs"],
  change_description: "Modified authentication flow and user queries"
)
```

Risk levels: `low`, `medium`, `high`, `critical`

### 3. Check Invariants

Verify that important invariants are maintained:

```
check_invariants(
  files: ["src/auth/login.rs"],
  invariants: [
    "All database queries must use parameterized statements",
    "Authentication tokens must be validated before use"
  ]
)
```

### 4. Review for Specific Concerns

Use specialized review tools:

```
review_security(files: ["src/auth/login.rs"])
review_performance(files: ["src/db/queries.rs"])
review_tests(files: ["tests/auth_test.rs"])
```

### 5. Generate Summary

Create a comprehensive review summary:

```
generate_review_summary(
  files: ["src/auth/login.rs", "src/db/users.rs"],
  findings: [...],
  recommendation: "approve" | "request_changes" | "comment"
)
```

## Available Tools

| Tool | Purpose |
|------|---------|
| `review_diff` | Review a unified diff |
| `analyze_risk` | Assess change risk level |
| `review_changes` | Review file changes in context |
| `check_invariants` | Verify code invariants |
| `review_security` | Security-focused review |
| `review_performance` | Performance-focused review |
| `review_tests` | Test coverage review |
| `review_documentation` | Documentation review |
| `suggest_improvements` | Generate improvement suggestions |
| `check_style` | Check code style compliance |
| `find_similar_code` | Find similar patterns in codebase |
| `check_breaking_changes` | Identify breaking API changes |
| `generate_review_summary` | Create review summary |
| `create_review_checklist` | Generate review checklist |

## Review Checklist

When reviewing code, check for:

### Correctness
- [ ] Logic is correct and handles edge cases
- [ ] Error handling is appropriate
- [ ] No off-by-one errors or boundary issues

### Security
- [ ] No SQL injection vulnerabilities
- [ ] Input validation is present
- [ ] Sensitive data is handled properly
- [ ] Authentication/authorization is correct

### Performance
- [ ] No N+1 query problems
- [ ] Appropriate caching is used
- [ ] No unnecessary allocations in hot paths

### Maintainability
- [ ] Code is readable and well-documented
- [ ] Functions are appropriately sized
- [ ] Naming is clear and consistent

### Testing
- [ ] New code has tests
- [ ] Edge cases are tested
- [ ] Tests are meaningful, not just coverage

## Example: PR Review

```
# 1. Get the diff
diff = get_diff_from_pr()

# 2. Initial review
review_diff(diff: diff, context: "PR #123: Add user preferences API")

# 3. Identify affected files
files = ["src/api/preferences.rs", "src/db/preferences.rs", "tests/preferences_test.rs"]

# 4. Risk analysis
analyze_risk(files: files, change_description: "New API endpoint with database changes")

# 5. Security review (since it's an API endpoint)
review_security(files: ["src/api/preferences.rs"])

# 6. Check test coverage
review_tests(files: ["tests/preferences_test.rs"])

# 7. Generate summary
generate_review_summary(
  files: files,
  findings: [...],
  recommendation: "approve"
)
```

## Best Practices

1. **Start broad, then focus**: Review diff first, then dive into specific concerns
2. **Consider context**: Understand why changes were made
3. **Be constructive**: Suggest improvements, don't just criticize
4. **Prioritize findings**: Focus on high-impact issues first
5. **Check tests**: Ensure changes are properly tested

