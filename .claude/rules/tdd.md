# TDD: RED → GREEN → REFACTOR

## The Cycle

| Phase | Agent | Hard rule |
|-------|-------|-----------|
| RED | writer-tests | Tests MUST fail. NEVER implement production logic. Stubs must compile but do nothing. |
| RED gate | runner-tests | Tests must compile AND fail. MUST pass before launching writer-code. |
| GREEN | writer-code | NEVER modify tests. NEVER add untested features. |
| REFACTOR | See `verification-tiers.md` | Complete when Standard Verification Tier is clean and /simplify finds nothing. |

See `delegating-to-subagents.md` for the full pipeline flow (spec → review → RED → GREEN → REFACTOR → commit → merge).

## Hard Rules

- **writer-tests**: ONLY tests + stubs. NEVER production logic.
- **writer-code**: ONLY production code. NEVER modify tests. If a test seems wrong, flag it — do not change it.
- **Neither agent runs cargo. EVER.** Only runner agents execute cargo commands. See `cargo.md`.
- **No implementation before failing tests.** No skipping the RED gate. No exceptions.

## RED Gate Procedure

When implementing multiple domains, sequence these steps correctly:

1. Launch ALL **writer-tests** in parallel (one per domain, background)
2. As each writer-tests completes: launch its **reviewer-tests** immediately (background)
3. After ALL reviewer-tests pass: launch a single **runner-tests** (cargo — serialized)
4. **Tests must compile.** If they don't → route back to writer-tests with the compiler error
5. **Tests must fail.** If any pass → the test is wrong or the behavior already exists. Investigate before proceeding.
6. After the RED gate passes: launch ALL **writer-codes** in parallel (background)

For single-domain work, the same sequence applies — it just has one agent per step.

Track RED gate status in session-state.md (the `RED Gate` column in the Specs table).

## When to Commit and Merge

- **Commit** when Standard Verification Tier is clean and `/simplify` finds nothing. See `verification-tiers.md`.
- **Merge** when Full Verification Tier is clean. See `git.md` — Pre-Merge Guard Gate.
- Do NOT commit after GREEN. Do NOT commit mid-REFACTOR.
