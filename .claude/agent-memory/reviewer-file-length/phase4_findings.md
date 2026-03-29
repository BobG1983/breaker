---
name: Phase 4+5 file length findings
description: Files flagged as over-threshold on feature/runtime-effects (Phase 4+5); updated 2026-03-29 with current splits status
type: project
---

Reviewed on feature/runtime-effects (2026-03-29 update — many Phase 4 files now split).

## Already split since last review (now clean)

These were HIGH/MEDIUM in Phase 4 memory and have been split since:
- bolt/systems/bolt_lost/tests.rs — now tests/ dir with shield_tests.rs, lost_detection_tests.rs, extra_bolt_tests.rs
- cells/systems/handle_cell_hit/tests.rs — now tests/ dir with damage_tests.rs, request_tests.rs, shield_tests.rs
- effect/effects/attraction.rs — now attraction/ dir with effect.rs + tests/ (apply_tests.rs, fire_tests.rs, manage_tests.rs)
- effect/effects/chain_lightning — now tests/ dir (tick_tests.rs 1192 lines, fire_tests.rs 785 lines) — tick_tests.rs still HIGH, fire_tests.rs MEDIUM
- effect/effects/tether_beam — now tests/ dir (tick_damage_tests.rs 586 lines, fire_tests.rs, tick_lifetime_tests.rs)
- effect/effects/piercing_beam — now tests/ dir (process_tests.rs 615 lines, fire_tests.rs 435 lines)
- effect/effects/pulse — now tests/ dir (tick_tests.rs 531 lines, damage_tests.rs, fire_tests.rs)
- breaker-scenario-runner/src/lifecycle/mod.rs — now lifecycle/systems.rs (1112 lines, production only)

## Current open findings (2026-03-29)

### HIGH priority (1000+ lines)

| File | Total | Prod | Tests | Test Fns | Strategy |
|------|-------|------|-------|----------|----------|
| effect/effects/chain_lightning/tests/tick_tests.rs | 1192 | 0 | 1192 | 23 | C: sub-split |
| effect/core/types.rs | 998 | 685 | 313 | 14 | B: concern separation |
| lifecycle/systems.rs (scenario runner) | 1112 | 1112 | 0 | 0 | B: concern separation |

### HIGH priority — mod.rs violations

| File | Total | Issue |
|------|-------|-------|
| breaker-scenario-runner/src/types/mod.rs | 568 | Production code in mod.rs (has tests/ subdir) |
| breaker-scenario-runner/src/input/mod.rs | 211 | Production code in mod.rs (has tests.rs alongside) |
| breaker-scenario-runner/src/verdict/mod.rs | 163 | Production code in mod.rs (has tests.rs alongside) |

### MEDIUM priority (501–999 lines)

| File | Total | Prod | Tests | Test Fns | Strategy |
|------|-------|------|-------|----------|----------|
| effect/triggers/impacted.rs | 876 | 278 | 598 | 9 | A: test extraction |
| effect/triggers/impact.rs | 792 | 278 | 514 | 9 | A: test extraction |
| chips/offering.rs | 777 | 72 | 705 | 24 | A: test extraction |
| effect/effects/shockwave/tests.rs | 851 | 0 | 851 | 26 | C: sub-split |
| effect/effects/entropy_engine/tests.rs | 736 | 0 | 736 | 23 | C: sub-split |
| run/systems/track_node_cleared_stats/tests.rs | 729 | 0 | 729 | 21 | C: sub-split |
| breaker-scenario-runner/src/runner/tests.rs | 743 | 0 | 743 | 39 | C: sub-split |
| breaker-scenario-runner/src/verdict/tests.rs | 626 | 0 | 626 | 22 | C: sub-split |
| effect/triggers/evaluate.rs | 623 | 138 | 485 | 12 | A: test extraction |
| effect/effects/explode/tests.rs | 685 | 0 | 685 | 21 | C: sub-split |
| run/systems/generate_node_sequence/tests.rs | 566 | 0 | 566 | 17 | C (watch) |
| chips/resources/tests.rs | 682 | 0 | 682 | 29 | C: sub-split |
| breaker-scenario-runner/src/lifecycle/tests/debug_setup.rs | 609 | 0 | 609 | 13 | C: sub-split |

### LOW priority (400–500 lines)

| File | Total | Prod | Tests | Test Fns | Strategy |
|------|-------|------|-------|----------|----------|
| effect/effects/chain_lightning/tests/fire_tests.rs | 785 | 0 | 785 | 25 | C: sub-split |
| effect/effects/spawn_bolts/tests.rs | 483 | 0 | 483 | 19 | C (watch) |
| effect/effects/tether_beam/tests/tick_damage_tests.rs | 586 | 0 | 586 | ~15 | C: sub-split |
| effect/effects/piercing_beam/tests/process_tests.rs | 615 | 0 | 615 | ~15 | C: sub-split |
| effect/effects/pulse/tests/tick_tests.rs | 531 | 0 | 531 | 13 | C (watch) |
| effect/effects/chain_bolt/tests.rs | 451 | 0 | 451 | 13 | A: test extraction |
| effect/effects/spawn_phantom/tests.rs | 403 | 0 | 403 | 14 | LOW watch |
| effect/effects/second_wind.rs | 440 | 95 | 345 | 10 | A: test extraction |
| effect/triggers/until.rs | 510 | 76 | 434 | 7 | A: test extraction |
| run/resources.rs | 497 | 272 | 225 | 20 | A: test extraction |
| run/node/resources.rs | 494 | 130 | 364 | 18 | A: test extraction |
| bolt/systems/bolt_wall_collision.rs | 497 | 134 | 363 | 6 | A: test extraction |
| screen/chip_select/systems/handle_chip_input.rs | 443 | 95 | 348 | 15 | A: test extraction |
| screen/chip_select/systems/generate_chip_offerings.rs | 432 | 86 | 346 | 9 | A: test extraction |
| chips/systems/build_chip_catalog.rs | 479 | 112 | 367 | 7 | A: test extraction |
| effect/effects/random_effect.rs | 466 | 55 | 411 | 16 | A: test extraction |
| breaker-scenario-runner/src/runner/execution.rs | 493 | 493 | 0 | 0 | B: concern separation |
| breaker-scenario-runner/src/runner/app.rs | 446 | 441 | 5 | 2 | B: concern separation |
| breaker-scenario-runner/lifecycle/tests/perfect_tracking.rs | 498 | 0 | 498 | 12 | C (watch) |
| breaker-scenario-runner/invariants/checkers/timer_monotonically_decreasing.rs | 485 | 71 | 414 | 10 | A: test extraction |
| breaker-scenario-runner/src/lifecycle/tests/debug_setup.rs | 609 | 0 | 609 | 13 | MEDIUM |
