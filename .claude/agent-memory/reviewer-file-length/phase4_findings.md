---
name: Post-refactor file length findings
description: Wave 6 scan (feature/breaker-builder-pattern 2026-04-02): 8 HIGH + 6 mod.rs violations + 12 MEDIUM open; dispatch_initial_effects_tests grew to 1571
type: project
---

Updated 2026-04-02 on feature/breaker-builder-pattern branch (Wave 6).

## Key changes from Wave 5 (same branch, earlier same day)

- dispatch_initial_effects_tests.rs grew from 1420 to 1571 (32 tests, +151 lines)
- breaker/queries.rs stable at 1440 but test count refined to 35
- spawn_or_reuse.rs crossed from MEDIUM (779) to HIGH (806, 26 tests)
- impact/context_entity_tests grew 548->611, impacted/context_entity_tests grew 548->626
- resolve_entity_targets grew 462->490
- bolt/builder/core.rs shrunk slightly 742->782 (still HIGH)
- breaker/builder/core.rs shrunk slightly 829->830 (stable HIGH)
- All 6 mod.rs violations still open

## All previously HIGH -- RESOLVED

All files from waves 1-4 have been split.

## OPEN HIGH priority (8 files)

| File | Total | Prod | Tests | Test Fns | Strategy |
|------|-------|------|-------|----------|----------|
| `effect/commands/tests/dispatch_initial_effects_tests.rs` | 1571 | 0 | 1571 | 32 | C: sub-split (8 files) |
| `breaker/queries.rs` | 1440 | 245 | 1195 | 35 | A: test extraction + sub-split (9 test files) |
| `bolt/builder/tests/definition_tests.rs` | 1329 | 0 | 1329 | 48 | C: sub-split (4 files + helpers) |
| `debug/hot_reload/systems/propagate_bolt_definition/tests.rs` | 1035 | 0 | 1035 | 20 | C: sub-split (4 files + helpers) |
| `rantzsoft_spatial2d/components/tests/velocity_tests.rs` | 966 | 0 | 966 | 72 | C: sub-split (8 files) |
| `breaker/builder/core.rs` | 830 | 830 | 0 | 0 | B: types + transitions + terminal |
| `breaker/systems/spawn_breaker/tests/spawn_or_reuse.rs` | 806 | 0 | 806 | 26 | C: sub-split (3 files) |
| `bolt/builder/core.rs` | 782 | 782 | 0 | 0 | B: types + transitions + terminal |

## OPEN mod.rs violations (4 MEDIUM + 2 LOW)

| File | Lines | Priority |
|------|-------|----------|
| `effect/triggers/evaluate/tests/bound_and_staged/mod.rs` | 142 | MEDIUM |
| `effect/effects/attraction/tests/mod.rs` | 114 | MEDIUM |
| `effect/effects/shockwave/tests/mod.rs` | 96 | MEDIUM |
| `effect/effects/pulse/tests/mod.rs` | 85 | MEDIUM |
| `effect/effects/piercing_beam/tests/mod.rs` | 76 | LOW |
| `effect/triggers/until/tests/mod.rs` | 25 | LOW |

## OPEN MEDIUM priority (12 files, 500-751 lines)

| File | Total | Notes |
|------|-------|-------|
| `bolt/builder/tests/visual_tests.rs` | 751 | 26 tests |
| `shared/size.rs` | 664 | 98 prod, 565 tests |
| `effect/triggers/impacted/tests/context_entity_tests.rs` | 626 | 13 tests, grew from 548 |
| `bolt/builder/tests/build_tests.rs` | 620 | 23 tests |
| `effect/triggers/impact/tests/context_entity_tests.rs` | 611 | 13 tests, grew from 548 |
| `bolt/systems/reset_bolt/tests.rs` | 601 | 22 tests |
| `effect/effects/piercing_beam/tests/fire_tests/geometry_tests.rs` | 527 | 19 tests |
| `debug/hot_reload/systems/propagate_breaker_changes/tests.rs` | 516 | 10 tests |
| `effect/core/types/definitions/enums.rs` | 503 | 432 prod, 71 tests |
| `bolt/systems/bolt_lost/tests/lost_detection_tests.rs` | 500 | 16 tests |
| `effect/triggers/evaluate/tests/on_resolution/resolve_entity_targets.rs` | 490 | 13 tests, grew from 462 |
| `bolt/systems/spawn_bolt/tests/migration_tests.rs` | 487 | 12 tests |

## OPEN LOW priority

| File | Total | Notes |
|------|-------|-------|
| `screen/chip_select/systems/generate_chip_offerings/tests.rs` | 461 | 11 tests |
| `effect/triggers/evaluate/tests/on_resolution/resolve_edge_cases.rs` | 453 | 9 tests |
| `bolt/systems/dispatch_bolt_effects/tests/basic_dispatch.rs` | 452 | 13 tests |
| `rantzsoft_spatial2d/builder.rs` | 451 | 295 prod, 156 tests |
| `effect/effects/anchor/tests/tick_timer_tests.rs` | 440 | 14 tests |
| `effect/effects/tether_beam/tests/fire_tests/fire_basic.rs` | 435 | 15 tests |
| `effect/effects/spawn_phantom/tests/fire_tests.rs` | 431 | 13 tests |

## Files confirmed unsplittable

- rantzsoft_physics2d/src/quadtree/tree.rs (451) -- single data structure, no tests
- rantzsoft_spatial2d/src/components/definitions.rs (402) -- pure type definitions
- breaker-scenario-runner/src/runner/execution.rs (439) -- single concern, no tests
- breaker-scenario-runner/src/runner/app.rs (402) -- single concern, tiny test section
