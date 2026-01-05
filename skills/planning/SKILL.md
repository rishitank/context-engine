---
name: planning
description: Task planning and execution workflow for complex multi-step tasks. Helps break down work into manageable steps, track progress, and ensure completion.
category: workflow
tags:
  - planning
  - task-management
  - workflow
  - execution
always_apply: false
---

# Planning Skill

Use this skill when you need to plan and execute complex multi-step tasks.

## When to Use

- Breaking down a large feature into implementation steps
- Managing a multi-file refactoring
- Tracking progress on a complex bug fix
- Coordinating changes across multiple components

## Workflow

### 1. Create a Plan

Start by creating a plan with a clear title and description:

```
create_plan(
  title: "Implement user authentication",
  description: "Add JWT-based authentication with login, logout, and session management"
)
```

### 2. Add Steps

Break down the work into discrete steps:

```
add_step(plan_id: "...", title: "Create User model", type: "implementation")
add_step(plan_id: "...", title: "Add JWT middleware", type: "implementation")
add_step(plan_id: "...", title: "Write authentication tests", type: "testing")
add_step(plan_id: "...", title: "Update API documentation", type: "documentation")
```

Step types: `research`, `implementation`, `testing`, `documentation`, `review`

### 3. Execute Steps

Work through steps one at a time:

```
start_step(plan_id: "...", step_id: "...")
# ... do the work ...
complete_step(plan_id: "...", step_id: "...", notes: "Created User model with email/password fields")
```

If a step fails:
```
fail_step(plan_id: "...", step_id: "...", reason: "Dependency conflict with existing auth library")
```

### 4. Track Progress

Check plan status at any time:

```
get_plan(plan_id: "...")
list_plans()
get_plan_progress(plan_id: "...")
```

### 5. Adapt as Needed

Plans can evolve:

```
update_step(plan_id: "...", step_id: "...", title: "Updated title", description: "More details")
reorder_steps(plan_id: "...", step_ids: ["step3", "step1", "step2"])
add_dependency(plan_id: "...", step_id: "...", depends_on: "other_step_id")
```

## Available Tools

| Tool | Purpose |
|------|---------|
| `create_plan` | Create a new plan |
| `get_plan` | Get plan details |
| `list_plans` | List all plans |
| `update_plan` | Update plan title/description |
| `delete_plan` | Delete a plan |
| `add_step` | Add a step to a plan |
| `update_step` | Update step details |
| `delete_step` | Remove a step |
| `start_step` | Mark step as in-progress |
| `complete_step` | Mark step as complete |
| `fail_step` | Mark step as failed |
| `skip_step` | Skip a step |
| `get_step` | Get step details |
| `list_steps` | List steps in a plan |
| `reorder_steps` | Change step order |
| `add_dependency` | Add step dependency |
| `remove_dependency` | Remove step dependency |
| `get_plan_progress` | Get completion percentage |
| `export_plan` | Export plan as markdown |
| `import_plan` | Import plan from markdown |

## Best Practices

1. **Start with research**: First step should gather information
2. **Keep steps atomic**: Each step should be completable in one session
3. **Add notes**: Document decisions and findings in step notes
4. **Track blockers**: Use `fail_step` with clear reasons
5. **Review progress**: Check `get_plan_progress` regularly

## Example: Feature Implementation

```
# 1. Create the plan
plan = create_plan(
  title: "Add dark mode support",
  description: "Implement system-wide dark mode with user preference persistence"
)

# 2. Add research step
add_step(plan_id: plan.id, title: "Research existing theme system", type: "research")

# 3. Add implementation steps
add_step(plan_id: plan.id, title: "Create theme context provider", type: "implementation")
add_step(plan_id: plan.id, title: "Add CSS variables for colors", type: "implementation")
add_step(plan_id: plan.id, title: "Implement theme toggle component", type: "implementation")
add_step(plan_id: plan.id, title: "Persist preference to localStorage", type: "implementation")

# 4. Add testing
add_step(plan_id: plan.id, title: "Write theme switching tests", type: "testing")

# 5. Add documentation
add_step(plan_id: plan.id, title: "Update component documentation", type: "documentation")

# 6. Execute each step...
```

