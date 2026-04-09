---
name: Phase 11 findings -- toughness-hp-scaling
description: Wave 15 scan (2026-04-08 toughness+hp-scaling): 1 MEDIUM (system.rs 743 all-prod), 4 LOW monitors. Detail at docs/todos/detail/2026-04-08-file-splits.md
type: project
---

## Scope

Feature scan: Toughness + HP Scaling changed files.

## MEDIUM (1 file)

- `breaker-game/src/state/run/node/systems/spawn_cells_from_layout/system.rs` (743 lines, all prod, 0 tests) -- Strategy B: extract lock_resolution.rs (~315 lines) from the 5 lock-related functions

## LOW (4 files, monitor only)

- `breaker-game/src/cells/resources/tests.rs` (665 lines, 53 fns) -- already extracted, watch for 800+ threshold
- `breaker-game/src/state/run/node/systems/spawn_cells_from_layout/tests/behaviors.rs` (610 lines, 11 fns) -- already in test dir, watch
- `breaker-game/src/cells/definition/tests.rs` (510 lines, 48 fns) -- already extracted, watch
- `breaker-game/src/state/run/node/systems/spawn_cells_from_layout/tests/helpers.rs` (453 lines, 0 tests, all helpers) -- no split needed

## Batching

Single batch (state/run domain only): system.rs concern separation.
