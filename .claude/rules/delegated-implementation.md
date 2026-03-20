# Delegated Implementation

All implementation goes through the delegated pipeline. The main agent is the orchestrator — it describes features, reviews outputs, routes failures, and handles shared wiring. The **planner-spec** → **planner-review** → **writer-tests** → **writer-code** pipeline produces the code.

## The Flow

See `.claude/rules/tdd.md` for the TDD cycle definition and when to commit.
See `.claude/rules/spec-workflow.md` for the spec revision loop (steps 2-5).
See `.claude/rules/spec-formats.md` for spec templates and quality rules.

```
1. Main agent describes the feature in plain language
2. Launch planner-spec to produce all specs (behavioral + implementation, per domain)
3. Launch planner-review to pressure-test specs
4. Main agent reviews planner-review feedback, then sends feedback back to planner-spec to revise
5. Repeat 3–4 until planner-review confirms specs are clean (usually one revision)
6. Main agent reviews final specs, creates shared prerequisites
7. Launch ALL writer-tests in parallel                              ── RED phase
8. Launch runner-tests to verify tests compile and fail             ── RED gate
9. As each passes RED gate: review, launch writer-code              ── GREEN phase
10. After ALL writer-codes complete: launch verification wave        ─┐
11. Route Phase 3 failures through fix agents                        │ REFACTOR phase
12. Run /simplify on changed code                                    │
13. Repeat 10–12 until all agents pass and /simplify is clean        │
14. Main agent handles wiring (lib.rs, game.rs, shared.rs)          ─┘
15. Update session-state.md
```

## Parallel Domain Implementation

When implementing multiple domains simultaneously:

1. planner-spec produces ALL specs upfront (test spec + implementation spec for each domain)
2. Launch ALL writer-tests in parallel **as background agents** (`run_in_background: true`)
3. When each writer-tests completes (notified automatically): run RED gate (runner-tests), then launch its writer-code — do NOT wait for other writer-tests still running
4. When ALL writer-codes have completed (they produce code only — no build verification): launch post-implementation verification per tier
5. Main agent handles wiring (lib.rs, game.rs, shared.rs)

### Safety Requirements for Parallel Execution

- Each agent ONLY touches files within its assigned domain directory
- The main agent handles all shared file modifications (`lib.rs`, `game.rs`, `shared.rs`, `mod.rs` at crate root)
- If two domains need a new shared type, the main agent creates it before launching agents
- If a domain needs a message from another domain, the main agent ensures the message type exists before launching agents
