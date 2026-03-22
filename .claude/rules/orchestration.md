# Orchestration Rules

Session state management, verification tiers, circuit breaking, RED gate, and context pruning for the main agent.

## Session State Protocol

Maintain `.claude/agent-memory/orchestrator/ephemeral/session-state.md`.

**Create** at start of every task.
**Update** after every phase transition.
**Read** before every routing decision in Phase 3.

Format (keep under 80 lines):

```
# Session State
## Task
[one-line description]
## Decisions
- [key decisions with rationale]
## Specs
| Domain | Spec Status | Writer-Tests | Test Review | RED Gate | Writer-Code | Notes |
## Phase 2 Results
| Agent | Status | Action Needed |
## Active Failures
- [failure]: attempt N — [what was tried] → [result]
## Resolved
- [failure]: fixed attempt N, verified by [agent]
## Stuck
- [failure]: N attempts, needs human input
```

When a Phase 2 failure or Phase 3 fix reveals an earlier decision was wrong, mark it REVISED in Decisions with rationale and list affected features. If completed code is now wrong, add rework entry to Active Failures. When a decision revision represents a recurring pattern, record in orchestrator stable memory.

## Context Pruning

When launching fix agents in Phase 3, provide only:
- The specific hint block or regression spec
- The relevant session-state row (domain + failure entry)
- NOT the full output of every Phase 2 agent

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

After each writer-tests completes, the orchestrator MUST run reviewer-tests and the RED gate before launching writer-code:

1. Launch **reviewer-tests** to verify tests match the spec (PASS → proceed; BLOCKING findings → route back to writer-tests)
2. Launch **runner-tests** to compile and run the new tests
3. **Tests must compile.** If they don't → route back to writer-tests with the compiler error
4. **Tests must fail.** If any pass → the test is wrong or the behavior already exists. Investigate before proceeding.
5. Only after reviewer-tests passes AND the RED gate passes → launch writer-code

Track RED gate status in session-state.md (the `RED Gate` column in the Specs table).

## Two-Tier Verification

### Standard — default for all work

Launch: **runner-linting** + **runner-tests** + **reviewer-correctness** + **reviewer-quality** + **reviewer-bevy-api** + **reviewer-architecture** + **reviewer-performance**

### Pre-planning research (before planner-spec — see `delegated-implementation.md` step 2)

| Condition | Agent |
|-----------|-------|
| Feature touches 2+ domains, or adds new messages/state transitions/cross-plugin flow | **researcher-system-dependencies** |
| Feature uses unfamiliar Bevy 0.18 APIs | **researcher-bevy-api** |
| Modifying existing types, systems, or messages | **researcher-impact** |
| Modifying existing behavior (need to understand current flow) | **researcher-codebase** |

### Conditional agents (add to either tier when triggered)

| Condition | Agent |
|-----------|-------|
| Pre-commit (end of REFACTOR phase) or phase boundary | **runner-scenarios** |
| 3+ systems added, or cross-plugin data flow | **researcher-system-dependencies** |
| New gameplay mechanic or upgrade designed | **guard-game-design** |
| Phase complete or significant structural change | **guard-docs** |
| New dependencies added or security-sensitive code | **guard-security** |
| New dependencies added or before release | **guard-dependencies** |
| New mechanic needs adversarial scenario coverage | **writer-scenarios** |
| Phase complete or multiple sessions since last audit | **guard-agent-memory** |
| Choosing a new dependency | **researcher-crates** |
| Modifying code with non-obvious history | **researcher-git** |

All agents in a tier launch in parallel.

planner-spec recommends a tier. Main agent may bump up (never down without good reason).
