# File Length Review: Toughness + HP Scaling Feature

Wave 15 scan (2026-04-08). Scope: files changed in Toughness + HP Scaling feature.

## Files Over Threshold

| File | Total | Prod | Tests | Test Fns | Strategy | Priority |
|------|-------|------|-------|----------|----------|----------|
| `breaker-game/src/state/run/node/systems/spawn_cells_from_layout/system.rs` | 743 | 743 | 0 | 0 | B: concern separation | MEDIUM |
| `breaker-game/src/cells/resources/tests.rs` | 665 | 0 | 665 | 53 | monitor (under 800) | LOW |
| `breaker-game/src/state/run/node/systems/spawn_cells_from_layout/tests/behaviors.rs` | 610 | 0 | 610 | 11 | monitor (under 800) | LOW |
| `breaker-game/src/cells/definition/tests.rs` | 510 | 0 | 510 | 48 | monitor (under 800) | LOW |
| `breaker-game/src/state/run/node/systems/spawn_cells_from_layout/tests/helpers.rs` | 453 | 0 | 453 | 0 (helpers only) | monitor (helpers, no split) | LOW |

### Files Under Threshold (no action)

| File | Total |
|------|-------|
| `breaker-game/src/state/run/systems/advance_node.rs` | 374 |
| `breaker-game/src/state/run/resources/definitions.rs` | 347 |

## Refactor Specs

### MEDIUM: `spawn_cells_from_layout/system.rs` (743 lines, all production)

This file has grown from 590 (phase 10) to 743 lines due to Toughness + HP scaling additions. It contains 5 distinct concerns:

1. **Grid computation** (lines 22-93, ~72 lines): `grid_extent`, `ScaledGridDims`, `compute_grid_scale`
2. **Types/context structs** (lines 96-177, ~82 lines): `RenderAssets`, `GridSpawnParams`, `HpContext`, `ToughnessHpData`, `GridCellContext`, `HpScale` + impls
3. **Core spawn logic** (lines 179-363, ~185 lines): `GridCellContext::spawn_pass1`, `spawn_cells_from_grid`
4. **Lock resolution** (lines 365-679, ~315 lines): `resolve_and_spawn_locks`, `spawn_locked_cell`, `spawn_unlocked_fallback`, `topological_sort_locks`
5. **Guardian helpers** (lines 530-618, ~89 lines): `build_guardian_skip_set`, `collect_guardian_slots`
6. **HP resolution + system entry** (lines 681-743, ~63 lines): `resolve_hp_context`, `CellSpawnContext`, `spawn_cells_from_layout`

**Recommended split**: Extract lock resolution (the largest concern at ~315 lines) into its own file. This keeps the main spawn logic clean and groups the topological sort with the lock spawning code that depends on it.

**Refactor spec hint:**
- Source file: `breaker-game/src/state/run/node/systems/spawn_cells_from_layout/system.rs`
- Total lines: 743 (prod: 743, tests: 0)
- Strategy: B (concern separation)
- Target structure:
  ```
  spawn_cells_from_layout/
    mod.rs              // pub(crate) mod system; pub(crate) mod lock_resolution; #[cfg(test)] mod tests;
                        // + re-exports for public API (unchanged from external perspective)
    system.rs           // grid computation, types, core spawn logic, guardian helpers, HP resolution, system entry (~428 lines)
    lock_resolution.rs  // resolve_and_spawn_locks, spawn_locked_cell, spawn_unlocked_fallback, topological_sort_locks (~315 lines)
    tests/              // unchanged
  ```
- Functions moving to `lock_resolution.rs`:
  - `resolve_and_spawn_locks` (line 367)
  - `spawn_locked_cell` (line 415)
  - `spawn_unlocked_fallback` (line 487)
  - `topological_sort_locks` (line 620)
- Functions staying in `system.rs`: everything else (grid_extent, compute_grid_scale, RenderAssets, GridSpawnParams, HpContext, ToughnessHpData, GridCellContext, HpScale, spawn_pass1, spawn_cells_from_grid, build_guardian_skip_set, collect_guardian_slots, resolve_hp_context, CellSpawnContext, spawn_cells_from_layout)
- Imports needed in `lock_resolution.rs`:
  - `use super::system::{GridCellContext, GridCoord};` (GridCellContext is private, needs pub(crate) or pub(super))
  - `use std::collections::{HashMap, HashSet, VecDeque};`
  - `use bevy::prelude::*;`
  - `use crate::cells::builder::core::types::...` (Cell builder)
  - `use crate::state::run::node::definition::LockMap;`
- Visibility changes needed:
  - `GridCellContext` — currently private, needs `pub(super)` for lock_resolution to use
  - `GridSpawnParams` — currently private, needs `pub(super)` if lock_resolution computes positions
  - `GridCellContext::compute_hp` — currently private, needs `pub(super)`
- Re-exports in mod.rs: same as current (spawn_cells_from_layout, spawn_cells_from_grid, compute_grid_scale, grid_extent, RenderAssets, HpContext, ToughnessHpData, ScaledGridDims, CellSpawnContext)
- Parent module (`spawn_cells_from_layout/mod.rs`): needs `pub(crate) mod lock_resolution;` added
- External imports: unchanged — re-exports maintain the same public API
- Delegate: writer-code can execute this refactor directly

### Note on monitored LOW-priority files

- `cells/resources/tests.rs` (665 lines, 53 tests): Already extracted. Will need Strategy C (sub-split into test directory) when it crosses 800. Three natural groups visible: CellConfig tests (~100 lines), SeedableRegistry/CellTypeRegistry tests (~260 lines), ToughnessConfig tests (~305 lines).
- `spawn_cells_from_layout/tests/behaviors.rs` (610 lines, 11 tests): Already in test directory. Watch for growth.
- `cells/definition/tests.rs` (510 lines, 48 tests): Already extracted. Nine section headers visible. Groups by: toughness enum, definition validation, guarded behavior, slide_speed, delegation, deserialization, alias validation, regen rate.
- `spawn_cells_from_layout/tests/helpers.rs` (453 lines, 0 tests): Pure test helpers. No split needed — helper functions should stay together for discoverability.
