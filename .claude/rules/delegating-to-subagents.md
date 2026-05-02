# Delegating to Sub-Agents

All implementation goes through the delegated pipeline. The main agent is the orchestrator — it describes features, reviews outputs, and routes failures. The pipeline runs **test specs first, all the way to RED, before code specs are written** — the failing tests on disk are the contract the impl spec is written against.

```
test spec → test-spec review → writer-tests → reviewer-tests → RED gate
  → code spec → code-spec review → writer-code → GREEN gate
```

Spec agents write to `.claude/specs/`; writers read from there.

## The Flow

See `.claude/rules/tdd.md` for the TDD cycle definition, RED/GREEN gate procedures, and when to commit.
See `.claude/rules/spec-workflow.md` for the spec revision loop and briefing requirements.
See `.claude/rules/spec-format-tests.md` and `.claude/rules/spec-format-code.md` for spec templates.
See `.claude/rules/routing-failures.md` for routing failures to fix agents.
See `.claude/rules/routing-repeated-failures.md` for when to stop retrying and escalate.
See `.claude/rules/verification-tiers.md` for Basic, Standard, and Full verification tier definitions.
See `.claude/rules/git.md` for git usage and rules.

```
 1. Main agent describes the feature, identifies parallel waves
 2. Research wave (when triggered — see below)                                  ── optional
 3. Shared prerequisites (cross-wave types/messages, if any)                    ── prereq
 4. Launch planning-writer-specs-tests per wave (in parallel)                   ── TEST SPEC
    Each writes its test spec to .claude/specs/<wave>-<feature>-tests.md
 5. Launch planning-reviewer-specs-tests as each test spec completes            ── TEST SPEC REVIEW
    (in parallel)
 6. Main agent triages reviews, sends revisions back to test-spec writers
 7. Repeat 5–6 until every test spec is clean
 8. Launch writer-tests per wave (reads test spec from .claude/specs/)          ── RED phase
    in parallel
 9. Launch reviewer-tests as each writer-tests completes (in parallel)
10. After ALL reviewer-tests pass: single runner-cargo                          ── RED gate
11. Launch planning-writer-specs-code per wave (in parallel)                    ── CODE SPEC
    Each reads BOTH the test spec AND the failing tests on disk; writes its
    impl spec to .claude/specs/<wave>-<feature>-code.md
12. Launch planning-reviewer-specs-code as each code spec completes             ── CODE SPEC REVIEW
    (in parallel). Reviewer cross-checks the impl plan against the actual
    failing tests, not just the test spec.
13. Main agent triages reviews, sends revisions back to code-spec writers
14. Repeat 12–13 until every code spec is clean
15. Launch ALL writer-codes in parallel (reads code spec from .claude/specs/)   ── GREEN phase
16. After ALL writer-codes complete: single runner-cargo                        ── GREEN gate
17. Basic Verification Tier                                                    ─┐
18. Route failures → fix agents → Basic Verification Tier after each fix        │ REFACTOR
19. /simplify on changed code → Basic Verification Tier if changes              │
20. Repeat 17–19 until Basic Verification Tier is clean and /simplify is clean  │
21. Wiring (lib.rs, game.rs, shared.rs) → Basic Verification Tier             ─┘
22. Standard Verification Tier                                                  ── commit gate
23. Route failures → fix agents → Basic Verification Tier → repeat from 17
24. Commit
25. Full Verification Tier                                                      ── pre-merge gate
26. Route failures → fix agents → Basic Verification Tier → Standard → Full
27. Merge according to git rules
```

Update session-state after every agent notification — see `.claude/rules/session-state.md`.

### Why test spec → RED → code spec (not parallel specs)

Writing the impl spec **after** the failing tests exist on disk lets the impl spec writer reference concrete test functions, exact assertions, and real file paths instead of inferring the contract from prose. It eliminates cross-spec drift (the tests pin the contract; the impl spec just describes how to satisfy them). The code-spec reviewer's job becomes "does this impl plan satisfy these specific failing tests?" rather than "do two prose documents agree?". Wave-level parallelism is unchanged — different waves still run their pipelines independently.

### Key principle: maximize parallelism within phases, serialize only cargo

- **Test-spec writers**: one per wave, in parallel (no cargo)
- **Test-spec reviewers**: one per wave, in parallel (no cargo)
- **Writer-tests**: one per wave, in parallel (no cargo)
- **Reviewer-tests**: launch as each writer-tests completes, in parallel (no cargo)
- **RED gate**: single `runner-cargo` after ALL reviewer-tests pass (cargo — serialized)
- **Code-spec writers**: one per wave, in parallel — but ONLY after RED gate (no cargo)
- **Code-spec reviewers**: one per wave, in parallel (no cargo)
- **Writer-codes**: one per wave, in parallel (no cargo)
- **GREEN gate**: single `runner-cargo` after ALL writer-codes complete (cargo — serialized)
- **Planning ahead**: launch test-spec writers for upcoming phases while current implementation is in flight

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

Each wave runs its own test-spec → test-spec-review → writer-tests → reviewer-tests pipeline in parallel. Then ALL waves batch into a single RED gate. After RED, each wave runs its own code-spec → code-spec-review → writer-code pipeline in parallel. Then ALL waves batch into a single GREEN gate, and a single verification sweep.

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
