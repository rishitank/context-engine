# Command: Check PR Review Comments

**Automatically fetches and analyzes all unresolved code review comments for a pull request.**

## Description

This command comprehensively retrieves all code review feedback for a PR by:

1. Fetching the PR details from GitHub
2. Getting all reviews submitted on the PR
3. Fetching all review comments (line-level comments)
4. Fetching all issue comments (general PR comments)
5. Analyzing which comments are unresolved and still relevant
6. Providing a structured report of actionable feedback

This command follows GitHub API best practices from AGENTS.md for complete comment retrieval.

## Usage

`@.augment/commands/check-pr-review-comments.md [PR_NUMBER]`

### Parameters

- `PR_NUMBER` (optional): The pull request number to check. If not provided, uses the current branch's PR.

### Examples

```bash
# Check review comments for PR #6291
@.augment/commands/check-pr-review-comments.md 6291

# Check review comments for current branch's PR
@.augment/commands/check-pr-review-comments.md
```

---

## Task

When this command is executed, perform the following steps:

## 1. Determine PR Number

- If `PR_NUMBER` is provided, use it
- Otherwise, get current branch name and search GitHub for open PRs from that branch
- Use: `GET /repos/rishitank/context-engine/pulls?head=rishitank:{branch_name}&state=open`

## 2. Fetch PR Details

- Use: `GET /repos/rishitank/context-engine/pulls/{PR_NUMBER}`
- Extract:
  - `head.sha`: Latest commit SHA
  - `head.ref`: Branch name
  - `title`: PR title
  - `state`: PR state
  - `created_at`: When PR was created

## 3. Fetch All Reviews

- Use: `GET /repos/rishitank/context-engine/pulls/{PR_NUMBER}/reviews`
- Set `per_page: 100` and handle pagination via Link headers
- For each review, extract:
  - `id`: Review ID
  - `user.login`: Reviewer username
  - `state`: APPROVED, CHANGES_REQUESTED, COMMENTED, DISMISSED
  - `body`: Review-level comment
  - `submitted_at`: When review was submitted

## 4. Fetch All Review Comments (Line-Level)

- Use: `GET /repos/rishitank/context-engine/pulls/{PR_NUMBER}/comments`
- Set `per_page: 100`, `sort: created`, `direction: desc`
- Handle pagination - ALWAYS fetch all pages using Link headers
- For each comment, extract:
  - `id`: Comment ID
  - `user.login`: Commenter username
  - `body`: Comment text
  - `path`: File path
  - `line` or `original_line`: Line number
  - `created_at`: When comment was created
  - `updated_at`: When comment was last updated
  - `in_reply_to_id`: Parent comment ID (for threads)
  - `pull_request_review_id`: Associated review ID

## 5. Fetch All Issue Comments (General PR Comments)

- Use: `GET /repos/rishitank/context-engine/issues/{PR_NUMBER}/comments`
- Set `per_page: 100`, `sort: created`, `direction: desc`
- Handle pagination using Link headers
- For each comment, extract:
  - `id`: Comment ID
  - `user.login`: Commenter username
  - `body`: Comment text
  - `created_at`: When comment was created
  - `updated_at`: When comment was last updated

## 6. Analyze Comment Status

For each comment, determine if it's:

- **Resolved**: Look for indicators in comment body like "✅ Addressed", "Fixed in commit", etc.
- **Acknowledged**: User has responded but not necessarily fixed
- **Unresolved**: No response or fix
- **Still Relevant**: Check if the file/line still exists in latest commit

Filter out:

- Bot comments (username contains `[bot]`)
- Comments marked as resolved
- Comments on code that no longer exists

## 7. Categorize Comments

Group unresolved comments by:

- **Critical**: From CHANGES_REQUESTED reviews, blocking issues
- **Major**: Significant refactoring suggestions, potential bugs
- **Minor**: Code style, nitpicks, suggestions
- **Questions**: Requests for clarification

## 8. Generate Report

Output format:

```markdown
## Code Review Comments for PR #{PR_NUMBER}

**PR**: #{number} - {title}
**Branch**: {head_ref}
**Status**: {state}

### Summary
- Total Reviews: {count}
- Total Comments: {count}
- Unresolved Comments: {count}
  - Critical: {count}
  - Major: {count}
  - Minor: {count}
  - Questions: {count}

### Unresolved Comments

#### Critical Issues ({count})

**{file_path}:{line}** - by @{reviewer}
> {comment_body}

[View Comment]({comment_url})

---

#### Major Issues ({count})

**{file_path}:{line}** - by @{reviewer}
> {comment_body}

[View Comment]({comment_url})

---

#### Minor Issues ({count})

**{file_path}:{line}** - by @{reviewer}
> {comment_body}

[View Comment]({comment_url})

---

#### Questions ({count})

**{file_path}:{line}** - by @{reviewer}
> {comment_body}

[View Comment]({comment_url})

---

### Review Status by Reviewer

- @{reviewer1}: {APPROVED|CHANGES_REQUESTED|COMMENTED} on {date}
- @{reviewer2}: {APPROVED|CHANGES_REQUESTED|COMMENTED} on {date}

```

## Notes

- Follow GitHub API best practices from AGENTS.md
- ALWAYS use pagination with Link headers - don't assume all comments fit in one page
- Sort comments by `created` date descending to see newest first
- Check for new comments that may have been added after initial fetch
- Filter out bot comments (coderabbitai[bot], linear[bot], etc.)
- Look for resolution markers in comment bodies
- Provide direct links to each comment for easy navigation
