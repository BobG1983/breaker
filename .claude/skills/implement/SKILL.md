---
name: implement
description: Start the full TDD implementation pipeline for a feature, refactor, or plan. Creates session-state, identifies waves, drives through specs → RED → GREEN → REFACTOR → commit. Use when beginning any new feature, fix, or refactor that should follow the delegated pipeline.
---

# Implement

Orchestrate the full delegated implementation pipeline for a feature. This skill is the single entry point — it sequences the phases and references the authoritative rules at each step.

## Rules - **IMPORTANT**

- **ALL agents launch with `run_in_background: true`** — no exceptions
- **Update session-state FIRST** after every agent notification — before triaging, before launching the next agent
- **Pass hint blocks verbatim** to fix agents — do not rephrase
- **Never skip phases** — specs before RED, RED before GREEN, GREEN before REFACTOR
- **Never skip the spec review loop** — even for "obvious" specs
- **Circuit breaking** — 3 failed attempts at the same failure → stop → report STUCK then use the /investigate skill
- **No speculative work** while agents are running — end turn after launching
- **Maximize parallelism** — waves are independent; only cargo commands serialize
- **Always** move between plan steps without user input once verification passes clean
- **NEVER** wait for user input to move forwards in a plan if verification passes clean
- **Always** create a TaskList and keep it up to date whenever you update session state

## When to Use

- Starting a new feature from a spec or plan
- Implementing a specific task from the build roadmap
- Any work that should follow the full TDD pipeline
- When you want the pipeline enforced rather than ad-hoc'd

## When NOT to Use

- No plan exists yet — write the plan first in plan mode, then `/implement` it
- Small, obvious, single-domain fix — use `/quickfix` instead
- Not on a topic branch — use `/start-dev` first
- Ready to merge — use `/finish-dev` instead

## Usage

```
/implement <feature description>
/implement --plan <path/to/plan.md>
/implement --plan <path/to/plan.md> --task <task name or number>
/implement todo <number or name>
```

When `--plan` is provided, read the plan file for the feature description, scope, and wave structure. When `--task` is also provided, implement only that specific task from the plan.

When `todo` is provided, read the todo's detail file from `docs/todos/` and use it as the feature description and scope. If the todo is `[NEEDS DETAIL]`, run the `/todo interrogate` procedure for it first. Update the todo status to `[in-progress]` before proceeding.

## Before You Begin

**ALWAYS** read these files before starting — they are the authoritative source for each phase:

- `.claude/rules/delegating-to-subagents.md` — pipeline flow, parallel execution, wave identification
- `.claude/rules/tdd.md` — TDD cycle, RED/GREEN gate procedures, hard rules
- `.claude/rules/spec-workflow.md` — spec creation, review loop, briefing requirements
- `.claude/rules/routing-failures.md` — routing failures to fix agents
- `.claude/rules/routing-repeated-failures.md` — when to stop retrying and escalate
- `.claude/rules/session-state.md` — session-state format and update triggers
- `.claude/rules/sub-agents.md` — every agent, its purpose, when to use it

## Procedure

### Step 0 — Check branch and parse input

1. Verify you're on a topic branch (not develop or main). If on develop, run `/start-dev` to create a branch first.
2. Parse the feature description (inline or from plan file)
3. Create `.claude/state/session-state.md` using the format in the `session-state.md` rule
4. Record the task description and any initial decisions

### Step 1 — Analyze scope and identify waves

Determine:
- **Domains involved**: which plugins/domains this feature touches
- **Parallel waves**: group independent work into waves per the criteria in `delegating-to-subagents.md`
- **Shared prerequisites**: cross-domain types, queries, messages that multiple waves need
- **Research triggers**: does this need a research wave? (2+ domains, unfamiliar APIs, new messages/state)

Record wave structure and decisions in session-state.

### Step 2 — Research gate (conditional)

**Skip if**: single-domain feature with familiar APIs and no new cross-domain communication.

**Run if** any trigger from `delegating-to-subagents.md` Research Wave applies. Launch applicable research agents from `sub-agents.md` Research Agents table in parallel, if no applicable research agent exists use Explore or a General sub-agent. Feed results into Step 3 and Step 4 briefings. Write research output to `.claude/research/<plan>-<optional:wave>-<feature>-<research_area>.md`

### Step 3 — Shared prerequisites

If waves share types, messages, or query aliases — create them now before specs begin. This may be a direct edit (simple type definitions) or a mini spec → writer cycle for anything non-trivial.

### Step 4 — Spec phase (per wave, parallel)

For each wave, launch in parallel:
- **planning-writer-specs-tests** — writes test spec to `.claude/specs/<wave>-<feature>-tests.md`
- **planning-writer-specs-code** — writes impl spec to `.claude/specs/<wave>-<feature>-code.md`

Brief each spec writer with ALL required context per `spec-workflow.md` Briefing Spec Writers section. Include research results from Step 2 if applicable.

Update session-state as each completes.

### Step 5 — Spec review loop

As each spec completes, launch its reviewer in parallel:
- **planning-reviewer-specs-tests** — pressure-tests the test spec
- **planning-reviewer-specs-code** — pressure-tests the impl spec

Triage findings per `spec-workflow.md` Spec Revision Loop:
1. Dismiss false positives
2. Route valid feedback back to the spec writer
3. Re-launch reviewer if BLOCKING or IMPORTANT findings were revised
4. **Do NOT proceed to Step 6 until BOTH specs are confirmed clean**

Update session-state after each agent notification.

### Step 6 — RED phase

For each wave:
1. Launch **writer-tests** (reads spec from `.claude/specs/<wave>-<feature>-tests.md`)
2. As each completes, launch **reviewer-tests** immediately
3. After ALL reviewer-tests pass, launch a single **runner-tests** (RED gate)

**RED gate rules** (from `tdd.md`):
- Tests MUST compile
- Tests MUST fail (if any pass → investigate before proceeding)
- If tests don't compile → route back to writer-tests with compiler error

Update session-state RED Gate column after the gate.

### Step 7 — GREEN phase

After RED gate passes:
1. Launch ALL **writer-codes** in parallel (one per wave, reads spec from `.claude/specs/<wave>-<feature>-code.md`)
2. After ALL complete, launch a single **runner-tests** (GREEN gate)

**GREEN gate rules** (from `tdd.md`):
- ALL tests must pass
- If any fail → route to writer-code with fix spec hints per `routing-failures.md`

Update session-state GREEN column after the gate.

### Step 8 — REFACTOR phase

```
/verify basic
    ↓ fix failures → /verify basic again
/simplify on changed code
    ↓ if changes → /verify basic again
Wiring (lib.rs, game.rs, shared.rs if needed)
    ↓ if changes → /verify basic again
```

Repeat until `/verify basic` is clean and `/simplify` finds nothing.

### Step 9 — Commit gate

Run `/verify standard`.

Fix any failures per `routing-failures.md`, re-running `/verify basic` after each fix, then `/verify standard` again.

### Step 10 — Commit

When `/verify standard` is clean:
1. Stage changed files
2. Commit using the format in `commit-format.md`
3. Update session-state to record completion


### Step 11 - Repeat 

1. Repeat Steps 1-10 until the **entire** plan is complete
2. Run /finish-dev to run final verifications and merge