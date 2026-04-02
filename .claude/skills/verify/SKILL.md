---
name: verify
description: Run a verification tier (basic, standard, full) on changed code. Launches the correct agents, handles fix loops, and updates session-state. Use to check code health at any point — not just during the implementation pipeline.
---

# Verify

Run a verification tier on the current branch's changes. Launches the right agents, routes failures through fix loops, and tracks results in session-state.

## Rules

- **ALL agents launch with `run_in_background: true`** — no exceptions
- **Update session-state FIRST** after every agent notification
- **Pass hint blocks verbatim** to fix agents — do not rephrase
- **Never skip tiers** — Basic must pass before Standard runs, Standard before Full
- **Circuit breaking** — 3 failed attempts at the same failure → stop → report STUCK then use the /investigate skill
- **No speculative work** while agents are running — end turn after launching

## When to Use

- Ad-hoc health check ("are things clean?")
- Before committing (use `standard`)
- Before merging (use `full`) — or just use `/finish-dev` which calls this
- After manual edits outside the pipeline
- After pulling changes from develop
- To re-verify after a fix without re-running the full implementation pipeline

## When NOT to Use

- The user hasn't asked for verification and you're not at a verification step in another skill's procedure — don't run `/verify` speculatively
- No files have changed — nothing to verify
- You need to merge — use `/finish-dev` instead (it runs `/verify full` for you)

## Usage

```
/verify              # defaults to basic
/verify basic        # lint + tests
/verify standard     # basic + reviewers (commit gate)
/verify full         # standard + guards + scenarios (pre-merge gate)
```

## Tier Definitions

The authoritative tier definitions are in `.claude/rules/verification-tiers.md`. **ALWAYS** read that file before beginning verification.

## Procedure

### Step 0 — Determine scope

Run `git diff develop --name-only` to identify changed files. This scopes what reviewers and guards examine. If no files have changed, report "nothing to verify" and stop.

### Step 1 — Initialize session-state tracking

If `.claude/state/session-state.md` exists, add/update the **Verification Results** table. If it doesn't exist, create a minimal one:

```markdown
# Session State

## Task
/verify {tier}

## Verification Results
| Agent | Status | Action Needed |
```

### Step 2 — Launch tier agents

Launch ALL agents for the requested tier in parallel (all with `run_in_background: true`). Respect these constraints:

- **Cargo serialization**: runner-linting and runner-tests both use cargo. Launch both — they serialize automatically.
- **Tier inclusion**: Standard includes Basic agents. Full includes Standard agents. Always launch the full set for the requested tier.
- **Reviewers scope**: Tell each reviewer/guard which files changed (from Step 0) so they focus their review.

### Step 3 — Triage results

As each agent completes:

1. **Update session-state FIRST** — before reading results in detail
2. Record status: PASS, FAIL, or findings count
3. For failures, classify using `.claude/rules/routing-failures.md`

Wait for ALL agents in the tier to complete before starting fix routing.

### Step 4 — Fix loop (if failures exist)

Follow the fix routing rules in `.claude/rules/routing-failures.md`:

- **runner-linting**: fmt auto-applied; clippy errors → writer-code with fix spec hints
- **runner-tests**: failing tests → writer-code with fix spec hints; build failures → researcher-rust-errors → writer-code
- **Reviewer findings**: triage per routing-failures.md — inline fixes for style/idiom, writer-code for logic issues
- **Guard findings**: triage per routing-failures.md — inline for warnings, TDD cycle for critical

After each fix cycle, re-run **Basic Verification Tier** first. If Basic passes and the requested tier was higher, re-run the requested tier.

### Step 5 — Report

When all agents pass with no remaining findings:

```
/verify {tier}: CLEAN
- runner-linting: PASS
- runner-tests: PASS
- reviewer-correctness: PASS (if standard+)
- ... (all agents that ran)
```

If stuck (3 failed attempts at same failure — see `.claude/rules/routing-repeated-failures.md`):

```
/verify {tier}: STUCK
- {failure}: 3 attempts exhausted — needs human input
- {details of what was tried}
```

## Fix Loop Sequencing

```
Failure detected
    ↓
Route per routing-failures.md
    ↓
Fix agent (writer-code, inline, etc.)
    ↓
Re-run Basic Verification Tier
    ↓ if Basic clean AND requested tier > basic
Re-run requested tier
    ↓ if new failures
Back to top (max 3 attempts per failure)
```

