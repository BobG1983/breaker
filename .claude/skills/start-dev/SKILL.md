---
name: start-dev
description: Start a new development branch from develop using git-flow. Use when beginning any new feature, bug fix, or refactor. Guards against accidentally working on develop. Optionally takes a --todo to drive the full lifecycle from todo → plan → implement.
---

# Start Dev

Start a new git-flow topic branch from develop. Guards against working directly on develop or starting a branch when you're already on one. When given a `--todo`, drives the full lifecycle: interrogate for missing detail → plan → `/implement`.

## Rules

- **NEVER** use raw `git checkout -b` or `git branch` — always `git flow <type> start`
- **ALWAYS**  read `.claude/rules/git.md` for the full git-flow workflow

## When to Use

- Beginning any new feature, bug fix, or refactor
- User says "let's start working on X" and you're on develop
- User wants to start working on a specific todo item

## When NOT to Use

- Already on a topic branch and don't need a new one — just work, or ask the user
- Need to finish current work first — use `/finish-dev`
- On main — switch to develop first

## Usage

```
/start-dev feature <name>
/start-dev fix <name>
/start-dev refactor <name>
/start-dev feature <name> --todo <number or name>
```

## Procedure

### Step 1 — Check current branch

Determine the current branch via `git branch --show-current`.

- If on `develop`: proceed to Step 2
- If on `main`: warn "You're on main. Switch to develop first." — stop
- If on any other branch: warn "Already on branch `<name>`. Do you want to continue on this branch?": If the user says yes: proceed to Step 4

### Step 2 — Pull latest

```bash
git pull origin develop
```

Ensure develop is up to date before branching.

### Step 3 — Start the branch

```bash
git flow <type> start <name>
```

Where `<type>` is `feature`, `bugfix`, or `refactor` based on the argument.

**Type mapping:**
- `feature` → `git flow feature start <name>`
- `fix` → `git flow bugfix start <name>`
- `refactor` → `git flow refactor start <name>`

### Step 4 — Confirm branch

Report the new branch name.

### Step 5 — Todo lifecycle (if `--todo` provided)

If `--todo` was provided:

1. Read the todo's detail file from `.claude/todos/`
2. If the todo is `[NEEDS DETAIL]`, run the `/todo interrogate` procedure for it — ask questions recursively until all open questions are resolved or the user says stop
3. Update the todo detail file with anything captured
4. Enter `/plan` mode with the todo's context as input
5. After the plan is approved by the user, update the todo detail file with any new decisions or scope changes from the planning discussion
6. Launch `/implement` with the plan

If `--todo` was NOT provided: done — report the branch is ready for work.
