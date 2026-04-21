# File Splits: protocol-burnout Full Verification Tier re-scan

Scan date: 2026-04-20 (second run)
Branch: `feature/protocol-burnout` (pre-merge to develop)
Scope:
1. Re-verify all in-branch protocol modules authored in this branch are within limits.
2. Workspace-wide scan for any `.rs` file that has grown past 400 lines and is NOT already tracked by an active split todo.
3. Compare current state against the previous scan (2026-04-20 first run, detail `docs/todos/detail/2026-04-20-file-splits.md`).

## Summary

The 4 HIGH hazard monoliths surfaced by the previous 2026-04-20 scan (volatility / echo_cells / fracture / cascade) and the 2 HIGH heal-pipeline items from the 2026-04-17 scan (`apply_heal.rs`, `death_pipeline/plugin.rs`) have ALL been resolved — every one is now a properly structured directory module with sub-split tests. The corresponding todo entries are stale and can be deleted.

This re-scan turned up **1 new HIGH** and **a small cluster of MEDIUM** items, none currently tracked.

All 8 custom-system protocol modules (Burnout, Reckless Dash, Echo Strike, Fission, Iron Curtain, Debt Collector, Siphon, Greed) are correctly structured as directory modules with per-behavior-group test sub-files. No in-branch file requires a split before this merge.

## In-branch protocol module result (Burnout — authored this branch)

All files within limits. Two sub-files sit just above 400 (awareness only):

| File | Lines | Notes |
|------|-------|-------|
| `protocol/protocols/burnout/mod.rs` | 11 | wiring only |
| `protocol/protocols/burnout/system.rs` | 395 | under threshold |
| `protocol/protocols/burnout/tests/update_heat.rs` | 455 | **LOW** — awareness |
| `protocol/protocols/burnout/tests/register.rs` | 436 | **LOW** — awareness |

No action required for the merge.

## File Length Review — new items (not previously tracked)

### Files Over Threshold

| File | Total | Prod | Tests | Test Fns | Strategy | Priority |
|------|-------|------|-------|----------|----------|----------|
| `breaker-scenario-runner/src/coverage.rs` | 834 | 149 | 685 | 28 | A: test extraction (no sub-split — under 800) | **HIGH** |
| `breaker-game/src/cells/systems/apply_damage_to_cells/tests/diffusion.rs` | 827 | 0 | 827 | ~30 | C: already extracted, sub-split by behavior band | **MEDIUM** |
| `breaker-scenario-runner/src/main.rs` | 661 | 403 | 258 | 24 | A: test extraction | **MEDIUM** |
| `breaker-game/src/hazard/hazards/diffusion.rs` | 565 | 114 | 451 | 30 | A: test extraction + sub-split by behavior group | **MEDIUM** |
| `breaker-game/src/state/run/chip_select/systems/spawn_chip_select.rs` | 621 | 218 | 403 | — | A: test extraction | **MEDIUM** |
| `breaker-game/src/hazard/hazards/resonance/system.rs` | 429 | 429 | 0 | 0 | — single-concern production, no split action | **LOW** |
| `breaker-game/src/hazard/definition.rs` | 422 | 210 | 212 | — | A: test extraction (borderline) | **LOW** |
| `breaker-game/src/protocol/definition.rs` | 439 | 186 | 253 | — | A: test extraction (borderline) | **LOW** |

All paths are relative to the repo root unless otherwise noted.

### Priority Guide
- **HIGH**: 1000+ lines, or 800+ test lines (biggest context pollution impact, split immediately)
- **MEDIUM**: 501-999 lines (noticeable, split at least once)
- **LOW**: 400-500 lines (flag for awareness, will need splitting soon)

### Additional LOW / MEDIUM awareness (already sub-split once; no split needed until they cross 800)
These are per-behavior-group test files already inside a `tests/` directory and below 800. Monitor; split only if they grow.

