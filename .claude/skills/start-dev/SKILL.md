---
name: start-dev
description: Start a new development branch from develop using git-flow. Use when beginning any new feature, bug fix, or refactor. Guards against accidentally working on develop. Drives the full lifecycle from context gathering → planning → implementation.
---

# Start Dev

Three things happen in order: **branch**, **plan**, **implement**. No step is ever skipped.

## Rules

- **NEVER** use raw `git checkout -b` or `git branch` — always `git flow <type> start`
- **ALWAYS** read `.claude/rules/git.md` for the full git-flow workflow
- **NEVER** launch `/implement` without an approved plan
- **NEVER** treat a todo detail file as a plan — it is input context for building a plan

## Usage

```
/start-dev feature <name>
/start-dev fix <name>
/start-dev refactor <name>
/start-dev feature <name> todo <number or name>
```

## Procedure

### Step 1 — Branch

Get onto a topic branch. Determine the current branch via `git branch --show-current`.

**On `develop`:**
```bash
git pull origin develop
git flow <type> start <name>
```
Report the new branch name. Proceed to Step 2.

**On `main`:** Warn "You're on main. Switch to develop first." Stop.

**On any other branch:** Warn "Already on branch `<name>`. Do you want to continue on this branch?" If yes, proceed to Step 2. If no, stop.

**Type mapping:**
| Argument | Command |
|----------|---------|
| `feature` | `git flow feature start <name>` |
| `fix` | `git flow bugfix start <name>` |
| `refactor` | `git flow refactor start <name>` |

### Step 2 — Plan

Every `/start-dev` produces a plan. The input context varies (todo detail file vs inline description) but the planning process is always the same.

**If a `todo` was provided**, prepare the input context first:
1. Read the todo item and its detail file from `docs/todos/`.
2. If the todo is `[NEEDS DETAIL]`, run `/todo interrogate` — ask questions recursively until all open questions are resolved or the user says stop. Update the detail file.
3. Mark the todo as `[in-progress]`.

Now, regardless of whether a todo was provided or not:

1. **Call `EnterPlanMode`.** This switches into plan mode where you can read files and explore the codebase but cannot write code.

2. **Build the plan.** Read the input context (todo detail file, or the user's inline description). Explore the codebase as needed to understand existing structure, patterns, and constraints. Write an implementation plan to the plan file. The plan must include:
   - **Scope** — what is in and out
   - **Domains** — which plugins/modules are touched
   - **Waves** — independent groups of work that can run in parallel
   - **Per-wave detail** — what types, systems, components, or tests each wave produces
   - **Shared prerequisites** — cross-domain types or wiring needed before waves begin
   - **Open questions** — anything that needs the user's input before starting

3. **Call `ExitPlanMode`.** The user sees the plan and either approves, requests changes, or rejects.
   - Approved → proceed to Step 3.
   - Changes requested → re-enter plan mode, revise, exit again.
   - Rejected → stop.

4. **Update the todo** (if one was provided) with any decisions or scope changes from the planning discussion.

### Step 3 — Implement

Launch `/implement` with the approved plan.
