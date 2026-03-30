---
name: Post-refactor file length findings
description: Files over threshold after develop post-merge (c9964b7 split 23 files, 2026-03-30); updated 2026-03-30 with Phase 7 findings from source-chip-shield-absorption merge
type: project
---

Updated after full scan of develop branch on 2026-03-30 (after feature/source-chip-shield-absorption merged).

## Split by c9964b7 (now clean)

Previously HIGH priority — all converted to directory modules:
- cells/systems/dispatch_cell_effects.rs → dispatch_cell_effects/ dir
- breaker-scenario-runner/src/lifecycle/systems.rs → systems/ dir (11 files)
- chips/systems/dispatch_chip_effects/tests/dispatch.rs → split across multiple files
- effect/core/types.rs → core/types/ dir (definitions.rs + tests.rs)

Previously MEDIUM priority — all converted to directory modules:
- effect/triggers/impacted.rs → impacted/ dir
- breaker/systems/dispatch_breaker_effects/tests.rs → tests/ dir
- effect/effects/attraction/tests/apply_tests.rs (split)
- effect/triggers/impact.rs → impact/ dir
- chips/offering.rs → offering/ dir
- bolt/systems/bolt_wall_collision.rs → bolt_wall_collision/ dir
- effect/triggers/evaluate.rs → evaluate/ dir
- effect/triggers/until.rs → until/ dir
- effect/effects/entropy_engine/tests.rs, explode/tests.rs, random_effect.rs (split)
- chips/resources.rs → resources/ dir
- run/systems/track_node_cleared_stats/ (split)
- run/systems/spawn_highlight_text/ (split)
- breaker-scenario-runner/src/types/mod.rs → types/ dir (definitions.rs extracted)
- rantzsoft_spatial2d compute_globals, components, plugin, save_previous (split)
- rantzsoft_physics2d maintain_quadtree, enforce_distance_constraints, ccd (split)
- bolt/components.rs → components/ dir
- bolt/systems/bolt_wall_collision.rs → dir
- run/resources.rs → dir
- run/node/resources.rs → dir
- effect/effects/second_wind.rs → dir
- effect/effects/random_effect.rs → dir
- chips/systems/build_chip_catalog.rs → dir
- invariants/checkers/timer_monotonically_decreasing.rs → dir

## Current open findings (2026-03-30 Phase 7)

Split spec written to `.claude/specs/file-splits.md`.

### NEW HIGH priority (800+ lines) — appeared from source-chip-shield-absorption

| File | Total | Test Fns | Strategy |
|------|-------|----------|----------|
| chips/systems/dispatch_chip_effects/tests/desugaring.rs | 1461 | 24 | C: all_cells + all_walls + all_bolts + single_targets + breaker + misc |
| chips/systems/dispatch_chip_effects/tests/dispatch.rs | 1044 | 24 | C: bare_do + bound_effects + target_desugaring + mixed |
| effect/triggers/evaluate/tests/on_resolution.rs | 934 | 22 | C: walk_bound + resolve_commands + edge_cases |
| lifecycle/tests/initial_effects.rs (scenario-runner) | 896 | 18 | C: breaker_target + entity_targets + pending_apply + misc |
| effect/effects/chain_lightning/tests/fire_tests.rs | 781 | 24 | C: basic_fire + arc_behavior + targeting + chip_attribution |
| rantzsoft_defaults/src/systems/tests.rs | 769 | 18 | C: config_seeding + registry_seeding |
| effect/core/types/definitions.rs | 730 | 0 | B: targeting + graph + effect_kind |

### mod.rs violations (still open)

| File | Total | Notes |
|------|-------|-------|
| breaker-scenario-runner/src/input/mod.rs | 211 | Production code in mod.rs — has tests.rs alongside |
| breaker-scenario-runner/src/verdict/mod.rs | 163 | Production code in mod.rs — has tests.rs alongside |

### MEDIUM priority still open

| File | Total | Test Fns | Grouping |
|------|-------|----------|----------|
| effect/effects/attraction/tests/apply_tests.rs | 838 | 16 | basic + force_clamping |
| bolt/systems/bolt_lost/tests/shield_tests.rs | 757 | 14 | absorption + reflection + extra_bolt |
| cells/systems/handle_cell_hit/tests/shield_tests.rs | 750 | 15 | absorption + dedup + edge_cases |
| runner/tests.rs (scenario-runner) | 744 | 39 | cli_parsing + run_list + execution + stress + grouping |
| effect/effects/entropy_engine/tests.rs | 736 | 23 | fire + reset + edge_cases |
| run/systems/track_node_cleared_stats/tests.rs | 729 | 21 | basic_stats + highlight_detection + streaks |
| effect/effects/explode/tests.rs | 685 | 21 | fire + process + source_chip |
| chips/resources/tests.rs | 682 | 29 | registry + recipes + templates |
| chips/offering/tests.rs | 646 | 24 | read file first |
| run/node/systems/spawn_cells_from_layout/tests/shield_cells.rs | 634 | 17 | spawning + orbit + locking |
| verdict/tests.rs (scenario-runner) | 633 | 22 | defaults + evaluation + health + violations |
| systems/compute_globals/tests.rs (rantzsoft_spatial2d) | 627 | 13 | root + hierarchy + absolute |
| lifecycle/tests/frame_mutations.rs (scenario-runner) | 626 | 13 | read file first |
| run/systems/spawn_highlight_text/tests/ordering.rs | 613 | 15 | layout + timing + culling |
| lifecycle/tests/debug_setup.rs (scenario-runner) | 609 | 13 | read file first |
| effect/triggers/impacted/tests.rs | 598 | 9 | bolt + breaker + other collisions |
| types/definitions.rs (scenario-runner) | 587 | 0 | B: read file first |
| effect/effects/tether_beam/tests/tick_damage_tests.rs | 574 | ~15 | read file first |
| run/systems/generate_node_sequence/tests.rs | 566 | 17 | sequence_properties + boss_nodes + system |
| plugin/tests.rs (rantzsoft_spatial2d) | 554 | 15 | plugin_behavior + system_ordering |
| effect/effects/shockwave/tests/damage_tests.rs | 543 | 17 | read file first |
| effect/triggers/impact/tests.rs | 518 | 9 | bolt + breaker + other collisions |
| effect/effects/pulse/tests/tick_tests.rs | 527 | 13 | read file first |
| invariants/checkers/bolt_in_bounds/tests.rs (scenario-runner) | 507 | ~20 | read file first |
| breaker/systems/spawn_breaker/tests.rs | 506 | ~20 | read file first |

### Files confirmed unsplittable

- rantzsoft_physics2d/src/quadtree/tree.rs — single data structure, no tests
- breaker-scenario-runner/src/runner/execution.rs — single concern, no tests
- breaker-scenario-runner/src/runner/app.rs — single concern, tiny test section
