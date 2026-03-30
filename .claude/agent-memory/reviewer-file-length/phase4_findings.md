---
name: Phase 4+5+6 file length findings
description: Files flagged as over-threshold on feature/runtime-effects and feature/source-chip-shield-absorption; updated 2026-03-29 with Phase 6 additions
type: project
---

Reviewed on feature/source-chip-shield-absorption (2026-03-29 update).

## Already split since last review (now clean)

These were HIGH/MEDIUM in Phase 4 memory and have been split since:
- bolt/systems/bolt_lost/tests.rs — now tests/ dir with shield_tests.rs (757 lines — see current open findings), lost_detection_tests.rs, extra_bolt_tests.rs
- cells/systems/handle_cell_hit/tests.rs — now tests/ dir with shield_tests.rs (750 lines — see current open findings), damage_tests.rs, request_tests.rs
- effect/effects/attraction.rs — now attraction/ dir with effect.rs + tests/ (apply_tests.rs 838 lines — see current open findings, fire_tests.rs, manage_tests.rs)
- effect/effects/chain_lightning — now tests/ dir, then tick_tests/ sub-split (arc_tests.rs 479, lifecycle_tests.rs 443 — see current open findings, idle_tests.rs 282)
- effect/effects/tether_beam — now tests/ dir (tick_damage_tests.rs 586 lines, fire_tests.rs, tick_lifetime_tests.rs)
- effect/effects/piercing_beam — now tests/ dir (process_tests.rs 615 lines, fire_tests.rs 435 lines)
- effect/effects/pulse — now tests/ dir (tick_tests.rs 531 lines, damage_tests.rs, fire_tests.rs)
- effect/effects/shockwave — now tests/ dir (damage_tests.rs 543 lines — see current open findings, expansion_tests.rs, fire_tests.rs)
- breaker-scenario-runner/src/lifecycle/mod.rs — now lifecycle/systems.rs (1268 lines, production only — see current HIGH)

## Current open findings (2026-03-29 Phase 6)

### HIGH priority (1000+ lines) — split immediately

| File | Total | Prod | Tests | Test Fns | Strategy |
|------|-------|------|-------|----------|----------|
| cells/systems/dispatch_cell_effects.rs | 1525 | 102 | 1423 | 24 | A: test extraction + sub-split |
| breaker-scenario-runner/src/lifecycle/systems.rs | 1268 | 1268 | 0 | 0 | B: concern separation |
| chips/systems/dispatch_chip_effects/tests/dispatch.rs | 970 | 0 | 970 | 24 | C: sub-split |
| effect/core/types.rs | 999 | 685 | 313 | 14 | B: concern separation |

### HIGH priority — mod.rs violations

| File | Total | Issue |
|------|-------|-------|
| breaker-scenario-runner/src/types/mod.rs | 590 | Production code in mod.rs (has tests/ subdir) |
| breaker-scenario-runner/src/input/mod.rs | 211 | Production code in mod.rs (has tests.rs alongside) |
| breaker-scenario-runner/src/verdict/mod.rs | 163 | Production code in mod.rs (has tests.rs alongside) |

### MEDIUM priority (501–999 lines) — split at next opportunity

| File | Total | Prod | Tests | Test Fns | Strategy |
|------|-------|------|-------|----------|----------|
| chips/systems/dispatch_chip_effects/tests/edge_cases.rs | 811 | 0 | 811 | 18 | C: sub-split |
| effect/triggers/impacted.rs | 876 | 278 | 598 | 9 | A: test extraction |
| breaker/systems/dispatch_breaker_effects/tests.rs | 877 | 0 | 877 | 20 | C: sub-split |
| effect/effects/attraction/tests/apply_tests.rs | 838 | 0 | 838 | 16 | C: sub-split |
| effect/triggers/impact.rs | 792 | 278 | 514 | 9 | A: test extraction |
| chips/offering.rs | 777 | 72 | 705 | 24 | A: test extraction |
| bolt/systems/bolt_lost/tests/shield_tests.rs | 757 | 0 | 757 | 14 | C: sub-split |
| cells/systems/handle_cell_hit/tests/shield_tests.rs | 750 | 0 | 750 | 15 | C: sub-split |
| breaker-scenario-runner/src/lifecycle/tests/initial_effects.rs | 745 | 0 | 745 | 15 | C: sub-split |
| breaker-scenario-runner/src/runner/tests.rs | 744 | 0 | 744 | 39 | C: sub-split |
| effect/effects/entropy_engine/tests.rs | 736 | 0 | 736 | 23 | C: sub-split |
| run/systems/track_node_cleared_stats/tests.rs | 729 | 0 | 729 | 21 | C: sub-split |
| effect/effects/explode/tests.rs | 685 | 0 | 685 | 21 | C: sub-split |
| chips/resources/tests.rs | 682 | 0 | 682 | 29 | C: sub-split |
| run/node/systems/spawn_cells_from_layout/tests/shield_cells.rs | 634 | 0 | 634 | 17 | C: sub-split |
| breaker-scenario-runner/src/verdict/tests.rs | 633 | 0 | 633 | 22 | C: sub-split |
| breaker-scenario-runner/src/lifecycle/tests/frame_mutations.rs | 626 | 0 | 626 | 13 | C: sub-split |
| effect/triggers/evaluate.rs | 623 | 138 | 485 | 12 | A: test extraction |
| run/systems/spawn_highlight_text/tests/ordering.rs | 613 | 0 | 613 | 15 | C: sub-split |
| breaker-scenario-runner/src/lifecycle/tests/debug_setup.rs | 609 | 0 | 609 | 13 | C: sub-split |
| effect/effects/tether_beam/tests/tick_damage_tests.rs | 586 | 0 | 586 | ~15 | C: sub-split |
| breaker-scenario-runner/src/types/mod.rs | 590 | ~200+ | ~350 | many | HIGH mod.rs violation |
| effect/effects/shockwave/tests/damage_tests.rs | 543 | 0 | 543 | 17 | C: sub-split |
| effect/effects/pulse/tests/tick_tests.rs | 531 | 0 | 531 | 13 | C (watch) |
| run/systems/generate_node_sequence/tests.rs | 566 | 0 | 566 | 17 | C (watch) |

