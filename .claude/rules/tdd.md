# TDD: RED → GREEN → REFACTOR

Authoritative reference for the TDD cycle in the delegated agent pipeline.

## The Cycle

| Phase | Agent | What happens | Hard rule |
|-------|-------|-------------|-----------|
| RED | writer-tests | Write failing tests + compilable stubs | Tests MUST fail. NEVER implement production logic. |
| RED gate | runner-tests (orchestrator) | Verify tests compile and fail | MUST pass before launching writer-code. |
| GREEN | writer-code | Minimum code to pass tests | NEVER modify tests. NEVER add untested features. |
| REFACTOR | Phase 2 + Phase 3 + /simplify | Reviewers find issues, fix agents resolve them | Complete when all reviewers pass and /simplify is clean. |

## REFACTOR Is Distributed

- **Reviewers find what to refactor**: reviewer-quality (idioms, naming, duplication), reviewer-correctness (logic bugs), reviewer-bevy-api (deprecated patterns), reviewer-performance (ECS inefficiencies), reviewer-architecture (structural violations)
- **Phase 3 routing executes the fixes**: reviewer findings route to writer-code (or inline main-agent fixes) per the failure routing table in agent-flow.md
- **`/simplify` catches the rest**: runs on changed code after Phase 3 fixes settle
- **Wiring is the final cleanup**: main agent integrates modules, ensuring the public API surface is clean

REFACTOR is complete when all Phase 2 agents pass and `/simplify` finds nothing to change.

## RED/GREEN Boundary

- **writer-tests**: ONLY tests + stubs. NEVER production logic. Stubs must compile but do nothing.
- **writer-code**: ONLY production code. NEVER modify tests. If a test seems wrong, flag it — do not change it.
- **NEITHER agent runs cargo. EVER.** Only runner agents (runner-tests, runner-linting, runner-scenarios) execute cargo commands. Multiple agents edit files concurrently — cargo builds would see partial state and lock contention corrupts builds.

## RED Gate

After writer-tests completes, the orchestrator launches runner-tests to verify:

1. Tests compile
2. Tests fail (if any pass → the test is wrong or the behavior already exists — investigate before proceeding)

Only after the RED gate passes does the orchestrator launch writer-code.

## The Cycle Loops

The flow is not linear — it repeats until clean:

1. RED → GREEN → REFACTOR (Phase 2 reviewers + Phase 3 fixes + /simplify)
2. If Phase 3 fixes introduce new code → run verification again (REFACTOR repeats)
3. Only when ALL reviewers pass and `/simplify` finds nothing to change → commit

Do NOT commit after GREEN. Do NOT commit mid-REFACTOR. Commit only when the full cycle is clean.

## No Exceptions

No implementation before failing tests. No skipping the RED gate. No exceptions.