| File | Lines |
|------|-------|
| `protocol/protocols/tier_regression/tests/apply.rs` | 793 |
| `protocol/protocols/afterimage/tests/check_phantom_bounce.rs` | 774 |
| `cells/systems/apply_damage_to_cells/tests/tether_redirect.rs` | 762 |
| `protocol/protocols/echo_strike/tests/on_impact.rs` | 708 |
| `state/run/node/systems/reset_bolt/tests.rs` | 667 |
| `state/run/chip_select/systems/handle_chip_input/tests.rs` | 792 |
| `state/run/chip_select/systems/generate_chip_offerings/tests.rs` | 786 |
| `state/run/chip_select/systems/snapshot_node_highlights/tests.rs` | 654 |
| `state/run/node/systems/spawn_cells_from_layout/tests/behaviors.rs` | 637 |
| `effect_v3/effects/pulse/config/tests.rs` | 643 |
| `effect_v3/conditions/evaluate_conditions/tests/during_basic.rs` | 673 |
| `effect_v3/conditions/evaluate_conditions/tests/shape_c.rs` | 617 |
| `effect_v3/triggers/bump/bridges/tests/occurred.rs` | 616 |
| `hazard/hazards/momentum/tests/split_check.rs` | 615 |
| `shared/death_pipeline/systems/tests/handle_kill_unit.rs` | 683 |
| `shared/size/tests.rs` | 648 |

No action needed on these for the merge. They are already well-bounded per-concern test files.

---

## Refactor Specs (HIGH + actionable MEDIUM)

### 1. `breaker-scenario-runner/src/coverage.rs` (834 lines) — HIGH

**Refactor spec hint:**
- Source file: `breaker-scenario-runner/src/coverage.rs`
- Total lines: 834 (prod: 149, tests: 685 — `#[cfg(test)]` at line 150)
- Strategy: A (test extraction only; tests are at 685 lines — no need to sub-split yet)
- Parent module: `breaker-scenario-runner/src/lib.rs` declares `pub mod coverage;` — unchanged
- Doc comment to preserve on new `coverage/mod.rs`:
  ```
  //! Coverage parity checking for invariant self-tests and layout usage.
  //!
  //! Pure analysis module — no Bevy, no ECS. Checks that every
  //! [`InvariantKind`] variant has at least one self-test scenario and that
  //! every layout RON file is referenced by at least one scenario.
  ```
- Target structure:
  ```
  breaker-scenario-runner/src/coverage/
    mod.rs      // doc comment above
                // pub mod system; #[cfg(test)] mod tests;
                // pub use system::{check_coverage, CoverageReport, format_report, print_coverage_report, normalize_layout_name};
    system.rs   // everything before line 150 — pure production (CoverageReport struct,
                //   check_coverage, normalize_layout_name, format_report, print_coverage_report)
    tests.rs    // everything from line 150 onwards (685 lines, 28 tests).
                //   Change `use super::*;` → `use super::system::*;`
                //   Preserve the `minimal_scenario` test helper.
  ```
- Imports needed in `tests.rs`: `use super::system::*;` plus the current test-module-internal imports (InvariantKind, ScenarioDefinition).
- Re-exports needed in `mod.rs`: re-export whatever `lib.rs` / other `coverage::*` callers use. Grep `coverage::` in `breaker-scenario-runner/src/**` to confirm.
- Delegate: writer-code can execute directly.

---

### 2. `breaker-game/src/cells/systems/apply_damage_to_cells/tests/diffusion.rs` (827 lines) — MEDIUM

