---
name: testing
description: Comprehensive test writing and maintenance workflow
category: quality
tags:
  - testing
  - unit-tests
  - integration-tests
  - tdd
  - test-coverage
always_apply: false
---

# Testing Skill

Use this skill when you need to write, update, or improve tests.

## When to Use

- Writing tests for new code
- Adding tests for untested code
- Fixing or updating existing tests
- Improving test coverage
- Writing integration tests
- Test-Driven Development (TDD)

## Testing Workflow

### Phase 1: Understand What to Test

1. **Get code to test**:
   ```
   get_file(path: "code/to/test.rs")
   file_outline(path: "code/to/test.rs")
   ```

2. **Find existing tests**:
   ```
   search_tests_for(symbol: "function_to_test")
   ```

3. **Understand dependencies**:
   ```
   find_references(symbol: "function_to_test", workspace: "/path")
   search_importers_for(module: "module_name")
   ```

### Phase 2: Plan Test Cases

4. **Identify test scenarios**:
   - Happy path (normal input)
   - Edge cases (boundary values)
   - Error cases (invalid input)
   - Integration points

5. **Check for patterns**:
   ```
   codebase_retrieval(query: "test patterns for similar functionality")
   ```

### Phase 3: Write Tests

6. **Follow project conventions**: Match existing test style

7. **Structure tests**:
   - Arrange: Set up test data
   - Act: Call the function
   - Assert: Verify results

8. **Name tests clearly**: `test_<function>_<scenario>_<expected>`

### Phase 4: Verify

9. **Run tests**: Ensure they pass

10. **Check coverage**: Verify all paths tested

11. **Review for quality**:
    ```
    review_diff(diff: "test changes", focus_areas: ["test_quality"])
    ```

## Test Types

### Unit Tests
- Test single functions in isolation
- Mock dependencies
- Fast execution
- High coverage

### Integration Tests
- Test component interactions
- Use real dependencies
- Slower execution
- Critical paths

### End-to-End Tests
- Test full workflows
- Simulate user actions
- Slowest execution
- Happy paths

## Test Patterns by Language

### Rust
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_happy_path() {
        let result = function(valid_input);
        assert_eq!(result, expected);
    }

    #[test]
    #[should_panic(expected = "error message")]
    fn test_function_error_case() {
        function(invalid_input);
    }
}
```

### TypeScript/JavaScript
```typescript
describe('FunctionName', () => {
  it('should return expected value for valid input', () => {
    expect(functionName(validInput)).toBe(expected);
  });

  it('should throw for invalid input', () => {
    expect(() => functionName(invalidInput)).toThrow('error');
  });
});
```

### Python
```python
def test_function_happy_path():
    result = function(valid_input)
    assert result == expected

def test_function_error_case():
    with pytest.raises(ValueError):
        function(invalid_input)
```

## Best Practices

### Independence
- Tests should not depend on each other
- Each test sets up its own state
- Clean up after tests

### Readability
- Clear test names
- Single assertion per test (when practical)
- Descriptive failure messages

### Maintainability
- DRY test setup with fixtures
- Avoid testing implementation details
- Test behavior, not structure

### Coverage
- 80%+ line coverage goal
- 100% for critical paths
- Test all error handling

## Output Format

When writing tests:

1. **Test File**: Location for new tests
2. **Test Cases**: List of scenarios to test
3. **Test Code**: The actual test implementation
4. **Verification**: How to run and verify tests

