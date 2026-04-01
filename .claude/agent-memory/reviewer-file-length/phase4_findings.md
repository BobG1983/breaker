---
name: Post-refactor file length findings
description: Wave 4 scan (feature/chip-evolution-ecosystem): 4 HIGH + 2 mod.rs violations + 14 MEDIUM open; bolt/builder.rs is new massive file (2308 lines)
type: project
---

Updated 2026-04-01 on feature/chip-evolution-ecosystem branch.

## Key changes from Wave 3 (2026-03-31)

- NEW: `bolt/builder.rs` at 2308 lines (504 prod, 1804 tests, 80 test fns) -- massive new file from bolt typestate builder migration
- Many files shrank significantly (Effective* cache removal + other refactors)
- 9 files dropped below 400 threshold: cells/resources.rs, maintain_quadtree.rs, enforce_distance_constraints.rs, gravity_well/pull_tests.rs, last_impact.rs, chip_catalog.rs, position2d.rs, damage_tests.rs, core/types/tests.rs
- All previously LOW files now below threshold

## Previously HIGH priority -- ALL SPLIT (as of post-new-scenarios merge)

All 7 files from earlier waves have been split into directory modules.

## Previously open mod.rs violations -- ALL FIXED (scenario-runner input + verdict)

## OPEN HIGH priority (not yet split)

| File | Total | Prod | Tests | Test Fns | Strategy | Priority |
|------|-------|------|-------|----------|----------|----------|
| `bolt/builder.rs` | 2308 | 504 | 1804 | 80 | A: test extraction + sub-split (6 test files) | HIGH |
| `effect/effects/anchor/tests.rs` | 985 | 0 | 985 | 33 | C: fire_reverse + tick_timer + bump_forces | HIGH |
| `effect/effects/tether_beam/tests/fire_tests.rs` | 915 | 0 | 915 | 36 | C: fire_basic + chain_fire + chain_reverse + dispatch | HIGH |
| `breaker/systems/dash/tests/flash_step.rs` | 821 | 0 | 821 | 20 | C: reversal + clamping + reset + active_boosts | HIGH |

## OPEN mod.rs violations

| File | Total | Violation |
|------|-------|-----------|
| `breaker-game/src/breaker/systems/dash/tests/mod.rs` | 408 | Contains 4 helpers + 14 test functions -- must extract to helpers.rs + dash_state_tests.rs |
| `breaker-game/src/effect/effects/tether_beam/tests/mod.rs` | 130 | Contains 7 helper functions + 1 test resource type -- must extract to helpers.rs |

## OPEN MEDIUM priority

| File | Total | Prod | Tests | Test Fns | Strategy | Priority |
|------|-------|------|-------|----------|----------|----------|
| `chips/systems/dispatch_chip_effects/tests/desugaring/e2e.rs` | 713 | 0 | 713 | 4 | C: all_cells + all_bolts + all_walls | MEDIUM |
| `chips/systems/dispatch_chip_effects/tests/edge_cases.rs` | 703 | 0 | 703 | 18 | C: error_handling + inventory + source_chip | MEDIUM |
| `effect/effects/circuit_breaker/tests.rs` | 691 | 0 | 691 | 18 | C: fire_counter + fire_reward + reverse | MEDIUM |
| `breaker/systems/bump/tests/anchor_multipliers.rs` | 679 | 0 | 679 | 17 | C: forward_grade + retroactive + window_duration | MEDIUM |
| `effect/commands/tests/transfer_insert_tests.rs` | 560 | 0 | 560 | 18 | C: permanent + non_permanent + edge_cases | MEDIUM |
| `effect/effects/piercing_beam/tests/fire_tests.rs` | 545 | 0 | 545 | 20 | C: geometry + damage + source_chip | MEDIUM |
| `effect/effects/piercing_beam/tests/process_tests.rs` | 505 | 0 | 505 | 16 | C: damage_processing + source_chip | MEDIUM |
| `breaker-scenario-runner aabb tests.rs` | 487 | 0 | 487 | 20 | C: bolt_aabb + breaker_aabb | MEDIUM |
| `bolt/systems/bolt_wall_collision/tests.rs` | 486 | 0 | 486 | 15 | C: impact + last_impact + piercing | MEDIUM |
| `effect/effects/attraction/tests/apply_tests/steering_tests.rs` | 451 | 0 | 451 | 10 | C: basic_steering + edge_cases | MEDIUM |
| `chips/definition/tests/template_tests.rs` | 444 | 0 | 444 | 13 | C: basic + evolution templates | MEDIUM |
| `bolt/systems/spawn_bolt/tests.rs` | 429 | 0 | 429 | 17 | C: primary + extra bolt tests | MEDIUM |
| `breaker/systems/move_breaker.rs` | 422 | 98 | 324 | 11 | A: test extraction | MEDIUM |

## OPEN LOW priority

| File | Total | Notes |
|------|-------|-------|
| `effect/core/types/definitions/enums.rs` | 479 | Mostly prod (407 lines), tests only 72 -- low priority |

## Files confirmed unsplittable

- rantzsoft_physics2d/src/quadtree/tree.rs (451) -- single data structure, no tests
- breaker-scenario-runner/src/runner/execution.rs (439) -- single concern, no tests
- breaker-scenario-runner/src/runner/app.rs (402) -- single concern, tiny test section (2 tests)
