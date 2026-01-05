---
name: debugging
description: Systematic debugging workflow for identifying and fixing bugs
category: troubleshooting
tags:
  - debugging
  - bugs
  - errors
  - troubleshooting
  - diagnostics
always_apply: false
---

# Debugging Skill

Use this skill when you need to debug code, investigate errors, or troubleshoot issues.

## When to Use

- User reports an error or unexpected behavior
- Tests are failing
- Runtime exceptions or crashes
- Performance issues or slowdowns
- Inconsistent or incorrect output

## Debugging Workflow

### Phase 1: Gather Information

1. **Understand the symptom**: What exactly is failing? Get error messages, stack traces, logs.

2. **Reproduce the issue**: Can you reliably trigger the bug?

3. **Search for context**:
   ```
   codebase_retrieval(query: "error message or symptom description")
   search_code(query: "relevant function or module name")
   ```

### Phase 2: Locate the Bug

4. **Find related code**:
   ```
   get_file(path: "file/with/error.rs", start_line: X, end_line: Y)
   find_references(symbol: "function_name", workspace: "/path")
   go_to_definition(symbol: "suspicious_function", workspace: "/path")
   ```

5. **Check call hierarchy**:
   ```
   search_callers_for(symbol: "failing_function")
   search_importers_for(module: "problematic_module")
   ```

6. **Review git history** (if regression):
   ```
   git_log(path: "affected/file.rs", max_commits: 10)
   git_blame(path: "affected/file.rs", start_line: X, end_line: Y)
   ```

### Phase 3: Analyze Root Cause

7. **Check dependencies**:
   ```
   dependency_graph(file_path: "affected/file.rs")
   ```

8. **Review related tests**:
   ```
   search_tests_for(symbol: "failing_function")
   ```

9. **Check configuration**:
   ```
   search_config_for(key: "relevant_config_key")
   ```

### Phase 4: Fix and Verify

10. **Propose a fix**: Based on analysis, suggest minimal code change

11. **Verify fix**:
    - Run affected tests
    - Check for regressions
    - Review impact on callers

## Common Bug Patterns

### Null/None Errors
- Check for missing Option/Result unwrapping
- Look for uninitialized variables
- Verify API responses are validated

### Type Mismatches
- Check function signatures changed
- Verify serialization/deserialization
- Look for implicit conversions

### Race Conditions
- Check async/await patterns
- Look for shared mutable state
- Verify lock ordering

### Memory Issues
- Check for leaks (unclosed resources)
- Look for unbounded growth (caches, buffers)
- Verify cleanup in error paths

### Logic Errors
- Check boundary conditions
- Verify loop termination
- Look for off-by-one errors

## Output Format

After debugging, provide:

1. **Root Cause**: Clear explanation of what caused the bug
2. **Location**: Exact file and line numbers
3. **Fix**: Proposed code change
4. **Verification**: How to verify the fix works
5. **Prevention**: How to prevent similar bugs

