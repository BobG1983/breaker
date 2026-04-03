---
name: Post-refactor file length findings
description: Wave 7 scan (develop 2026-04-02): all Wave 6 HIGH + mod.rs violations resolved; 2 new HIGH (shield.rs 889, visual_tests.rs 825), 2 MEDIUM actionable (wall/registry.rs, spatial2d/builder.rs), 20+ MEDIUM monitor
type: project
---

Updated 2026-04-02 on develop branch (Wave 7, post refactor/file-splits merge at 30ec4a0).

## Key changes from Wave 6

- All 8 HIGH files from Wave 6 have been split (c72dff8)
- All 6 mod.rs violations from Wave 6 have been fixed (c72dff8)
- shield.rs rewritten with 29 tests, grew to 889 lines (NEW HIGH)
- visual_tests.rs grew from 751 to 825 (crossed HIGH threshold)
- Several MEDIUM test files grew slightly (context_entity_tests 611->730 / 626->722)
- shared/size.rs was split (no longer flagged)

## OPEN HIGH priority (2 files)

| File | Total | Prod | Tests | Test Fns | Strategy |
|------|-------|------|-------|----------|----------|
| `effect/effects/shield.rs` | 889 | 97 | 792 | 29 | A: test extraction + sub-split |
| `bolt/builder/tests/visual_tests.rs` | 825 | 0 | 825 | 26 | C: test sub-split |

## OPEN MEDIUM priority -- actionable (2 files, need test extraction)

| File | Total | Prod | Tests | Test Fns | Strategy |
|------|-------|------|-------|----------|----------|
| `wall/registry.rs` | 548 | 97 | 451 | 23 | A: test extraction |
| `rantzsoft_spatial2d/builder.rs` | 511 | 306 | 205 | 11 | A: test extraction |

## OPEN MEDIUM priority -- monitor only (20 files, already-extracted test files)

| File | Total | Test Fns | Notes |
|------|-------|----------|-------|
| `effect/triggers/impact/tests/context_entity_tests.rs` | 730 | 13 | approaching 800 |
| `effect/triggers/impacted/tests/context_entity_tests.rs` | 722 | 13 | approaching 800 |
| `bolt/systems/reset_bolt/tests.rs` | 691 | 22 | approaching 800 |
| `bolt/builder/tests/build_tests.rs` | 669 | 23 | approaching 800 |
| `effect/effects/piercing_beam/tests/fire_tests/geometry_tests.rs` | 628 | 19 | |
| `debug/hot_reload/systems/propagate_breaker_changes/tests.rs` | 613 | 10 | grew from 516 |
| `rantzsoft_spatial2d/components/tests/velocity_tests/clamp_angle.rs` | 570 | 25 | |
| `bolt/systems/bolt_lost/tests/lost_detection_tests.rs` | 568 | 16 | grew from 500 |
| `bolt/systems/spawn_bolt/tests/migration_tests.rs` | 564 | 12 | grew from 487 |
| `effect/core/types/definitions/enums.rs` | 537 | 5 | 431 prod, unsplittable |
| `bolt/builder/tests/definition_tests/from_definition.rs` | 536 | 17 | |
| `screen/chip_select/systems/generate_chip_offerings/tests.rs` | 512 | 11 | |
| `effect/effects/tether_beam/tests/fire_tests/fire_basic.rs` | 511 | 15 | |
| `effect/triggers/evaluate/tests/on_resolution/resolve_entity_targets.rs` | 507 | 9 | |
| `effect/effects/anchor/tests/tick_timer_tests.rs` | 502 | 14 | |
| `effect/effects/spawn_phantom/tests/fire_tests.rs` | 498 | 13 | |
| `bolt/systems/dispatch_bolt_effects/tests/basic_dispatch.rs` | 496 | 13 | |
| `effect/triggers/evaluate/tests/on_resolution/resolve_edge_cases.rs` | 496 | 13 | |
| `breaker/systems/spawn_breaker/tests/spawn_or_reuse/first_spawn_components.rs` | 479 | 9 | |
| `effect/effects/piercing_beam/tests/process_tests/damage_processing_tests.rs` | 469 | 11 | |

## OPEN LOW priority (400-464 lines)

| File | Total | Notes |
|------|-------|-------|
| `bolt/builder/core/terminal.rs` | 464 | pure prod, unsplittable |
| `breaker/systems/sync_breaker_scale.rs` | 441 | 39 prod, 402 tests |
| `effect/triggers/bump.rs` | 435 | 49 prod, 386 tests |
| `breaker/systems/breaker_wall_collision.rs` | 434 | 68 prod, 366 tests |
| `cells/resources.rs` | 430 | 122 prod, 308 tests |
| `effect/core/types/tests.rs` | 426 | already extracted |
| `bolt/registry.rs` | 415 | 95 prod, 320 tests |
| `bolt/systems/sync_bolt_scale.rs` | 407 | 30 prod, 377 tests |
| `breaker/definition.rs` | 406 | 284 prod, 122 tests |
| `bolt/systems/launch_bolt.rs` | 405 | 48 prod, 357 tests |

## Files confirmed unsplittable

- rantzsoft_physics2d/src/quadtree/tree.rs (487) -- single data structure, no tests
- rantzsoft_spatial2d/src/components/definitions.rs (472) -- pure type definitions
- breaker-scenario-runner/src/runner/execution.rs (493) -- single concern, no tests
- breaker-scenario-runner/src/runner/app.rs (447) -- single concern, tiny test section
- effect/core/types/definitions/enums.rs (537) -- tightly coupled enum definitions
- bolt/builder/core/terminal.rs (464) -- pure production typestate terminal methods

## No mod.rs violations

All previously flagged mod.rs violations resolved. No new violations detected.
