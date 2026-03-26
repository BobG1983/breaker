# TDD: RED → GREEN → REFACTOR

## The Cycle

| Phase | Agent | Hard rule |
|-------|-------|-----------|
| RED | writer-tests | Tests MUST fail. NEVER implement production logic. Stubs must compile but do nothing. |
| RED gate | runner-tests | Tests must compile AND fail. MUST pass before launching writer-code. |
| GREEN | writer-code | NEVER modify tests. NEVER add untested features. |
| REFACTOR | See `verification-tiers.md` | Complete when Standard Verification Tier is clean and /simplify finds nothing. |

See `delegated-implementation.md` for the full pipeline flow (spec → review → RED → GREEN → REFACTOR → commit → merge).

## Hard Rules

- **writer-tests**: ONLY tests + stubs. NEVER production logic.
- **writer-code**: ONLY production code. NEVER modify tests. If a test seems wrong, flag it — do not change it.
- **Neither agent runs cargo. EVER.** Only runner agents execute cargo commands. See `cargo.md`.
- **No implementation before failing tests.** No skipping the RED gate. No exceptions.

## When to Commit and Merge

- **Commit** when Standard Verification Tier is clean and `/simplify` finds nothing. See `verification-tiers.md`.
- **Merge** when Full Verification Tier is clean. See `git.md` — Pre-Merge Guard Gate.
- Do NOT commit after GREEN. Do NOT commit mid-REFACTOR.
