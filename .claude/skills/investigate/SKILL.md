---
name: investigate
description: Methodical root-cause analysis for bugs, failing tests, failing scenarios, or unexpected behavior. The orchestrator does NOT run the DEBUG protocol itself — it spawns the `debugger` subagent (Opus) for hypothesis work, runs targeted tests / researcher agents on the debugger's recommendation, feeds results back, and routes the final regression spec hint through the standard fix pipeline.
---

# Investigate

A structured workflow for debugging that puts heavy reasoning on `debugger` (Opus) while the orchestrator (Sonnet) coordinates spawns and routes the eventual fix.

## Rules

- **The orchestrator does NOT run the DEBUG protocol itself.** Hypothesis generation, ranking, Five Whys, and root-cause confirmation are all `debugger`'s job.
- **All agents launch with `run_in_background: true`** — no exceptions
- **Update `.claude/state/session-state.md` and `.claude/state/investigation-state.md`** after every agent notification
- **Pass hint blocks verbatim** to fix agents — never rephrase
- **Circuit-break on the debugger too**: if 3 consecutive debugger spawns fail to converge on a root cause, stop the loop and surface to the user

## Purpose

1. Gather evidence before changing code
2. Identify root cause, not just symptoms
3. Prevent introducing new bugs while fixing
4. Document findings for future investigations (debugger memory)

## When to Use

- Bug reports with unclear cause
- Errors that don't make sense
- Issues that have "already been fixed" before
- Problems spanning multiple components
- Failing unit tests
- Failing scenarios
- Escalation from `/verify`, `/implement`, or `/quickfix` after circuit-breaking (3 failed fix attempts on the same failure — see `.claude/rules/routing-repeated-failures.md`)

## When NOT to Use

- You're planning new work, not debugging existing behavior — use `/start-dev` then `/implement`
- The fix is obvious and you can describe it in a sentence — use `/quickfix`
- A reviewer surfaced a finding with high-confidence routing in `.claude/rules/routing-failures.md` — route it directly, no investigation needed

## Procedure

### Step 1 — Capture failure context

Gather what you have:

- Failure type: failing test, failing scenario, build error, runtime panic, unexpected behavior
- Exact error message / test output / scenario violation
- File path and line of the failure if known
- Recent changes (last 3–5 commits on the touched files): `git log -5 --oneline path/to/file.rs`
- Prior fix attempts (if any) — names of agents spawned, what they tried, what failed

Do NOT start hypothesizing yourself. You are gathering raw evidence for the debugger.

### Step 2 — Initialize investigation state

Create or update `.claude/state/investigation-state.md` with the captured context (the debugger reads/updates this file across iterations).

If `session-state.md` exists, add the investigation to its **Active Investigations** section per `.claude/rules/session-state.md`. If session-state doesn't exist, create a minimal one with just the investigation entry.

### Step 3 — Launch the debugger

Brief the `debugger` subagent with:
- The captured failure context from Step 1
- Path to `.claude/state/investigation-state.md`
- Path(s) to relevant source / test files
- Any prior fix attempts and why they failed (if escalated from a circuit-break)

**Do NOT brief with your own hypotheses.** The debugger generates hypotheses; you are not its peer reviewer.

### Step 4 — Iteration loop

The debugger returns a **Next Action** in its summary. Execute it, then re-launch the debugger with the result. Possible next actions:

| Debugger says | Orchestrator does |
|---------------|-------------------|
| "Spawn `runner-cargo` against test path X" | Spawn `runner-cargo` for that specific test, capture pass/fail + output |
| "Spawn `researcher-codebase` to trace data flow Z" | Spawn the researcher with the specific question, capture findings |
| "Spawn `researcher-impact` for type T" | Spawn the researcher, capture reference list |
| "Spawn `researcher-git` for file F" | Spawn the researcher, capture history |
| "Spawn `researcher-bevy-api` for API X" | Spawn the researcher, capture verification |
| "Spawn `researcher-rust` for error E or idiom Q" | Spawn the researcher, capture diagnosis |
| "Re-launch debugger with [evidence]" | Spawn `debugger` again, briefing with the new evidence |
| "Root cause confirmed; regression spec hint at investigation-state.md" | Exit loop → Step 5 |

After each spawn:
1. **Update session-state and investigation-state FIRST** (per `.claude/rules/session-state.md`)
2. Re-launch the debugger with: prior state file path + the new evidence (test output, researcher report)
3. Track loop iteration count

**Circuit-breaker on the debugger**: if 3 consecutive debugger spawns fail to advance the investigation (no hypothesis disproven, no new lead, no convergence), stop the loop and report STUCK to the user with the full investigation-state for human review.

### Step 5 — Route the regression spec hint

When the debugger returns a confirmed root cause and a filled regression spec hint:

1. Read the hint from `investigation-state.md`
2. Route per `.claude/rules/routing-failures.md`:
   - **Code-level bug, high confidence** → `writer-tests` (with the regression spec hint as briefing) → RED gate → `writer-code` → GREEN gate
   - **Code-level bug, low confidence** → re-launch `debugger` with a request for stronger evidence
   - **Scenario-level bug** → `writer-scenarios` to add a regression scenario, then `writer-code` if a code fix is also needed
   - **Spec-level bug** (the test was wrong) → write a test revision spec, route to `writer-tests`

3. Run **Basic Verification Tier** (`/verify basic`) after the fix lands

### Step 6 — Resolve

Once `/verify basic` is clean:

1. Update `investigation-state.md` status to `resolved` with the fix commit / file:line
2. Move the entry from `Active Investigations` to `Resolved` in `session-state.md`
3. Continue the parent flow that escalated to `/investigate` (if any)

### Step 7 — Stuck

If the circuit-breaker fires (3 unproductive debugger spawns), or if the debugger returns "no plausible hypotheses remain":

1. Update `investigation-state.md` status to `stuck`
2. Move the entry to `Stuck` in `session-state.md`
3. Surface to the user with: full investigation-state, all hypotheses tested, all evidence gathered, what's blocking convergence

## Anti-Patterns to Avoid

| Anti-Pattern | Why bad | Instead |
|--------------|---------|---------|
| Orchestrator hypothesizing | Sonnet doing Opus-quality reasoning | Spawn `debugger` (Opus) for hypotheses |
| Skipping evidence | Unfounded fix → bug returns | Let debugger drive evidence-gathering |
| Fixing while investigating | Mixes phases, hides the root cause | Investigate first, then route the hint |
| Ad-hoc grep across the codebase | Wastes tokens, rarely finds the issue | Spawn `researcher-codebase` on debugger's recommendation |
| Re-launching debugger without new evidence | Wastes Opus tokens, no progress | Run the test/researcher first; THEN re-launch |
| Continuing past 3 unproductive spawns | Burning tokens with no progress | Trigger circuit-breaker; escalate to user |

## Quick Reference

```
1. Capture failure context (raw evidence only — no hypothesizing)
2. Init investigation-state.md
3. Spawn debugger (Opus) — it owns DEBUG (D-E-B-U-G)
4. Loop:
   - Read debugger's Next Action
   - Spawn the recommended runner / researcher
   - Update state files
   - Re-launch debugger with the new evidence
   - Stop when regression spec hint is filled (or 3-spawn circuit-break)
5. Route the regression spec hint via routing-failures.md
6. /verify basic after the fix
7. Resolve → continue parent flow / Stuck → surface to user
```
