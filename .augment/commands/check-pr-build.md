# Command: Check PR Build Status

**Automatically checks CI/CD build status for a pull request and retrieves detailed logs for any failures.**

## Description

This command comprehensively investigates CI build status for a PR by:

1. Determining the PR number (from parameter or current branch)
2. Getting the latest commit SHA for the PR
3. Fetching all check runs and workflow runs for the commit
4. Identifying failed jobs and retrieving their logs
5. Parsing logs to extract error messages and failure details
6. Providing a structured report with actionable information

This command follows GitHub API best practices from AGENTS.md for complete CI status retrieval.

## Usage

`@.augment/commands/check-pr-build.md [PR_NUMBER]`

### Parameters

- `PR_NUMBER` (optional): The pull request number to check. If not provided, uses the current branch's PR.

### Examples

```bash
# Check build status for PR #100
@.augment/commands/check-pr-build.md 100

# Check build status for current branch's PR
@.augment/commands/check-pr-build.md
```

---

## Task

When this command is executed, perform the following steps:

## 1. Determine PR Number and Commit SHA

- If `PR_NUMBER` is provided, use it
- Otherwise, get current branch name and search GitHub for open PRs from that branch
  - Use: `GET /repos/{owner}/{repo}/pulls?head={owner}:{branch_name}&state=open`
- Get the latest commit SHA:
  - From local git: `git rev-parse HEAD`
  - Or from PR details: `head.sha`

## 2. Fetch Check Runs

Use: `GET /repos/{owner}/{repo}/commits/{commit_sha}/check-runs`

- Set `per_page: 100` and handle pagination if needed
- For each check run, extract:
  - `id`: Check run ID
  - `name`: Check run name (e.g., "build-test")
  - `status`: `queued`, `in_progress`, `completed`
  - `conclusion`: `success`, `failure`, `cancelled`, `skipped`, etc.
  - `html_url`: Link to the check run on GitHub

## 3. Fetch Workflow Runs

Use: `GET /repos/{owner}/{repo}/actions/runs?head_sha={commit_sha}`

- Set `per_page: 100`
- For each workflow run, extract:
  - `id`: Workflow run ID
  - `name`: Workflow name (e.g., "CI")
  - `status`: `queued`, `in_progress`, `completed`
  - `conclusion`: `success`, `failure`, `cancelled`, etc.
  - `html_url`: Link to the workflow run on GitHub
  - `run_number`: Run number for reference

## 4. Get Jobs for Failed Workflow Runs

For each failed workflow run, use: `GET /repos/{owner}/{repo}/actions/runs/{run_id}/jobs`

- Set `per_page: 100`
- For each job, extract:
  - `id`: Job ID
  - `name`: Job name (e.g., "build-test")
  - `status`: Job status
  - `conclusion`: Job conclusion
  - `steps`: Array of steps with their status and conclusion
  - Identify which step(s) failed

## 5. Retrieve Logs for Failed Jobs

For each failed job, use: `GET /repos/{owner}/{repo}/actions/jobs/{job_id}/logs`

- Logs are returned as plain text
- Parse logs to extract:
  - Error messages (lines containing "Error:", "FAIL", "✕", "✖")
  - Test failures (snapshot mismatches, assertion failures)
  - Build errors (TypeScript errors, linting issues)
  - Stack traces
  - File paths and line numbers

## 6. Parse and Categorize Errors

Categorize errors by type:

- **Test Failures**: Snapshot mismatches, failed assertions, test timeouts
- **Build Errors**: TypeScript compilation errors, module resolution failures
- **Linting Issues**: ESLint/Prettier violations
- **Dependency Issues**: Missing packages, version conflicts
- **Other**: Uncategorized errors

## 7. Generate Report

Output format:

```markdown
## CI Build Status for PR #{PR_NUMBER}

**PR**: #{number} - {title}
**Branch**: {head_ref}
**Commit**: {commit_sha}

### Summary
- Total Check Runs: {count}
- Total Workflow Runs: {count}
- Failed Jobs: {count}

### Check Runs Status
- ✅ {check_run_name}: {conclusion}
- ❌ {check_run_name}: {conclusion}

### Failed Jobs

#### Job: {job_name}
**Workflow**: {workflow_name}
**Run**: #{run_number}
**Failed Step**: {step_name}

[View Job]({job_url})
[View Logs]({logs_url})

**Error Summary**:

```text
{extracted_error_messages}
```

**Suggested Fix**:

{analysis_and_suggestions}
```

---

### Next Steps

1. {actionable_step_1}
2. {actionable_step_2}

## Error Analysis Algorithm

When generating `{analysis_and_suggestions}`, apply the following logic:

| Error Category | Pattern | Suggested Fix |
|----------------|---------|---------------|
| **Test Failures** | `FAIL`, `✕`, `assertion failed`, `expected X but got Y` | Re-run failing tests locally, check for flaky tests, review test assertions |
| **Build Errors** | `error[E`, `cannot find`, `unresolved import` | Check for missing dependencies, verify import paths, run `cargo check` locally |
| **TypeScript Errors** | `TS\d+:`, `Type .* is not assignable` | Fix type annotations, check for missing type definitions |
| **Linting Issues** | `warning:`, `clippy::`, `eslint` | Run formatter (`cargo fmt`, `npm run lint:fix`), address warnings |
| **Dependency Issues** | `could not resolve`, `version conflict`, `not found in registry` | Update lockfile, check version constraints, verify package exists |
| **Timeout/Hang** | `timed out`, `exceeded`, `killed` | Increase timeout, check for infinite loops, optimize slow operations |

For each error, provide:
1. **Root cause**: What specifically failed
2. **File/line**: Where the error occurred (if available)
3. **Fix command**: Specific command to run (e.g., `cargo fmt`, `npm test -- --updateSnapshot`)
4. **Prevention**: How to avoid this in the future

## Notes

- Follow GitHub API best practices from AGENTS.md
- Use pagination for all list endpoints
- Parse logs efficiently - focus on error patterns
- Provide actionable suggestions based on error types
- Include direct links to GitHub UI for easy navigation

