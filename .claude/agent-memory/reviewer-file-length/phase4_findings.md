---
name: Post-refactor file length findings
description: Wave 8 scan (2026-04-02 refactor/state-folder-structure): 0 HIGH (all resolved), 0 MEDIUM actionable, 20 MEDIUM monitor, 12 LOW. State restructure clean -- no new splits needed.
type: project
---

Updated 2026-04-02 on refactor/state-folder-structure branch (Wave 8, post state/ hierarchy restructure).

## Key changes from Wave 7

- shield.rs (889) split into effect/effects/shield/ directory (system.rs + tests/) -- resolved
- visual_tests.rs (825) split into bolt/builder/tests/visual_tests/ directory -- resolved
- walls/registry (548) split into walls/registry/ directory (core.rs + tests.rs) -- resolved
- rantzsoft_spatial2d/builder (511) split into builder/ directory (core.rs + tests.rs) -- resolved
- State folder restructure created 58 new mod.rs files, all wiring-only, no violations
- resolve_entity_targets.rs grew 507->546, resolve_edge_cases.rs grew 496->517
- generate_chip_offerings/tests.rs path changed from screen/ to state/run/chip_select/
- reset_bolt/tests.rs path changed from bolt/systems/ to state/run/node/systems/

## OPEN HIGH priority -- NONE

All previously HIGH files have been resolved.

## OPEN MEDIUM priority -- actionable -- NONE

All previously MEDIUM actionable files have been split.

## OPEN MEDIUM priority -- monitor only (20 files, already-extracted test files)

| File | Total | Test Fns | Notes |
|------|-------|----------|-------|
| `effect/triggers/impact/tests/context_entity_tests.rs` | 731 | 13 | approaching 800 |
| `effect/triggers/impacted/tests/context_entity_tests.rs` | 723 | 13 | approaching 800 |
| `state/run/node/systems/reset_bolt/tests.rs` | 692 | 22 | approaching 800 |
| `bolt/builder/tests/build_tests.rs` | 670 | 23 | approaching 800 |
| `effect/effects/piercing_beam/tests/fire_tests/geometry_tests.rs` | 629 | 19 | |
| `debug/hot_reload/systems/propagate_breaker_changes/tests.rs` | 614 | 10 | |
| `rantzsoft_spatial2d/components/tests/velocity_tests/clamp_angle.rs` | 571 | 25 | |
| `bolt/systems/bolt_lost/tests/lost_detection_tests.rs` | 569 | 16 | |
| `bolt/systems/spawn_bolt/tests/migration_tests.rs` | 565 | 12 | |
| `effect/core/types/definitions/enums.rs` | 541 | 5 | 431 prod, unsplittable |
| `bolt/builder/tests/definition_tests/from_definition.rs` | 537 | 17 | |
| `effect/triggers/evaluate/tests/on_resolution/resolve_entity_targets.rs` | 546 | 9 | grew from 507 |
| `effect/triggers/evaluate/tests/on_resolution/resolve_edge_cases.rs` | 517 | 13 | grew from 496 |
| `state/run/chip_select/systems/generate_chip_offerings/tests.rs` | 513 | 11 | path updated |
| `effect/effects/tether_beam/tests/fire_tests/fire_basic.rs` | 512 | 15 | |
| `effect/effects/anchor/tests/tick_timer_tests.rs` | 503 | 14 | |
| `effect/effects/spawn_phantom/tests/fire_tests.rs` | 499 | 13 | |
| `bolt/systems/dispatch_bolt_effects/tests/basic_dispatch.rs` | 497 | 13 | |
| `breaker/systems/spawn_breaker/tests/spawn_or_reuse/first_spawn_components.rs` | 480 | 9 | |
| `effect/effects/piercing_beam/tests/process_tests/damage_processing_tests.rs` | 470 | 11 | |

## OPEN LOW priority (400-465 lines)

| File | Total | Notes |
|------|-------|-------|
| `bolt/builder/core/terminal.rs` | 465 | pure prod, unsplittable |
| `breaker/systems/sync_breaker_scale.rs` | 442 | 39 prod, 402 tests |
| `effect/triggers/bump.rs` | 436 | 49 prod, 386 tests |
| `breaker/systems/breaker_wall_collision.rs` | 435 | 68 prod, 366 tests |
| `cells/resources.rs` | 431 | 122 prod, 308 tests |
| `walls/builder/tests/build_tests.rs` | 429 | already extracted |
| `effect/core/types/tests.rs` | 427 | already extracted |
| `bolt/registry.rs` | 416 | 95 prod, 320 tests |
| `effect/effects/shield/tests/fire_tests.rs` | 409 | newly extracted |
| `bolt/systems/sync_bolt_scale.rs` | 408 | 30 prod, 377 tests |
| `breaker/definition.rs` | 407 | 284 prod, 122 tests |
| `bolt/systems/launch_bolt.rs` | 406 | 48 prod, 357 tests |

## Files confirmed unsplittable

- rantzsoft_physics2d/src/quadtree/tree.rs (488) -- single data structure, no tests
- rantzsoft_spatial2d/src/components/definitions.rs (473) -- pure type definitions
- breaker-scenario-runner/src/runner/execution.rs (494) -- single concern, no tests
- breaker-scenario-runner/src/runner/app.rs (448) -- single concern, tiny test section
- effect/core/types/definitions/enums.rs (541) -- tightly coupled enum definitions
- bolt/builder/core/terminal.rs (465) -- pure production typestate terminal methods

## mod.rs violations

Minor only: `state/run/chip_select/mod.rs` has a 4-line `color_from_rgb` const helper. Not actionable.

All 58 new state/ hierarchy mod.rs files are wiring-only. No violations detected.
