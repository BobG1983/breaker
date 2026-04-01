# Delegating to Sub-Agents

All implementation goes through the delegated pipeline. The main agent is the orchestrator — it describes features, reviews outputs, and routes failures. The **planning-writer-specs** → **planning-reviewer-specs** → **writer-tests** → **writer-code** pipeline produces the code. Spec agents write to `.claude/specs/`; writers read from there.

## The Flow

See `.claude/rules/tdd.md` for the TDD cycle definition, RED/GREEN gate procedures, and when to commit.
See `.claude/rules/spec-workflow.md` for the spec revision loop (steps 3-6) and briefing requirements.
See `.claude/rules/spec-format-tests.md` and `.claude/rules/spec-format-code.md` for spec templates.
See `.claude/rules/routing-failures.md` for routing failures to fix agents.
See `.claude/rules/routing-repeated-failures.md` for when to stop retrying and escalate.
See `.claude/rules/verification-tiers.md` for Basic, Standard, and Full verification tier definitions.
See `.claude/rules/git.md` for git usage and rules.

```
1. Main agent describes the feature, identifies parallel waves
2. Research wave (when triggered — see below)                                  ── optional
3. Launch planning-writer-specs-tests + planning-writer-specs-code             ── SPEC phase
   per wave (in parallel — both write to .claude/specs/)
4. Launch planning-reviewer-specs-tests + planning-reviewer-specs-code         ── SPEC REVIEW
   as each spec completes (in parallel)
5. Main agent triages reviews, sends revisions back to spec writers
6. Repeat 4–5 until both reviewers confirm specs are clean
7. Main agent reviews final spec summaries, creates shared prerequisites
8. Launch writer-tests per wave (reads spec from .claude/specs/)               ── RED phase
9. Launch reviewer-tests as each writer-tests completes (in parallel)
10. After ALL reviewer-tests pass: single runner-tests                         ── RED gate
11. Launch ALL writer-codes in parallel (reads spec from .claude/specs/)       ── GREEN phase
12. After ALL writer-codes complete: single runner-tests                       ── GREEN gate
13. Basic Verification Tier                                                   ─┐
14. Route failures → fix agents → Basic Verification Tier after each fix       │ REFACTOR
15. /simplify on changed code → Basic Verification Tier if changes             │
16. Repeat 13–15 until Basic Verification Tier is clean and /simplify is clean │
17. Wiring (lib.rs, game.rs, shared.rs) → Basic Verification Tier            ─┘
18. Standard Verification Tier                                                 ── commit gate
19. Route failures → fix agents → Basic Verification Tier → repeat from 13
20. Commit
21. Full Verification Tier                                                     ── pre-merge gate
22. Route failures → fix agents → Basic Verification Tier → Standard → Full
23. Merge according to git rules
```

Update session-state after every agent notification — see `.claude/rules/session-state.md`.

### Key principle: maximize parallelism, serialize only cargo

- **Spec writers**: planning-writer-specs-tests + planning-writer-specs-code run in parallel (no cargo)
- **Spec reviewers**: planning-reviewer-specs-tests + planning-reviewer-specs-code run in parallel (no cargo)
- **Writer-tests**: launch as each test spec is finalized (no cargo)
- **Reviewer-tests**: launch as each writer-tests completes (no cargo)
- **RED gate**: single `runner-tests` after ALL reviewer-tests pass (cargo — serialized)
- **Writer-codes**: run in parallel after RED gate (no cargo)
- **GREEN gate**: single `runner-tests` after ALL writer-codes complete (cargo — serialized)
- **Planning ahead**: launch spec writers for upcoming phases while current implementation is in flight

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

Before spec writers run, launch research agents in parallel to surface conflicts early. This is optional — skip it for single-domain features with familiar APIs. See `.claude/rules/sub-agents.md` (Research Agents) for the full agent list and when each applies.

**Triggers** (any of these):
- Feature touches 2+ domains
- Feature uses unfamiliar Bevy 0.18 APIs
- Feature adds new messages, state transitions, or cross-plugin data flow

**Feed results into spec writers**: include the research reports in the spec writer prompts so specs account for known conflicts and correct API patterns from the start.

**Why**: pitfalls that surface late (during reviewer revision loops or post-implementation review) cost 2-10x more than catching them before spec writing. This gets proactive conflict detection without a new agent.

## Background Agent Rule

**ALL agents MUST be launched with `run_in_background: true`. No exceptions.** Every Agent tool call — runners, writers, reviewers, researchers, guards, planners — runs in the background. You will be notified when each completes.

When background agents are running, the main agent must **not** fill time with unnecessary analysis or speculation. End your turn with at most one brief status sentence and wait for notifications. Don't read files, don't plan ahead, don't analyze speculatively while waiting.

## Context Pruning

When launching fix agents, provide only:
- The specific hint block or regression spec
- The relevant session-state row (domain + failure entry)
- NOT the full output of every verification agent
