# TDD: RED → GREEN → REFACTOR

> **Team mode**: when an agent team is active (see `.claude/rules/team-mode.md`), the TDD cycle is identical but phase transitions happen peer-to-peer between persistent team members instead of one-shot launches by the orchestrator. Hard rules below apply unchanged.

## The Cycle

| Phase | Agents | Hard rule |
|-------|--------|-----------|
| Test spec | planning-writer-specs-tests → planning-reviewer-specs-tests (revision loop) | Test spec must be clean before RED phase begins. |
| RED | writer-tests → reviewer-tests | Tests MUST fail. NEVER implement production logic. Stubs must compile but do nothing. |
| RED gate | runner-cargo | Tests must compile AND fail. MUST pass before launching the code spec phase. |
| Code spec | planning-writer-specs-code → planning-reviewer-specs-code (revision loop) | Code spec is written AFTER RED gate; failing tests on disk are the authoritative contract. Must be clean before writer-code runs. |
| GREEN | writer-code | NEVER modify tests. NEVER add untested features. |
| GREEN gate | runner-cargo | All tests must pass. |
| REFACTOR | See `verification-tiers.md` | Complete when Standard Verification Tier is clean and /simplify finds nothing. |

The full sequence: **test spec → review test spec (loop) → writer-tests → reviewer-tests → RED gate → code spec → review code spec (loop) → writer-code → GREEN gate → REFACTOR**.

See `delegating-to-subagents.md` for the full pipeline flow (spec → review → RED → spec → review → GREEN → REFACTOR → commit → merge).

## Hard Rules

- **writer-tests**: ONLY tests + stubs. NEVER production logic.
- **writer-code**: ONLY production code. NEVER modify tests. If a test seems wrong, flag it — do not change it.
- **Neither agent runs cargo. EVER.** Only runner agents execute cargo commands. See `cargo.md`.
- **No implementation before failing tests.** No skipping the RED gate. No exceptions.

## RED Gate Procedure

When implementing multiple domains, sequence these steps correctly:

1. Launch ALL **writer-tests** in parallel (one per domain, background) — each reads its test spec from `.claude/specs/`
2. As each writer-tests completes: launch its **reviewer-tests** immediately (background)
3. After ALL reviewer-tests pass: launch a single **runner-cargo** (cargo — serialized)
4. **Tests must compile.** If they don't → route back to writer-tests with the compiler error
5. **Tests must fail.** If any pass → the test is wrong or the behavior already exists. Investigate before proceeding.
6. After the RED gate passes: launch ALL **planning-writer-specs-code** in parallel (one per domain) — they read both the test spec AND the failing tests on disk; the failing tests are the authoritative contract
7. As each code spec completes: launch its **planning-reviewer-specs-code** (background); revise via the spec revision loop until every code spec is clean
8. Only after every code spec is clean: launch ALL **writer-codes** in parallel (background)

For single-domain work, the same sequence applies — it just has one agent per step.

Track RED gate status in session-state.md (the `RED Gate` column in the Specs table). Track the code spec phase in the `Code Spec` and `Code-Spec Review` columns.

## When to Commit and Merge

- **Commit** when Standard Verification Tier is clean and `/simplify` finds nothing. See `verification-tiers.md`.
- **Merge** when Full Verification Tier is clean. See `git.md` — Pre-Merge Guard Gate.
- Do NOT commit after GREEN gate. Do NOT commit mid-REFACTOR.
