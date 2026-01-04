---
name: refactoring
description: Safe code refactoring workflow with impact analysis
category: quality
tags:
  - refactoring
  - cleanup
  - restructure
  - modernize
  - technical-debt
always_apply: false
---

# Refactoring Skill

Use this skill when you need to refactor code, improve structure, or reduce technical debt.

## When to Use

- Cleaning up duplicated code
- Improving code organization
- Extracting reusable components
- Modernizing legacy patterns
- Reducing complexity
- Improving testability

## Refactoring Workflow

### Phase 1: Assess Impact

1. **Understand the code**:
   ```
   get_file(path: "file/to/refactor.rs")
   file_outline(path: "file/to/refactor.rs")
   ```

2. **Find all usages**:
   ```
   find_references(symbol: "function_to_refactor", workspace: "/path")
   search_callers_for(symbol: "function_name")
   search_importers_for(module: "module_name")
   ```

3. **Check dependencies**:
   ```
   dependency_graph(file_path: "file/to/refactor.rs")
   ```

4. **Find existing tests**:
   ```
   search_tests_for(symbol: "function_to_refactor")
   ```

### Phase 2: Plan Changes

5. **Create a plan**:
   ```
   create_plan(title: "Refactor X", objective: "...", constraints: [...])
   add_step(plan_id: "...", description: "Step 1", step_type: "refactor")
   ```

6. **Identify safe boundaries**: What can change without breaking callers?

7. **Determine migration strategy**: Big bang vs incremental?

### Phase 3: Execute Safely

8. **Make changes incrementally**:
   - One logical change per commit
   - Keep tests passing at each step
   - Update callers before removing old code

9. **Update all references**:
   ```
   find_references(symbol: "old_name", workspace: "/path")
   ```
   Update each caller to use new API

10. **Update tests**: Ensure tests cover new code paths

### Phase 4: Verify

11. **Run tests**: Verify no regressions

12. **Review changes**:
    ```
    review_diff(diff: "...", focus_areas: ["correctness", "performance"])
    ```

13. **Check for missed usages**:
    ```
    search_code(query: "old_function_name OR old_pattern")
    ```

## Common Refactoring Patterns

### Extract Function
- Identify repeated logic
- Create new function with clear name
- Replace all occurrences

### Extract Module
- Group related functions
- Move to new file
- Update imports

### Rename Symbol
- Find all references
- Update definition and all usages
- Update documentation

### Change Signature
- Find all callers
- Update signature
- Update all call sites
- Update tests

### Replace Algorithm
- Identify performance issue
- Implement new algorithm
- Verify same behavior
- Benchmark improvement

### Remove Dead Code
- Use `search_callers_for` to verify no usages
- Remove with tests
- Verify build passes

## Safety Checklist

Before refactoring:
- [ ] Tests exist for affected code
- [ ] All callers identified
- [ ] Dependencies mapped
- [ ] Migration plan documented

After refactoring:
- [ ] All tests pass
- [ ] No new warnings
- [ ] Documentation updated
- [ ] Changelog updated

## Output Format

Provide:
1. **Scope**: Files and symbols affected
2. **Plan**: Step-by-step changes
3. **Impact**: Callers that need updates
4. **Risks**: Potential issues
5. **Verification**: How to verify success

