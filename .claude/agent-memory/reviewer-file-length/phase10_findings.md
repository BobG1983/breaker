---
name: Phase 10 findings -- cell-builder-pattern
description: Wave 14 scan (2026-04-08 feature/bolt-birthing-animation): 2 HIGH (optional_tests.rs 918, lock_resolution.rs 852), 3 MEDIUM (definition.rs 764, system.rs 590, slide_guardian_cells.rs 512), 2 LOW. Spec at .claude/specs/file-splits.md
type: project
---

## Scope

Branch scan: cell builder pattern files on feature/bolt-birthing-animation.

## HIGH (2 files)

- `breaker-game/src/cells/builder/tests/optional_tests.rs` (918 lines, pure test, 32 fns) -- Strategy C: sub-split into chainable_methods, locked, guarded_builder, guarded_behavior
- `breaker-game/src/state/run/node/systems/spawn_cells_from_layout/tests/lock_resolution.rs` (852 lines, pure test, 24 fns) -- Strategy C: sub-split into basic_locking, chain_resolution, cycle_handling, lock_properties, edge_cases

## MEDIUM (3 files)

- `breaker-game/src/cells/definition.rs` (764 lines, 134 prod, 630 test, 51 fns) -- Strategy A: test extraction
- `breaker-game/src/state/run/node/systems/spawn_cells_from_layout/system.rs` (590 lines, all prod) -- Strategy B: optional lock_resolution extraction
- `breaker-game/src/cells/behaviors/guarded/systems/slide_guardian_cells.rs` (512 lines, 108 prod, 404 test, 8 fns) -- Strategy A: test extraction

## LOW (2 files, monitor only)

- `breaker-game/src/cells/builder/tests/spawn_tests.rs` (675 lines, pure test, 23 fns) -- under 800, watch
- `breaker-game/src/cells/builder/tests/definition_tests.rs` (551 lines, pure test, 21 fns) -- under 800, watch

## Batching

Batch 1 (cells domain): optional_tests.rs, definition.rs, slide_guardian_cells.rs
Batch 2 (state/run domain): lock_resolution.rs, system.rs (optional)
Batches can run in parallel.
