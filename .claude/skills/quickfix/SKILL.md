---
name: quickfix
description: Abbreviated TDD pipeline for small, obvious fixes. Skips the spec phase — the description IS the spec. Still follows TDD (failing test → implementation → verify). Use instead of /implement when the fix is small (ONLY: a single function, a single test, a single file, or a rename).
---

# Quickfix

An abbreviated TDD pipeline for small fixes that don't need the full spec phase. The description you provide IS the behavioral spec — it goes straight to writer-tests.

## Rules

- **ALL agents launch with `run_in_background: true`** — no exceptions
- **Update session-state FIRST** after every agent notification
- **Still TDD** — never skip the RED gate, even for "obvious" fixes
- **Circuit breaking** — 3 failed attempts → `/investigate`. See `.claude/rules/routing-repeated-failures.md`
- **No speculative work** while agents are running — end turn after launching
- **Escalation** — if at any point the fix turns out to be more complex than expected, OR the user requests broader changes mid-quickfix: stop immediately, enter `/plan` mode, create a plan, then switch to `/implement`. Do not continue the quickfix pipeline with expanded scope.

## Usage

```
/quickfix <description of the fix>
```

Example:
```
/quickfix "BoltVelocity clamp doesn't handle zero velocity — should leave it unchanged"
```

## When to Use vs `/implement`

| | `/quickfix` | `/implement` |
|---|---|---|
| Scope | Single domain, 1-2 files | Any size, multi-domain |
| Input | Inline description | Plan file |
| Spec phase | Skipped — description = spec | Full spec writers + reviewers |
| Waves | Always one | Multiple possible |
| Research | Skipped | Conditional |
| TDD | Yes (RED → GREEN → REFACTOR) | Yes |
| Verification | basic → simplify → standard | basic → simplify → standard |

**Use `/quickfix` when:** the fix is obvious, single-domain, and you can describe the expected behavior in a sentence or two.

**Use `/implement` when:** the work touches multiple domains, needs waves, or the behavior is complex enough to benefit from spec review.

## When NOT to Use

- Multi-domain or multi-file change — use `/implement`
- Not sure what's wrong — use `/investigate` first to find the root cause
- Not on a topic branch — use `/start-dev` first
- The description takes more than two sentences — it's too complex, use `/implement`

## Before You Begin

**ALWAYS** read these files:
- `.claude/rules/tdd.md` — TDD cycle, RED/GREEN gate procedures
- `.claude/rules/routing-failures.md` — routing failures to fix agents
- `.claude/rules/routing-repeated-failures.md` — when to stop retrying and escalate

## Procedure

### Step 1 — Validate scope

Confirm this is appropriate for `/quickfix`:
- Single domain? If multi-domain → suggest `/implement` instead
- Obvious fix? If complex or uncertain → suggest `/implement` instead
- On a topic branch? If on develop/main → suggest `/start-dev` first

### Step 2 — Create minimal session-state

Create `.claude/state/session-state.md`:

```markdown
# Session State

## Task
/quickfix: <description>

## Specs
| Domain | Writer-Tests | Test Review | RED Gate | Writer-Code | GREEN | Notes |
```

### Step 3 — RED phase

1. Launch **writer-tests** with the description as the behavioral spec
   - Include: concrete values, expected behavior, edge cases from the description
   - Point to reference files in the relevant domain
2. Launch **reviewer-tests** when writer-tests completes
3. Launch **runner-tests** (RED gate) when reviewer-tests passes

**RED gate rules** (from `tdd.md`):
- Tests MUST compile
- Tests MUST fail
- If tests don't compile → route back to writer-tests with compiler error

### Step 4 — GREEN phase

1. Launch **writer-code** with the description + failing test file as context
2. Launch **runner-tests** (GREEN gate) when writer-code completes

**GREEN gate rules** (from `tdd.md`):
- ALL tests must pass
- If any fail → route to writer-code with fix spec hints

### Step 5 — REFACTOR phase

```
/verify basic
    ↓ fix failures → /verify basic again
/simplify on changed code
    ↓ if changes → /verify basic again
```

### Step 6 — Commit gate

Run `/verify standard`.

Fix any failures, re-running `/verify basic` after each fix, then `/verify standard` again.

### Step 7 — Commit

When `/verify standard` is clean:
1. Stage changed files
2. Commit using the format in `commit-format.md`
3. Update session-state to record completion
