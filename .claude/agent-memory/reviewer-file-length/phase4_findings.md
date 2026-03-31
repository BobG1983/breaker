---
name: Post-refactor file length findings
description: Phase 7+ open findings; updated 2026-03-31 after Effective* cache removal refactor; 6 HIGH + 2 mod.rs violations + 15 MEDIUM open
type: project
---

Updated after Effective* cache removal refactor (2026-03-31).

## Previously HIGH priority — ALL SPLIT (as of post-new-scenarios merge)

All 7 files listed as HIGH priority after source-chip-shield-absorption have been split:
- `chips/systems/dispatch_chip_effects/tests/desugaring.rs` (1461) → `desugaring/` directory
- `chips/systems/dispatch_chip_effects/tests/dispatch.rs` (1044) → `dispatch/` directory
- `effect/triggers/evaluate/tests/on_resolution.rs` (934) → `on_resolution/` directory
- `lifecycle/tests/initial_effects.rs` (896) → `initial_effects/` directory
- `effect/effects/chain_lightning/tests/fire_tests.rs` (781) → `fire_tests/` directory
- `rantzsoft_defaults/src/systems/tests.rs` (769) → `tests/` directory
- `effect/core/types/definitions.rs` (730) → `definitions/` directory

## Previously open mod.rs violations — ALL FIXED

- `breaker-scenario-runner/src/input/mod.rs` — now routing-only
- `breaker-scenario-runner/src/verdict/mod.rs` — now routing-only

## OPEN mod.rs violations (Wave 3 scan, not yet fixed)

| File | Total | Violation |
|------|-------|-----------|
| `breaker-game/src/breaker/systems/dash/tests/mod.rs` | 471 | Contains test helpers + 14 test functions — must extract to `dash_state_tests.rs` |
| `breaker-game/src/effect/effects/tether_beam/tests/mod.rs` | 130 | Contains shared test helper functions — must extract to `helpers.rs` |

## OPEN HIGH priority (not yet split)

| File | Total | Prod | Tests | Test Fns | Strategy | Priority |
|------|-------|------|-------|----------|----------|----------|
| `effect/effects/anchor/tests.rs` | 1141 | 0 | 1141 | 33 | C: fire_reverse + tick_timer + bump_forces | HIGH |
| `effect/effects/tether_beam/tests/fire_tests.rs` | 1068 | 0 | 1068 | 36 | C: fire_basic + chain_fire + chain_reverse + dispatch | HIGH |
| `chips/systems/dispatch_chip_effects/tests/edge_cases.rs` | 842 | 0 | 842 | 18 | C: read file to group | HIGH |
| `breaker/systems/dash/tests/flash_step.rs` | 918 | 0 | 918 | 20 | C: reversal + clamping + reset + active_boosts | HIGH |
| `chips/systems/dispatch_chip_effects/tests/desugaring/e2e.rs` | 817 | 0 | 817 | 4 | C: already at boundary, read to group | HIGH |
| `breaker/systems/dash/tests/mod.rs` | 471 | 0 | 471 | 14 | mod.rs violation + extract to dash_state_tests.rs | HIGH |

Notes on changes from Wave 3:
- `anchor/tests.rs`: was 1166, now 1141 (shrank 25 lines after Effective* cache removal)
- `flash_step.rs`: was 824, now 918 (grew 94 lines — 2 new tests for active boost reads)

## Previously open MEDIUM priority — ALL SPLIT (refactor/file-splits branch, merged to develop 2026-03-30)

All 25 files from the previous list were split by the refactor/file-splits branch. Verified as directory modules.

## OPEN MEDIUM priority (Wave 3 scan + Effective* cache removal updates)

