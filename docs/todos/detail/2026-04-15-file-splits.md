# File Splits — 2026-04-15

Scan of changed files in `breaker-scenario-runner/src/runner/`.

## File Length Review

### Files Over Threshold

| File | Total | Prod | Tests | Test Fns | Strategy | Priority |
|------|-------|------|-------|----------|----------|----------|
| `breaker-scenario-runner/src/runner/tests/app_tests.rs` | 1110 | 0 | 1110 | 37 | C: oversized test file sub-split | HIGH |
| `breaker-scenario-runner/src/runner/tests/tiling_tests.rs` | 562 | 0 | 562 | 31 | — (monitor) | MEDIUM |
| `breaker-scenario-runner/src/runner/app.rs` | 546 | 546 | 0 | 0 | B: concern separation | MEDIUM |

### Files Under Threshold (no action)

| File | Total |
|------|-------|
| `breaker-scenario-runner/src/runner/tiling.rs` | 131 |
| `breaker-scenario-runner/src/runner/streaming.rs` | 224 |
| `breaker-scenario-runner/src/runner/mod.rs` | 22 |

### Priority Guide
- **HIGH**: 1000+ lines, or 800+ test lines (biggest context pollution impact, split immediately, across 2+ files)
- **MEDIUM**: 501-999 lines (noticeable, split at least once)
- **LOW**: 400-500 lines (flag for awareness, will need splitting soon)

---

## Refactor Specs

### 1. HIGH — `app_tests.rs` (1110 lines, 37 tests)

**Refactor spec hint:**
- Source file: `breaker-scenario-runner/src/runner/tests/app_tests.rs`
- Total lines: 1110 (prod: 0, tests: 1110)
- Strategy: C (oversized test file, already extracted)
- Parent module: `breaker-scenario-runner/src/runner/tests/mod.rs` declares `mod app_tests;`
- External imports: none (internal test module only)
- Target structure:
  ```
  breaker-scenario-runner/src/runner/tests/
    app_tests/
      mod.rs                    // mod helpers; mod timeout_tests; mod drain_logs_tests; ...
      helpers.rs                // shared helper functions (apply_tile_layout_app, spawn_primary_monitor, spawn_primary_window_tracked, make_definition)
      timeout_tests.rs          // is_timed_out tests
      drain_logs_tests.rs       // drain_remaining_logs tests
      guarded_update_tests.rs   // guarded_update tests
      snapshot_tests.rs         // snapshot_eval_data tests
      sync_ui_scale_tests.rs    // sync_ui_scale tests
      should_fail_fast_tests.rs // should_fail_fast tests
      evaluate_tests.rs         // collect_and_evaluate tests
      tile_layout_tests.rs      // apply_tile_layout tests
  ```
- Test groups (for sub-splitting):
  - `timeout_tests.rs`: is_timed_out_returns_true_when_timeout_exceeded, is_timed_out_returns_false_when_within_timeout (2 tests, ~40 lines)
  - `drain_logs_tests.rs`: drain_remaining_logs_transfers_buffered_entries_to_captured_logs (1 test, ~55 lines)
  - `guarded_update_tests.rs`: guarded_update_returns_err_on_panic, guarded_update_returns_ok_on_clean_update (2 tests, ~45 lines)
  - `snapshot_tests.rs`: snapshot_eval_data_captures_results_into_shared_buffer (1 test, ~70 lines)
  - `sync_ui_scale_tests.rs`: 7 tests for UI scale behavior (Behaviors 20-26) (~170 lines)
  - `should_fail_fast_tests.rs`: 10 tests for fail-fast logic (Behaviors 5-8 with edges) (~230 lines)
  - `evaluate_tests.rs`: collect_and_evaluate_fails_when_no_snapshot, collect_and_evaluate_passes_with_clean_snapshot, collect_and_evaluate_reports_failure_for_early_exit_snapshot_with_violations (3 tests, ~100 lines)
  - `tile_layout_tests.rs`: 10 tests for apply_tile_layout (Behaviors 16-26) + 3 helpers (~370 lines)
- Shared test helpers to extract into `helpers.rs`:
  - `apply_tile_layout_app()` — builds minimal app with apply_tile_layout system
  - `spawn_primary_monitor()` — spawns Monitor + PrimaryMonitor entity
  - `spawn_primary_window_tracked()` — spawns Window + PrimaryWindow entity
  - Helper for creating `ScenarioDefinition` (used by should_fail_fast and collect_and_evaluate tests)
- Imports needed: each sub-file needs `use super::super::super::app::*;` path or specific items from `crate::runner::app`, `crate::invariants`, `crate::lifecycle`, `crate::log_capture`, `crate::types`, `crate::runner::tiling`
- Re-exports needed: none (test modules have no public API)
- Parent mod.rs (`tests/mod.rs`): change `mod app_tests;` to `mod app_tests;` (no change needed — Rust resolves to `app_tests/mod.rs`)
- Delegate: writer-code can execute this refactor directly

### 2. MEDIUM — `tiling_tests.rs` (562 lines, 31 tests)

Monitor only. At 562 lines this file is over threshold but not urgently large. The tests are well-organized into clear sections (grid_dimensions, tile_position, constants, tile_config_env_vars, TileConfig, parse_tile_config). If it grows past 800 lines from future tiling features, apply Strategy C with groups: `grid_dimensions_tests.rs`, `tile_position_tests.rs`, `tile_config_tests.rs`.

No refactor spec needed at this time.

### 3. MEDIUM — `app.rs` (546 lines, pure production)

Monitor only. At 546 lines of production code, this file contains several cohesive concerns related to scenario app building and running:
- Constants and types (EvalSnapshot, SharedEvalBuffer) — lines 1-48
- Snapshot systems — lines 49-93
- App building (build_app) — lines 95-195
- Scenario running (run_scenario) — lines 197-323
- Evaluation (collect_and_evaluate) — lines 324-404
- Utility functions (should_fail_fast, is_timed_out, drain_remaining_logs, guarded_update) — lines 405-495
- Window systems (sync_ui_scale, apply_tile_layout) — lines 496-546

These concerns are cohesive (all serve scenario execution). Strategy B could split window systems and evaluation into separate files, but the file is only modestly over threshold. If it grows past 700 lines, split into: `build.rs` (app construction), `evaluate.rs` (collect_and_evaluate, should_fail_fast), `window.rs` (sync_ui_scale, apply_tile_layout, tile types).

No refactor spec needed at this time.

---

## Batching for Parallel Execution

Only one file needs immediate splitting, all in `breaker-scenario-runner`:

- **Batch 1**: `app_tests.rs` sub-split (Strategy C) — single crate, no conflicts with any other work
