---
name: finish-dev
description: Run the Full Verification Tier and finish the current git-flow branch. Use when done with a feature, fix, or refactor and ready to merge to develop.
---

# Finish Dev

Run the pre-merge gate (Full Verification Tier) and finish the current git-flow branch, merging it to develop.

## Rules

- **NEVER** use raw `git merge` — always `git flow <type> finish`
- **NEVER** skip the Full Verification Tier — it's the pre-merge gate
- **ALWAYS** read `.claude/rules/git.md` for the full git-flow workflow
- **ALWAYS** read `.claude/rules/verification-tiers.md` for what the Full tier includes

## When to Use

- Done with a feature, fix, or refactor and ready to merge to develop
- All commits are made and you want the pre-merge gate enforced

## When NOT to Use

- Still implementing — finish the work with `/implement` or `/quickfix` first
- On develop or main — nothing to finish
- Want to verify without merging — use `/verify full` standalone

## Usage

```
/finish-dev
```

No arguments — detects the current branch type automatically.

## Procedure

### Step 1 — Detect branch type

Determine the current branch via `git branch --show-current`.

- If on `feature/*`: type = `feature`
- If on `fix/*`: type = `bugfix`
- If on `refactor/*`: type = `refactor`
- If on `develop`, `main`, `release/*`, or `hotfix/*`: warn and stop — this skill is for topic branches only

### Step 2 — Check for uncommitted work

Run `git status`. If there are uncommitted changes, warn: "Uncommitted changes detected. Commit or stash before finishing?": If the user responds with "commit" or "stash" do that, then proceed to Step 3.

### Step 3 — Update from develop

```bash
git flow update
```

Pull latest develop into the current branch to prevent merge conflicts at finish time. If this fails due to stale state:

```bash
rm .git/gitflow/state/merge.json
git flow update
```

### Step 4 — Full Verification Tier

Run `/verify full`.

This is the pre-merge gate. ALL findings must be resolved — see `.claude/rules/verification-tiers.md` for what "clean" means at the Full tier.

### Step 5 — Final check

After `/verify full` is clean:
1. Run `git status` — verify clean working tree (background agents may have left dirty files)
2. If dirty: stage and commit the changes, then re-run `/verify basic` to confirm

### Step 6 — Finish the branch

```bash
git flow <type> finish
```

This merges to develop and deletes the topic branch.

### Step 7 — Cleanup ephemeral artifacts

After a successful merge, remove all ephemeral artifacts from this branch's work:

```bash
rm -rf .claude/specs/*
rm -rf .claude/state/*
rm -rf .claude/research/*
rm -rf .claude/fixes/*
rm -rf .claude/agent-memory/*/ephemeral/*
```

These are all gitignored and only useful during active development. Specs, session-state, research output, debug reports, and ephemeral agent memory are dead once the branch is merged. Stable agent memory (the root of each agent's memory dir) is preserved.

### Step 8 — Confirm

Report: branch merged to develop, topic branch deleted, ephemeral artifacts cleaned up.
