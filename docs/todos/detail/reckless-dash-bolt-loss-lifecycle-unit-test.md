# RecklessDash Bolt-Loss Lifecycle Unit Test

## Summary
Add an integration-level unit test exercising the full RecklessDash bolt-loss overlay lifecycle end-to-end.

## Context
Wave 5 of the `refactor/bolt-loss-behavior` branch introduced `OriginalBoltLossBehavior` — a newtype component added to the breaker on dash entry that saves the original `BoltLossBehavior` and doubles it. It is restored on dash exit and on `OnExit(NodeState::Playing)` via `reckless_dash_cleanup_node`. The `OriginalBoltLossBehaviorOrphaned` scenario-runner invariant guards against orphaned state leaks.

What remains untested end-to-end in unit tests: the full round-trip — breaker enters dash → `BoltLossBehavior` doubles → `BoltLostOccurred` fires → doubled penalty applied → dash exits → `BoltLossBehavior` restored to original. Each phase is tested in isolation (Wave 5 unit tests, `double_penalty.rs`), but no single test runs all the systems together and asserts on the values at each stage.

This is a unit integration test using `App`, NOT a scenario. The scenario runner is for chaos exploration, regression replay from recordings, and invariant-based oracle checks — it cannot assert specific state values at specific frames. The lifecycle assertion requires checking concrete `BoltLossBehavior` values before/during/after dash, which is a unit-test concern. See `docs/architecture/scenario-runner.md`.

## Scope
- In:
  - New test(s) in `breaker-game/src/mutators/protocols/reckless_dash/tests/` (or add to existing lifecycle test file)
  - Use `App`/`with_dmg_pipeline()` test harness (established pattern)
  - Test: enter dashing → verify BoltLossBehavior doubled → fire BoltLostOccurred → verify penalty applied (doubled amount) → exit dashing → verify BoltLossBehavior restored
  - Test: mid-dash node exit via `OnExit(NodeState::Playing)` → verify `OriginalBoltLossBehavior` removed + `BoltLossBehavior` restored (regression for the correctness fix from Wave 5)
- Out:
  - New production code
  - Scenario RON files

## Dependencies
- Depends on: nothing (all production code exists)
- Blocks: nothing

## Notes
- The node-exit restoration test is the most valuable: it covers a path that was flagged as BLOCKING during Wave 5 Standard Verification (mid-dash node exit leaves `OriginalBoltLossBehavior` orphaned + `BoltLossBehavior` permanently doubled). The cleanup system was added to fix this; an integration test proves the fix works end-to-end.

## Status
`ready`
