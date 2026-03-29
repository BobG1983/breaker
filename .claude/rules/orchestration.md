# Orchestration Rules

Circuit breaking, RED gate, context pruning, and session-state for the main agent.

## Session State

**See `.claude/rules/session-state.md`** for the complete session-state protocol — format, update triggers, and the cardinal rule. Session-state is critical infrastructure. Read that file.

## Context Pruning

When launching fix agents, provide only:
- The specific hint block or regression spec
- The relevant session-state row (domain + failure entry)
- NOT the full output of every verification agent

## Circuit Breaking

After **3 failed attempts** at the same failure → stop routing → move to Stuck → surface to user.

What counts as 1 attempt:
- writer-code with fix spec
- writer-tests → writer-code cycle
- Main agent inline fix + rerun

What resets the counter:
- User provides new direction or changes spec
- Failure changes character (different error, different test, different file)

Do not: keep trying variations, weaken tests, escalate to different agent types hoping for luck.

## RED Gate

See `.claude/rules/tdd.md` for the full TDD cycle definition.

When implementing multiple domains, the orchestrator MUST sequence these steps correctly:

All agents launch with `run_in_background: true` — see `delegated-implementation.md` Background Agent Rule.

1. Launch ALL **writer-tests** in parallel (one per domain, background)
2. As each writer-tests completes: launch its **reviewer-tests** immediately (background)
3. After ALL reviewer-tests pass: launch a single **runner-tests** (background — cargo cannot run concurrently)
4. **Tests must compile.** If they don't → route back to writer-tests with the compiler error
5. **Tests must fail.** If any pass → the test is wrong or the behavior already exists. Investigate before proceeding.
6. After the RED gate passes: launch ALL **writer-codes** in parallel (background)

For single-domain work, the same sequence applies — it just has one agent per step.

Track RED gate status in session-state.md (the `RED Gate` column in the Specs table). See `.claude/rules/session-state.md` for the full format.

## Verification Tiers

See `.claude/rules/verification-tiers.md` for the authoritative definition of the Basic, Standard, and Full Verification Tiers — which agents, when to run each, and the pipeline flow.

See `.claude/rules/sub-agents.md` for the complete agent directory — every agent, its purpose, and when to use it (including pre-planning research agents).
