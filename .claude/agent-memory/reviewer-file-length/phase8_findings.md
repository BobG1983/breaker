---
name: Phase 8 findings — scenario-runner-wiring scan
description: Wave 12 scan (2026-04-07 feature/scenario-runner-wiring): 2 HIGH, 2 MEDIUM actionable, 4 MEDIUM monitor in breaker-scenario-runner + propagate_breaker_changes. Spec at .claude/specs/file-splits.md
type: project
---

## Scope

Focused scan: breaker-scenario-runner/src/ and breaker-game/src/debug/hot_reload/systems/propagate_breaker_changes/.

## HIGH (2 files)

- `breaker-scenario-runner/src/invariants/screenshot.rs` (1169 lines, 98 prod, 1071 test, 31 fns) -- Strategy A + sub-split
- `breaker-scenario-runner/src/runner/tests/run_log_tests.rs` (1150 lines, all test, 49 fns) -- Strategy C sub-split into 9 files

## MEDIUM actionable (2 files)

- `breaker-scenario-runner/src/runner/execution.rs` (584 lines, all prod, 0 tests) -- Strategy B: 3 concerns (in-process run, subprocess batching, stress scenarios)
- `breaker-scenario-runner/src/runner/app.rs` (547 lines, 501 prod, 46 test, 2 fns) -- Strategy A: move 2 inline tests to existing app_tests.rs

## MEDIUM monitor (4 files, all extracted test files under 800)

- `runner/tests/tiling_tests.rs` (650, 37 fns)
- `runner/tests/app_tests.rs` (634, 22 fns -- will grow to ~680 after app.rs merge)
- `propagate_breaker_changes/tests.rs` (616, 10 fns)
- `runner/tests/streaming_tests.rs` (559, 33 fns)

## Batching

All 4 actionable items are in breaker-scenario-runner, touching different modules. All can run in parallel.
