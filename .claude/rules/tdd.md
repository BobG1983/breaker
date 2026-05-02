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

When implementing parallel sub-waves (e.g., `parallel: [4A, 4B, 4C]` per `plan-format.md`), sequence these steps correctly:

1. Dispatch ALL **writer-tests** in parallel (one slot per sub-wave, background) — each reads its test spec from `.claude/specs/wave<N><LETTER>-<feature>-tests.md`
2. As each writer-tests completes: dispatch its **reviewer-tests** immediately (background)
3. After ALL sub-waves' reviewer-tests pass: dispatch a single **runner-cargo** (cargo — serialized) for the BATCHED RED gate
4. **Tests must compile.** If they don't → route back to the appropriate writer-tests slot with the compiler error
5. **Tests must fail.** If any pass → the test is wrong or the behavior already exists. Investigate before proceeding.
6. After the RED gate passes: dispatch ALL **planning-writer-specs-code** in parallel (one slot per sub-wave) — each reads both its test spec AND its failing tests on disk; the failing tests are the authoritative contract
7. As each code spec completes: dispatch its **planning-reviewer-specs-code** (background); revise via the spec revision loop until every code spec is clean
8. Only after every code spec is clean: dispatch ALL **writer-codes** in parallel (background)

For non-parallel waves (single sub-wave), the same sequence applies with one slot per step.

In team mode, the wave-coordinator drives this sequence — see `.claude/rules/team-mode.md`. The orchestrator (team-lead) only sees the milestone events the coordinator surfaces.

Track RED gate status in session-state.md (the `RED Gate` column in the Specs table, one row per sub-wave). Track the code spec phase in the `Code Spec` and `Code-Spec Review` columns.

### Single batched gate, not per-sub-wave gates

The RED gate (and GREEN gate) runs ONCE per wave, after ALL sub-waves are ready. Running runner-cargo per sub-wave would multiply cargo invocations and serialize them anyway. The batched gate proves all sub-waves' tests fail together AND none of them broke each other (e.g., 4A's tests don't accidentally pass because 4B's stub is wrong).

Routing on a batched failure: runner-cargo categorizes failures by sub-wave (matching the failing test path's directory) and replies to each sub-wave's writer-tests slot independently. wave-coordinator and team-lead are CC'd. See `.claude/rules/routing-failures.md` (team-mode addendum).

## When to Commit and Merge

- **Commit** when Standard Verification Tier is clean and `/simplify` finds nothing. See `verification-tiers.md`.
- **Merge** when Full Verification Tier is clean. See `git.md` — Pre-Merge Guard Gate.
- Do NOT commit after GREEN gate. Do NOT commit mid-REFACTOR.
