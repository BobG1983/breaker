---
name: Post-refactor file length findings
description: Wave 5 scan (feature/breaker-builder-pattern 2026-04-02): 6 HIGH (queries.rs, 4 test files, builder/core.rs) + 4 mod.rs violations + 12 MEDIUM open
type: project
---

Updated 2026-04-02 on feature/breaker-builder-pattern branch.

## Key changes from Wave 4 (2026-04-01 on feature/chip-evolution-ecosystem)

- All 4 previously HIGH files SPLIT: bolt/builder.rs, anchor/tests.rs, tether_beam/fire_tests.rs, dash/flash_step.rs
- Both previous mod.rs violations FIXED
- NEW HIGH files from breaker-builder-pattern: breaker/queries.rs (1440), breaker/builder/core.rs (829)
- bolt/builder.rs (2308) now split but sub-files grew: definition_tests.rs (1329), visual_tests.rs (751), build_tests.rs (620)
- Previous MEDIUM files: dispatch_chip_effects tests split (e2e, edge_cases), circuit_breaker/tests split, anchor_multipliers split, move_breaker split, piercing_beam split, bolt_wall_collision split, transfer_insert_tests split, template_tests split, spawn_bolt/tests split, aabb_tests split, steering_tests split
- NEW: dispatch_initial_effects_tests.rs (1420), propagate_bolt_definition/tests.rs (1035), velocity_tests.rs (966)
- NEW mod.rs violations: attraction/tests, shockwave/tests, pulse/tests, bound_and_staged, piercing_beam/tests, until/tests

## All previously HIGH -- RESOLVED

All files from waves 1-4 have been split.

## OPEN HIGH priority (6 files)

| File | Total | Prod | Tests | Test Fns | Strategy |
|------|-------|------|-------|----------|----------|
| `breaker/queries.rs` | 1440 | 245 | 1195 | 33 | A: test extraction + sub-split (8 test files) |
| `effect/commands/tests/dispatch_initial_effects_tests.rs` | 1420 | 0 | 1420 | 32 | C: sub-split by behavior (6 files) |
| `bolt/builder/tests/definition_tests.rs` | 1329 | 0 | 1329 | 48 | C: sub-split (4 files + helpers) |
| `debug/hot_reload/systems/propagate_bolt_definition/tests.rs` | 1035 | 0 | 1035 | 20 | C: sub-split (5 files + helpers) |
| `rantzsoft_spatial2d/components/tests/velocity_tests.rs` | 966 | 0 | 966 | 72 | C: sub-split (8 files) |
| `breaker/builder/core.rs` | 829 | 829 | 0 | 0 | B: types + methods + terminal |

## OPEN mod.rs violations (4 MEDIUM + 2 LOW)

| File | Lines | Priority |
|------|-------|----------|
| `effect/triggers/evaluate/tests/bound_and_staged/mod.rs` | 127 | MEDIUM |
| `effect/effects/attraction/tests/mod.rs` | 102 | MEDIUM |
| `effect/effects/shockwave/tests/mod.rs` | 84 | MEDIUM |
| `effect/effects/pulse/tests/mod.rs` | 74 | MEDIUM |
| `effect/effects/piercing_beam/tests/mod.rs` | 65 | LOW |
| `effect/triggers/until/tests/mod.rs` | 20 | LOW |

## OPEN MEDIUM priority (12 files, 501-799 lines)

| File | Total | Notes |
|------|-------|-------|
| `breaker/systems/spawn_breaker/tests/spawn_or_reuse.rs` | 779 | 26 tests |
| `bolt/builder/tests/visual_tests.rs` | 751 | 26 tests |
| `bolt/builder/core.rs` | 742 | All prod, typestate builder |
| `shared/size.rs` | 664 | 98 prod, 565 tests |
| `bolt/builder/tests/build_tests.rs` | 620 | 23 tests |
| `bolt/systems/reset_bolt/tests.rs` | 601 | 22 tests |
| `effect/triggers/impacted/tests/context_entity_tests.rs` | 548 | 13 tests |
| `effect/effects/piercing_beam/tests/fire_tests/geometry_tests.rs` | 527 | 19 tests |
| `effect/triggers/impact/tests/context_entity_tests.rs` | 524 | 13 tests |
| `bolt/systems/bolt_lost/tests/lost_detection_tests.rs` | 500 | 16 tests |
| `bolt/systems/spawn_bolt/tests/migration_tests.rs` | 487 | 12 tests |
| `rantzsoft_spatial2d/builder.rs` | 451 | 295 prod, 156 tests |

## OPEN LOW priority

| File | Total | Notes |
|------|-------|-------|
| `effect/core/types/definitions/enums.rs` | 503 | 432 prod, 71 tests |
| `screen/chip_select/systems/generate_chip_offerings/tests.rs` | 461 | 11 tests |
| `effect/triggers/evaluate/tests/on_resolution/resolve_entity_targets.rs` | 462 | 13 tests |
| `effect/triggers/evaluate/tests/on_resolution/resolve_edge_cases.rs` | 443 | 9 tests |
| `effect/effects/anchor/tests/tick_timer_tests.rs` | 440 | 14 tests |
| `debug/hot_reload/systems/propagate_breaker_changes/tests.rs` | 438 | 10 tests |
| `effect/effects/tether_beam/tests/fire_tests/fire_basic.rs` | 435 | 15 tests |
| `effect/effects/spawn_phantom/tests/fire_tests.rs` | 431 | 13 tests |
| `bolt/systems/dispatch_bolt_effects/tests/basic_dispatch.rs` | 452 | 13 tests |

## Files confirmed unsplittable

- rantzsoft_physics2d/src/quadtree/tree.rs (451) -- single data structure, no tests
- rantzsoft_spatial2d/src/components/definitions.rs (402) -- pure type definitions
- breaker-scenario-runner/src/runner/execution.rs (439) -- single concern, no tests
- breaker-scenario-runner/src/runner/app.rs (402) -- single concern, tiny test section
