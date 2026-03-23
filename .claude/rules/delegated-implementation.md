# Delegated Implementation

All implementation goes through the delegated pipeline. The main agent is the orchestrator — it describes features, reviews outputs, routes failures, and handles shared wiring. The **planner-spec** → **planner-review** → **writer-tests** → **writer-code** pipeline produces the code.

## The Flow

See `.claude/rules/tdd.md` for the TDD cycle definition and when to commit.
See `.claude/rules/spec-workflow.md` for the spec revision loop (steps 3-6).
See `.claude/rules/spec-formats.md` for spec templates and quality rules.

```
1. Main agent describes the feature in plain language
2. Research wave (when triggered — see below)                      ── optional
3. Launch planner-spec to produce all specs (behavioral + implementation, per domain)
4. Launch planner-review to pressure-test specs
5. Main agent reviews planner-review feedback, then sends feedback back to planner-spec to revise
6. Repeat 4–5 until planner-review confirms specs are clean (usually one revision)
7. Main agent reviews final specs, creates shared prerequisites
8. Launch ALL writer-tests in parallel                              ── RED phase
9. As each completes: launch reviewer-tests (read-only, parallel)   ── test review
10. After ALL reviewer-tests pass: single runner-tests               ── RED gate
11. Launch ALL writer-codes in parallel                              ── GREEN phase
12. After ALL writer-codes complete: launch verification wave        ─┐
13. Route Phase 3 failures through fix agents                        │ REFACTOR phase
14. Run /simplify on changed code                                    │
15. Repeat 12–14 until all agents pass and /simplify is clean        │
16. Main agent handles wiring (lib.rs, game.rs, shared.rs)          ─┘
17. Update session-state.md
```

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

## Parallel Domain Implementation

When implementing multiple domains simultaneously:

1. planner-spec produces ALL specs upfront (test spec + implementation spec for each domain)
2. Launch ALL writer-tests in parallel **as background agents** (`run_in_background: true`)
3. When each writer-tests completes (notified automatically): launch its reviewer-tests immediately (read-only, no cargo — safe to run in parallel)
4. After ALL reviewer-tests pass: run RED gate once (single runner-tests for all domains), then launch writer-codes
5. When ALL writer-codes have completed (they produce code only — no build verification): launch post-implementation verification per tier
6. Main agent handles wiring (lib.rs, game.rs, shared.rs)

### Background Agent Efficiency

When a background agent is running, the main agent should **not** fill time with unnecessary analysis or speculation. Background agents notify on completion — end your turn with a brief status message and wait. Don't read files, don't plan ahead, don't analyze speculatively while waiting.

### Safety Requirements for Parallel Execution

- Each agent ONLY touches files within its assigned domain directory
- The main agent handles all shared file modifications (`lib.rs`, `game.rs`, `shared.rs`, `mod.rs` at crate root)
- If two domains need a new shared type, the main agent creates it before launching agents
- If a domain needs a message from another domain, the main agent ensures the message type exists before launching agents