**Refactor spec hint:**
- Source file: `breaker-game/src/cells/systems/apply_damage_to_cells/tests/diffusion.rs`
- Total lines: 827 (test-only — the file lives under `tests/`)
- Strategy: C (convert single test file into a test sub-directory, sub-split by behavior band)
- Parent module: `cells/systems/apply_damage_to_cells/tests/mod.rs` declares `mod diffusion;` — unchanged
- Target structure:
  ```
  cells/systems/apply_damage_to_cells/tests/diffusion/
    mod.rs              // mod helpers; mod share_math; mod depth_math;
                        //   mod bfs_branch; mod integration;
                        // Plus the original `//! Section D — ...` doc comment.
    helpers.rs          // `PendingCellDamage`, `enqueue_cell_damage`, app-construction helpers
                        // — all `pub(super)` visibility
    share_math.rs       // `share_percent_*` tests (math-only, no ECS)
    depth_math.rs       // `depth_*` tests (math-only, no ECS)
    bfs_branch.rs       // Core Diffusion redistribution BFS tests (ECS-driven)
    integration.rs      // Remaining cross-cutting integration tests (register, activate interplay)
  ```
- Writer-code should grep `// Section` markers and test-function prefixes (`fn share_*`, `fn depth_*`, `fn diffusion_*`) to confirm bands.
- Imports per sub-file: `use super::super::*;` (one extra `super::` vs the original, since tests moved one directory deeper). Keep the existing use statements block.
- Delegate: writer-code can execute directly.

---

### 3. `breaker-scenario-runner/src/main.rs` (661 lines) — MEDIUM

**Refactor spec hint:**
- Source file: `breaker-scenario-runner/src/main.rs`
- Total lines: 661 (prod: 403, tests: 258 — `#[cfg(test)]` at line 404)
- Strategy: A (test extraction into a sibling module). `main.rs` is the binary entry point — promoting to `main/` would force restructuring. The simpler path is to extract just the tests.
- Approach: extract the inline `mod tests { ... }` into an existing module file (or a new peer module) accessible from main. Two options:
  1. Extract the unit-testable helper functions (arg parsing, etc.) from `main.rs` into a new `breaker-scenario-runner/src/cli.rs` module, then move the `mod tests` block to `cli.rs` (or `cli/tests.rs`).
  2. If option 1 is too invasive, leave `main.rs` as-is and accept it as a LOW priority (since tests live inside the binary crate and cannot be imported externally).
- Preferred: option 1. Grep the 24 tests for what they exercise — typically `parse_parallelism`, `parse_args`, etc. Move those helpers plus the tests to `src/cli.rs` + `src/cli/tests.rs` (or inline tests, since the new file will be ~500 lines).
- Delegate: writer-code can execute. If investigation shows main.rs truly has no extractable helpers (the 24 tests all test `main()` directly), leave it as LOW priority.

---

### 4. `breaker-game/src/hazard/hazards/diffusion.rs` (565 lines) — MEDIUM

