---
name: Post-refactor file length findings
description: Phase 7+ open findings updated 2026-03-30; all HIGH priority and mod.rs violations from source-chip-shield-absorption have been split; MEDIUM priority items remain
type: project
---

Updated after full scan post-new-scenarios merge (2026-03-30).

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

- `breaker-scenario-runner/src/input/mod.rs` — now routing-only (drivers/ + re-exports)
- `breaker-scenario-runner/src/verdict/mod.rs` — now routing-only (evaluation/ + re-exports)

## MEDIUM priority still open (as of 2026-03-30)

These are the MEDIUM items from the phase4 list that have NOT been confirmed split yet.
Run a fresh `reviewer-file-length` pass before acting — some may have been addressed.

| File | ~Lines | Strategy |
|------|--------|----------|
| effect/effects/attraction/tests/apply_tests.rs | 838 | C: basic + force_clamping |
| bolt/systems/bolt_lost/tests/shield_tests.rs | 757 | C: absorption + reflection + extra_bolt |
| cells/systems/handle_cell_hit/tests/shield_tests.rs | 750 | C: absorption + dedup + edge_cases |
| runner/tests.rs (scenario-runner) | 744 | C: cli_parsing + run_list + execution + stress + grouping |
| effect/effects/entropy_engine/tests.rs | 736 | C: fire + reset + edge_cases |
| run/systems/track_node_cleared_stats/tests.rs | 729 | C: basic_stats + highlight_detection + streaks |
| effect/effects/explode/tests.rs | 685 | C: fire + process + source_chip |
| chips/resources/tests.rs | 682 | C: registry + recipes + templates |
| chips/offering/tests.rs | 646 | C: read file first |
| run/node/systems/spawn_cells_from_layout/tests/shield_cells.rs | 634 | C: spawning + orbit + locking |
| verdict/tests.rs (scenario-runner) | 633 | C: defaults + evaluation + health + violations |
| systems/compute_globals/tests.rs (rantzsoft_spatial2d) | 627 | C: root + hierarchy + absolute |
| lifecycle/tests/frame_mutations.rs (scenario-runner) | 626 | C: read file first |
| run/systems/spawn_highlight_text/tests/ordering.rs | 613 | C: layout + timing + culling |
| lifecycle/tests/debug_setup.rs (scenario-runner) | 609 | C: read file first |
| effect/triggers/impacted/tests.rs | 598 | C: bolt + breaker + other collisions |
| types/definitions.rs (scenario-runner) | 587 | B: read file first |
| effect/effects/tether_beam/tests/tick_damage_tests.rs | 574 | C: read file first |
| run/systems/generate_node_sequence/tests.rs | 566 | C: sequence_properties + boss_nodes + system |
| plugin/tests.rs (rantzsoft_spatial2d) | 554 | C: plugin_behavior + system_ordering |
| effect/effects/shockwave/tests/damage_tests.rs | 543 | C: read file first |
| effect/triggers/impact/tests.rs | 518 | C: bolt + breaker + other collisions |
| effect/effects/pulse/tests/tick_tests.rs | 527 | C: read file first |
| invariants/checkers/bolt_in_bounds/tests.rs (scenario-runner) | 507 | C: read file first |
| breaker/systems/spawn_breaker/tests.rs | 506 | C: read file first |

## Files confirmed unsplittable

- rantzsoft_physics2d/src/quadtree/tree.rs — single data structure, no tests
- breaker-scenario-runner/src/runner/execution.rs — single concern, no tests
- breaker-scenario-runner/src/runner/app.rs — single concern, tiny test section
