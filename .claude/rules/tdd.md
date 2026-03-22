# TDD: RED → GREEN → REFACTOR

Authoritative reference for the TDD cycle in the delegated agent pipeline.

## The Cycle

| Phase | Agent | What happens | Hard rule |
|-------|-------|-------------|-----------|
| RED | writer-tests | Write failing tests + compilable stubs | Tests MUST fail. NEVER implement production logic. |
| Test review | reviewer-tests (orchestrator) | Verify tests match spec | MUST pass before RED gate. |
| RED gate | runner-tests (orchestrator) | Verify tests compile and fail | MUST pass before launching writer-code. |
| GREEN | writer-code | Minimum code to pass tests | NEVER modify tests. NEVER add untested features. |
| REFACTOR | Phase 2 + Phase 3 + /simplify | Reviewers find issues, fix agents resolve them | Complete when all reviewers pass and /simplify is clean. |

## REFACTOR Is Distributed

- **Reviewers find what to refactor**: reviewer-quality (idioms, naming, duplication), reviewer-correctness (logic bugs), reviewer-bevy-api (deprecated patterns), reviewer-performance (ECS inefficiencies), reviewer-architecture (structural violations)
- **Phase 3 routing executes the fixes**: reviewer findings route to writer-code (or inline main-agent fixes) per `.claude/rules/failure-routing.md`
- **`/simplify` catches the rest**: runs on changed code after Phase 3 fixes settle
- **Wiring is the final cleanup**: main agent integrates modules, ensuring the public API surface is clean

## RED/GREEN Boundary

- **writer-tests**: ONLY tests + stubs. NEVER production logic. Stubs must compile but do nothing.
- **writer-code**: ONLY production code. NEVER modify tests. If a test seems wrong, flag it — do not change it.
- **NEITHER agent runs cargo. EVER.** Only runner agents (runner-tests, runner-linting, runner-scenarios) execute cargo commands. See `.claude/rules/cargo.md`.

## Test Review

After writer-tests completes, the orchestrator launches **reviewer-tests** to verify tests match the spec. If reviewer-tests reports BLOCKING findings, route back to writer-tests with a test revision spec. Only proceed to the RED gate once reviewer-tests passes.

## RED Gate

After reviewer-tests passes, the orchestrator launches runner-tests to verify:

1. Tests compile
2. Tests fail (if any pass → the test is wrong or the behavior already exists — investigate before proceeding)

Only after both reviewer-tests and the RED gate pass does the orchestrator launch writer-code.

## When to Commit

The cycle repeats until clean:

1. RED → GREEN → REFACTOR (Phase 2 verification + Phase 3 fixes + /simplify)
2. If Phase 3 fixes introduce new code → re-run verification (REFACTOR repeats)
3. Only when ALL verifiers pass and `/simplify` finds nothing → commit

**Pre-commit checklist:**
- All verification agents pass (per tier — see `.claude/rules/orchestration.md`)
- `/simplify` finds nothing to change
- Commit with conventional format — see `.claude/rules/commit-format.md`

Do NOT commit after GREEN. Do NOT commit mid-REFACTOR. Commit only when the full cycle is clean.

## No Exceptions

No implementation before failing tests. No skipping the RED gate. No exceptions.