**Refactor spec hint:**
- Source file: `breaker-game/src/hazard/hazards/diffusion.rs`
- Total lines: 565 (prod: 114, tests: 451 — `#[cfg(test)]` at line 115)
- Strategy: A (promote to `diffusion/` directory). Tests are 451 lines and fall cleanly into four behavior groups — optional sub-split (can keep a single `tests.rs` since it's under 800, but sub-splitting now future-proofs the module and matches the `volatility/` / `echo_cells/` / `fracture/` pattern applied to the other hazards).
- Parent module: `hazard/hazards/mod.rs` declares `mod diffusion;` — unchanged
- Doc comment to preserve:
  ```
  //! Diffusion hazard — stateless damage-sharing redistribution.
  //!
  //! Per the design doc at `docs/todos/detail/mod-system-design/hazards/diffusion.md`,
  //! redistribution lives INSIDE the cells-domain system `apply_damage_to_cells`.
  //! The hazard domain owns only the per-run `DiffusionConfig` resource populated
  //! by [`activate`] and a no-op [`register`] — there is no hazard-domain runtime
  //! system for the BFS / HP math.
  ```
- Target structure (keeps future room; matches sibling hazard layout):
  ```
  hazard/hazards/diffusion/
    mod.rs      // doc comment above
                // pub(crate) mod system;
                // #[cfg(test)] mod tests;
                // Plus re-exports: `pub(crate) use system::{DiffusionConfig, activate, register, DIFFUSION_SHARE_CAP_PERCENT};`
    system.rs   // everything before line 115 (DiffusionConfig + DIFFUSION_SHARE_CAP_PERCENT
                //   + activate + register). All 4 function definitions.
    tests/
      mod.rs              // mod helpers; mod share_math; mod depth_math;
                          //   mod activate_tests; mod register_tests;
      helpers.rs          // test_app_playing, add_diffusion_stacks, activate_now — `pub(super)`
      share_math.rs       // All `share_percent_*` tests (13 tests) — lines ~168-285
      depth_math.rs       // All `depth_*` tests (8 tests) — lines ~286-360
      activate_tests.rs   // `activate_*` tests (5 tests) — lines ~361-500
      register_tests.rs   // `register_*` tests (3 tests) — lines ~501-565
  ```
- Imports per test sub-file: `use super::super::system::*;` plus `use super::helpers::*;` and existing imports like `use crate::hazard::definition::*;`.
- Delegate: writer-code can execute directly.

---

### 5. `breaker-game/src/state/run/chip_select/systems/spawn_chip_select.rs` (621 lines) — MEDIUM

**Refactor spec hint:**
- Source file: `breaker-game/src/state/run/chip_select/systems/spawn_chip_select.rs`
- Total lines: 621 (prod: 218, tests: 403 — `#[cfg(test)]` at line 219)
- Strategy: A (test extraction)
- Parent module: `state/run/chip_select/systems/mod.rs` declares `mod spawn_chip_select;` — unchanged
- Doc comment: copy the `//! System to spawn the chip selection screen UI.` to the new `mod.rs`.
- Target structure:
  ```
  state/run/chip_select/systems/spawn_chip_select/
    mod.rs      // doc comment above
                // pub(crate) mod system;
                // #[cfg(test)] mod tests;
                // Plus re-exports for the public system fn + any public helpers.
    system.rs   // everything before line 219 (the system fn, UI-spawning helpers)
    tests.rs    // everything from line 219 onwards (403 lines).
                //   Change `use super::*;` → `use super::system::*;`
  ```
- Tests are at 403 lines — under 800, a flat `tests.rs` is sufficient. Sub-split only if it grows.
- Delegate: writer-code can execute directly.

---

## Borderline LOW items — no action this merge

The three `definition.rs` files below are just past 400; they're in the awareness band. No split needed unless they grow past ~500.

- `breaker-game/src/hazard/definition.rs` (422 lines, 210 prod + 212 tests) — currently OK
- `breaker-game/src/protocol/definition.rs` (439 lines, 186 prod + 253 tests) — currently OK
- `breaker-game/src/hazard/hazards/resonance/system.rs` (429 lines, pure prod, no tests) — the matching `tests/` directory already holds the tests; this is a single-concern production file. No action.

---

## Recommended batching for parallel writer-code agents

Group 1 — `breaker-scenario-runner` crate (serialize runner-tests naturally):
- `coverage.rs` (834 HIGH)
- `main.rs` (661 MEDIUM — conditional on helpers being extractable)

Group 2 — `breaker-game` crate, independent subtrees (can batch in parallel):
- `cells/systems/apply_damage_to_cells/tests/diffusion.rs` (827 MEDIUM — Strategy C)
- `hazard/hazards/diffusion.rs` (565 MEDIUM — Strategy A)
- `state/run/chip_select/systems/spawn_chip_select.rs` (621 MEDIUM — Strategy A)

No cross-group dependencies. Within Group 2, the three items live in disjoint subdirectories and can split in parallel. After each wave, run `cargo all-dclippy` + `cargo all-dtest` (Basic Verification Tier).

## Hygiene — stale todos

The current `docs/todos/TODO.md` top two entries are stale:
1. "Split oversized files (4 HIGH) — hazard monoliths" → all 4 items already split (volatility, echo_cells, fracture, cascade).
2. "Split oversized files (2 HIGH) — heal-pipeline follow-up" → both items already split (apply_heal, death_pipeline/plugin).

Consider removing both entries (plus their detail files) when adding the new top entry for this scan.
