# Delegated Implementation

All implementation goes through the delegated pipeline. The main agent is the orchestrator — it describes features, reviews outputs, and routes failures. The **planner-spec** → **planner-review** → **writer-tests** → **writer-code** pipeline produces the code.

## The Flow

See `.claude/rules/tdd.md` for the TDD cycle definition and when to commit.
See `.claude/rules/spec-workflow.md` for the spec revision loop (steps 3-6).
See `.claude/rules/spec-formats.md` for spec templates and quality rules.
See `.claude/rules/git.md` for git usage and rules.

Verification tiers are defined in `.claude/rules/verification-tiers.md`.

```
1. Main agent describes the feature, identifies parallel waves
2. Research wave (when triggered — see below)                                  ── optional
3. Launch planner-specs per wave (in parallel)                                  ── SPEC phase
4. Launch planner-reviews as each spec completes (in parallel)
5. Main agent triages reviews, sends revisions back to planner-spec
6. Repeat 4–5 until planner-review confirms specs are clean
7. Main agent reviews final specs, creates shared prerequisites
8. Launch writer-tests as each spec is finalized (in parallel)                  ── RED phase
9. Launch reviewer-tests as each writer-tests completes (in parallel)
10. After ALL reviewer-tests pass: single runner-tests                          ── RED gate
11. Launch ALL writer-codes in parallel                                         ── GREEN phase
12. After ALL writer-codes complete: single runner-tests                        ── GREEN gate
13. Basic Verification Tier                                                    ─┐
14. Route failures → fix agents → Basic Verification Tier after each fix        │ REFACTOR
15. /simplify on changed code → Basic Verification Tier if changes              │
16. Repeat 13–15 until Basic Verification Tier is clean and /simplify is clean  │
17. Wiring (lib.rs, game.rs, shared.rs) → Basic Verification Tier             ─┘
18. Standard Verification Tier                                                  ── commit gate
19. Route failures → fix agents → Basic Verification Tier → repeat from 13
20. Commit
21. Full Verification Tier                                                      ── pre-merge gate
22. Route failures → fix agents → Basic Verification Tier → Standard → Full
23. Merge according to git rules
```

### Key principle: maximize parallelism, serialize only cargo

- **Specs**: run in parallel per wave (no cargo)
- **Reviews**: launch as each spec completes (no cargo)
- **Writer-tests**: launch as each spec is finalized (no cargo)
- **Reviewer-tests**: launch as each writer-tests completes (no cargo)
- **RED gate**: single `runner-tests` after ALL reviewer-tests pass (cargo — serialized)
- **Writer-codes**: run in parallel after RED gate (no cargo)
- **GREEN gate**: single `runner-tests` after ALL writer-codes complete (cargo — serialized)
- **Basic → Standard → Full Verification Tiers**: see `.claude/rules/verification-tiers.md`
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

Before planner-spec runs, launch research agents in parallel to surface conflicts early. This is optional — skip it for single-domain features with familiar APIs. See `.claude/rules/sub-agents.md` (Research Agents) for the full agent list and when each applies.

**Triggers** (any of these):
- Feature touches 2+ domains
- Feature uses unfamiliar Bevy 0.18 APIs
- Feature adds new messages, state transitions, or cross-plugin data flow

**Feed results into planner-spec**: include the research reports in the planner-spec feature description so specs account for known conflicts and correct API patterns from the start.

**Why**: pitfalls that surface late (during planner-review revision loops or post-implementation review) cost 2-10x more than catching them before spec writing. This gets proactive conflict detection without a new agent.

## Background Agent Rule

**ALL agents MUST be launched with `run_in_background: true`. No exceptions.** Every Agent tool call — runners, writers, reviewers, researchers, guards, planners — runs in the background. You will be notified when each completes.

When background agents are running, the main agent must **not** fill time with unnecessary analysis or speculation. End your turn with at most one brief status sentence and wait for notifications. Don't read files, don't plan ahead, don't analyze speculatively while waiting.


