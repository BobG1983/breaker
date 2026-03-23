# Delegated Implementation

All implementation goes through the delegated pipeline. The main agent is the orchestrator — it describes features, reviews outputs, routes failures, and handles shared wiring. The **planner-spec** → **planner-review** → **writer-tests** → **writer-code** pipeline produces the code.

## The Flow

See `.claude/rules/tdd.md` for the TDD cycle definition and when to commit.
See `.claude/rules/spec-workflow.md` for the spec revision loop (steps 3-6).
See `.claude/rules/spec-formats.md` for spec templates and quality rules.
See `.claude/rules/git.md` for git usage and rules.

```
1. Main agent describes the feature, identifies parallel waves
2. Research wave (when triggered — see below)                      ── optional
3. Launch planner-specs per wave (in parallel)                      ── SPEC phase
4. Launch planner-reviews as each spec completes (in parallel)
5. Main agent triages reviews, sends revisions back to planner-spec
6. Repeat 4–5 until planner-review confirms specs are clean
7. Main agent reviews final specs, creates shared prerequisites
8. Launch writer-tests as each spec is finalized (in parallel)      ── RED phase
9. Launch reviewer-tests as each writer-tests completes (in parallel)
10. After ALL reviewer-tests pass: single runner-tests               ── RED gate (cargo — serialized)
11. Launch ALL writer-codes in parallel                              ── GREEN phase
12. After ALL writer-codes complete: single runner-tests             ── GREEN gate (cargo — serialized)
13. Launch verification wave (lint, reviewers, scenarios)           ─┐
14. Route failures through fix agents                                │ REFACTOR phase
15. Run /simplify on changed code                                    │
16. Repeat 13–15 until all agents pass and /simplify is clean        │
17. Main agent handles wiring (lib.rs, game.rs, shared.rs)          ─┘
18. Update session-state.md
19. Run the full verification suite (all lints, tests, reviewers, and guards)  ── BUG IDENTIFICATION AND FIX phase
20. Commit and Merge according to git rules. 
```

### Key principle: maximize parallelism, serialize only cargo

- **Specs**: run in parallel per wave (no cargo)
- **Reviews**: launch as each spec completes (no cargo)
- **Writer-tests**: launch as each spec is finalized (no cargo)
- **Reviewer-tests**: launch as each writer-tests completes (no cargo)
- **RED gate**: single `runner-tests` after ALL reviewer-tests pass (cargo — serialized)
- **Writer-codes**: run in parallel after RED gate (no cargo)
- **GREEN gate**: single `runner-tests` after ALL writer-codes complete (cargo — serialized)
- **Verification**: lint + reviewers + scenarios (cargo steps serialized)
- **Planning ahead**: launch planner-spec/planner-review for upcoming phases while current implementation is in flight

## Parallel Waves

When producing a plan, the main agent **MUST** identify which parts of the work can run in parallel. Group independent work into **waves**.

**How to identify waves:**
- Work that touches **different files** can run in parallel
- Work that touches **different domains** can usually run in parallel 
- Work with **no data dependencies** can run in parallel
- Cross-domain types (queries, filters, messages) are **shared prerequisites** — create in a prerequisite wave or refactor in a final wave

**Example:** Migrating bolt, breaker, and cells to Position2D:
- Wave 1: bolt domain (bolt/systems/*)
- Wave 2: breaker domain (breaker/systems/*)
- Wave 3: cells+walls domain (run/node/systems/*, wall/systems/*)
- Wave 4: cross-domain updates (collision system reads)
- Main agent creates shared prerequisites (query aliases) before all waves launch

Each wave runs its own spec → review → writer-tests → reviewer-tests pipeline in parallel. Then ALL waves batch into a single RED gate, single GREEN gate, and single verification sweep.

## Research Wave (Step 2)

Before planner-spec runs, launch research agents in parallel to surface conflicts early. This is optional — skip it for single-domain features with familiar APIs.

**Triggers** (any of these):
- Feature touches 2+ domains
- Feature uses unfamiliar Bevy 0.18 APIs
- Feature adds new messages, state transitions, or cross-plugin data flow

**Agents to launch** (in parallel):
- **researcher-system-dependencies** — "Analyze potential conflicts for a feature that adds [X] to [domain A] and [domain B]. Focus on query conflicts, message flow gaps, and ordering issues."
- **researcher-bevy-api** — only when unfamiliar APIs are involved
- **researcher-impact** — when modifying existing types/systems/messages
- **researcher-codebase** — when modifying existing behavior (need to understand current flow)

**Feed results into planner-spec**: include the research reports in the planner-spec feature description so specs account for known conflicts and correct API patterns from the start.

**Why**: pitfalls that surface late (during planner-review revision loops or post-implementation review) cost 2-10x more than catching them before spec writing. This gets proactive conflict detection without a new agent.

## Background Agent Efficiency

When a background agent is running, the main agent should **not** fill time with unnecessary analysis or speculation. Background agents notify on completion — end your turn with a brief status message and wait. Don't read files, don't plan ahead, don't analyze speculatively while waiting.