### LOW priority (400–500 lines) — flag for awareness

| File | Total | Notes |
|------|-------|-------|
| breaker/systems/spawn_breaker/tests.rs | 506 | C: sub-split when touched |
| effect/triggers/until.rs | 508 | A: test extraction |
| breaker-scenario-runner/invariants/checkers/bolt_in_bounds/tests.rs | 507 | C (watch) |
| run/systems/select_highlights/tests.rs | 494 | C: sub-split when touched |
| run/resources.rs | 497 | A: test extraction |
| run/node/resources.rs | 494 | A: test extraction |
| bolt/systems/bolt_wall_collision.rs | 499 | A: test extraction |
| breaker-scenario-runner/src/runner/execution.rs | 493 | B: concern separation |
| bolt/systems/bolt_breaker_collision/tests/collision.rs | 490 | C (watch) |
| effect/effects/chain_lightning/tests/tick_tests/arc_tests.rs | 479 | C (watch) |
| chips/systems/build_chip_catalog.rs | 479 | A: test extraction |
| breaker/systems/dash/tests.rs | 467 | C (watch) |
| bolt/systems/spawn_bolt/tests.rs | 467 | C (watch) |
| effect/effects/random_effect.rs | 466 | A: test extraction |
| cells/resources.rs | 460 | A: test extraction |
| bolt/systems/bolt_cell_collision/tests/piercing.rs | 453 | C (watch) |
| wall/systems/spawn_walls/tests.rs | 451 | C (watch) |
| effect/effects/chain_bolt/tests.rs | 451 | A: test extraction |
| breaker-scenario-runner/src/runner/app.rs | 447 | B: concern separation |
| screen/chip_select/systems/handle_chip_input.rs | 443 | A: test extraction |
| effect/effects/chain_lightning/tests/tick_tests/lifecycle_tests.rs | 443 | C (watch) |
| effect/effects/second_wind.rs | 440 | A: test extraction |
| run/node/systems/spawn_cells_from_layout/tests/position2d.rs | 436 | C (watch) |
| effect/effects/piercing_beam/tests/fire_tests.rs | 435 | C (watch) |
| breaker-scenario-runner/src/input/tests.rs | 432 | C (watch) |
| screen/chip_select/systems/generate_chip_offerings.rs | 432 | A: test extraction |
| screen/run_end/tests/highlight_selection.rs | 427 | C (watch) |
| breaker-scenario-runner/invariants/checkers/valid_breaker_state/tests.rs | 422 | C (watch) |
| run/node/definition/tests.rs | 421 | C (watch) |
| effect/effects/spawn_phantom/tests.rs | 403 | C (watch) |
| rantzsoft_spatial2d/src/systems/compute_globals.rs | 755 | A: test extraction (rantzsoft) |
| rantzsoft_spatial2d/src/components.rs | 734 | A: test extraction (rantzsoft) |
| rantzsoft_spatial2d/src/plugin.rs | 640 | A: test extraction (rantzsoft) |
| rantzsoft_spatial2d/src/systems/save_previous.rs | 487 | A: test extraction (rantzsoft) |
| rantzsoft_physics2d/src/quadtree/tree.rs | 487 | pure prod (rantzsoft) |
| rantzsoft_physics2d/src/systems/maintain_quadtree.rs | 453 | A: test extraction (rantzsoft) |
| rantzsoft_physics2d/src/systems/enforce_distance_constraints.rs | 436 | A: test extraction (rantzsoft) |
| rantzsoft_physics2d/src/ccd.rs | 406 | A: test extraction (rantzsoft) |
