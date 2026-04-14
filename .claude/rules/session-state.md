# Session State — ALWAYS KEEP UPDATED

Session-state is the orchestrator's single source of truth for task progress. Without it, context compression loses critical state and failure routing breaks.

**Cardinal rule: After EVERY background agent notification, update session-state BEFORE doing anything else.** Not after. Not when you remember. BEFORE. This is the single most important habit for the orchestrator.

## Location

`.claude/state/session-state.md`

## Lifecycle

| Event | Action |
|-------|--------|
| Start of task | **Create** session-state with Task, Decisions, empty Specs table |
| After EVERY agent notification | **Update** the relevant row/section FIRST, before any other action |
| Before failure routing | **Read** session-state to check attempt count and history |
| Decision changes | **Mark** old decision as REVISED with rationale |
| Task complete | Session-state remains for reference (ephemeral, cleaned on next session) |

## Update Triggers

Every one of these events requires an immediate session-state update:

| Agent completes | Update |
|----------------|--------|
| planning-writer-specs-tests | Specs table → Test Spec column |
| planning-writer-specs-code | Specs table → Code Spec column |
| planning-reviewer-specs-tests | Specs table → Test Review column |
| planning-reviewer-specs-code | Specs table → Code Review column |
| writer-tests | Specs table → Writer-Tests column |
| reviewer-tests | Specs table → Test Review column |
| runner-tests (RED gate) | Specs table → RED Gate column |
| writer-code | Specs table → Writer-Code column |
| runner-tests (GREEN gate) | Specs table → GREEN column |
| runner-linting | Verification Results table |
| runner-tests (verification) | Verification Results table |
| reviewer-completeness | Verification Results table |
| Any reviewer (Standard tier) | Verification Results table |
| Any guard (Full tier) | Verification Results table |
| Fix agent | Active Failures → update attempt count and result |

## Format

Keep under 80 lines. Use this exact structure:

```
# Session State

## Plan
[The **FULL** filepath to the current plan that is being worked on]

## Wave
[Which wave of the plan is currently in flight — e.g., "Wave 1: bolt domain"]

## Todo
[The todo item being worked on]
- Detail: [path to docs/todos/detail/<file>.md]

## Specs
[The full file path to every fully completed, reviewed, and accepted spec with a one line description. One spec per line.]

## Task
[one-line description]

## Decisions
- [key decisions with rationale]
- REVISED: [old decision] → [new decision] — [why]

## Specs
| Domain | Test Spec | Code Spec | Spec Review | Writer-Tests | Test Review | RED Gate | Writer-Code | GREEN | Notes |

## Verification Results
| Agent | Status | Action Needed |

## Active Failures
- [failure]: attempt N 
  — Hypothesis: [Hypothesis for what is wrong]
  - Test written to confirm: [name of test & file]
  - Proven: [True/False]
  - Fix Attempted: [what fixwas tried] 
  → RESULT: [result]

## Resolved
- [failure]: fixed attempt N, verified by [test]

## Stuck
- [failure]: N attempts pre-investigate, N attempts using /investigate, needs human input
```

## Column Values

Use these exact status values in the Specs table:

| Value | Meaning |
|-------|---------|
| `pending` | Not started |
| `writing` | Agent currently running |
| `done` | Agent completed successfully |
| `reviewing` | Reviewer currently running |
| `approved` | Reviewer approved |
| `revising` | Spec being revised after review |
| `PASS` | Gate passed |
| `FAIL` | Gate failed — see Active Failures |
| `-` | Not applicable yet |

## Decision Revisions

When a verification failure or fix attempt reveals an earlier decision was wrong:
1. Mark the old decision as `REVISED` in Decisions with rationale
2. List affected spec rows
3. If completed code is now wrong, add a rework entry to Active Failures
4. When a revision represents a recurring pattern, record in orchestrator stable memory

## Circuit Breaking

After **3 failed attempts** at the same failure → stop routing → move to Stuck. See `.claude/rules/routing-repeated-failures.md` for the full protocol.