| File | Total | Prod | Tests | Test Fns | Strategy | Priority |
|------|-------|------|-------|----------|----------|----------|
| `effect/effects/circuit_breaker/tests.rs` | 782 | 0 | 782 | 18 | C: fire_counter + fire_reward + reverse + edge_cases | MEDIUM |
| `breaker/systems/bump/tests/anchor_multipliers.rs` | 769 | 0 | 769 | 17 | C: forward_grade + retroactive + timer + planted | MEDIUM |
| `effect/effects/piercing_beam/tests/fire_tests.rs` | 654 | 0 | 654 | 20 | C: geometry + damage + source_chip | MEDIUM |
| `effect/commands/tests/transfer_insert_tests.rs` | 623 | 0 | 623 | 18 | C: permanent + non_permanent + edge_cases | MEDIUM |
| `bolt/systems/bolt_wall_collision/tests.rs` | 610 | 0 | 610 | 15 | C: impact + reflection + last_impact + piercing_reset | MEDIUM |
| `effect/effects/piercing_beam/tests/process_tests.rs` | 609 | 0 | 609 | 16 | C: damage + source_chip + targeting | MEDIUM |
| `invariants/checkers/check_aabb_matches_entity_dimensions/tests.rs` | 515 | 0 | 515 | 20 | C: bolt_aabb + breaker_aabb + combined | MEDIUM |
| `effect/effects/attraction/tests/apply_tests/steering_tests.rs` | 506 | 0 | 506 | 10 | C: basic_steering + multi_target + edge_cases | MEDIUM |
| `effect/effects/gravity_well/tests/pull_tests.rs` | 493 | 0 | 493 | 10 | C: steering + boundary + position2d | MEDIUM |
| `breaker/systems/move_breaker.rs` | 468 | 97 | 370 | 11 | A: test extraction | MEDIUM |
| `bolt/systems/spawn_bolt/tests.rs` | 467 | 0 | 467 | 17 | C: at boundary, monitor | MEDIUM |
| `breaker/systems/dash/tests/mod.rs` | 471 | 0 | 471 | 14 | mod.rs violation — listed under HIGH | HIGH |
| `chips/definition/tests/template_tests.rs` | 475 | 0 | 475 | 13 | C: at boundary, monitor | MEDIUM |
| `cells/resources.rs` | 460 | 152 | 307 | 14 | A: test extraction | MEDIUM |
| `rantzsoft_physics2d/src/systems/maintain_quadtree.rs` | 453 | 64 | 388 | 9 | A: test extraction | MEDIUM |
| `rantzsoft_physics2d/src/systems/enforce_distance_constraints.rs` | 436 | 59 | 376 | 7 | A: test extraction | MEDIUM |

Notes on changes from Wave 3:
- `bolt_wall_collision/tests.rs`: was 572, now 610 (grew 38 lines — 1 new test added)
- `move_breaker.rs`: NEW — was not in Wave 3 scan (added after last scan)
- `attraction/tests/apply_tests/steering_tests.rs`: NEW — was not in Wave 3 scan

## OPEN LOW priority

| File | Total | Notes |
|------|-------|-------|
| `effect/core/types/definitions/enums.rs` | 512 | Mostly prod (406 lines), tests only 105 — low priority for extraction |
| `bolt/systems/bolt_cell_collision/tests/last_impact.rs` | 435 | 10 tests, under sub-split threshold |
| `chips/resources/tests/chip_catalog.rs` | 431 | 19 tests, already separate — monitor |
| `run/node/systems/spawn_cells_from_layout/tests/position2d.rs` | 436 | 10 tests, already separate |
| `effect/effects/pulse/tests/damage_tests.rs` | 411 | 12 tests, already separate |
| `effect/core/types/tests.rs` | 411 | 18 tests, already separate — monitor |

Previously LOW: `breaker-scenario-runner/src/types/tests/invariant_kinds.rs` (was 418) dropped to 375 after Effective* cache removal — REMOVED from tracking.

## Files confirmed unsplittable

- rantzsoft_physics2d/src/quadtree/tree.rs — single data structure, no tests
- breaker-scenario-runner/src/runner/execution.rs — single concern, no tests
- breaker-scenario-runner/src/runner/app.rs — single concern, tiny test section (44 test lines)
