---
name: Phase 4+5 file length findings
description: Files flagged as over-threshold on feature/runtime-effects (Phase 4+5); major growth in effect/effects/ and several other domains
type: project
---

Reviewed on feature/runtime-effects (2026-03-28).

## effect/effects/ — all HIGH priority (1000+ lines or 800+ test lines)

These were all clean in Phase 3 (22–273 lines). Phase 4+5 added large test suites.

| File | Total | Prod | Tests | Test Fns | Strategy |
|------|-------|------|-------|----------|----------|
| attraction.rs | 1145 | 192 | 953 | 22 | A + sub-split |
| chain_lightning.rs | 1081 | 125 | 956 | 25 | A + sub-split |
| tether_beam.rs | 1080 | 246 | 834 | 28 | A + sub-split |
| piercing_beam.rs | 1049 | 163 | 886 | 23 | A + sub-split |
| pulse.rs | 873 | 188 | 685 | 18 | A + sub-split |
| entropy_engine.rs | 725 | 99 | 626 | 20 | A (single tests.rs) |
| shockwave.rs | 694 | 131 | 563 | 17 | A (single tests.rs) |
| spawn_bolts.rs | 599 | 113 | 486 | 19 | A (single tests.rs) |
| chain_bolt.rs | 573 | 120 | 453 | 13 | A (single tests.rs) |
| explode.rs | 553 | 79 | 474 | 14 | A (single tests.rs) |
| spawn_phantom.rs | 524 | 112 | 412 | 14 | A (single tests.rs) |
| second_wind.rs | 440 | 95 | 345 | 10 | A (single tests.rs) |

## New large files outside effect/effects/

### Strategy C — already-extracted tests.rs needing sub-split
- bolt/systems/bolt_lost/tests.rs — 1135 lines, 24 test fns. HIGH.
- cells/systems/handle_cell_hit/tests.rs — 1051 lines, 23 test fns. HIGH.

### Strategy A — inline tests need extraction
- bolt/systems/spawn_bolt.rs — 555 lines (96 prod / 459 tests, 17 fns). MEDIUM.
- wall/systems/spawn_walls.rs — 522 lines (74 prod / 448 tests, 12 fns). MEDIUM.
- run/node/definition.rs — 556 lines (132 prod / 424 tests, 22 fns). MEDIUM.
- run/definition.rs — 543 lines (185 prod / 358 tests, 20 fns). MEDIUM.

### Strategy C — already-extracted tests.rs at MEDIUM threshold
- run/systems/track_node_cleared_stats/tests.rs — 729 lines, 21 test fns. MEDIUM (watch).
- chips/resources/tests.rs — 682 lines, 29 test fns. MEDIUM (watch).
- run/systems/generate_node_sequence/tests.rs — 566 lines, 17 test fns. MEDIUM (watch).

### mod.rs with production code (rules violation)
- breaker-scenario-runner/src/lifecycle/mod.rs — 1068 lines, production plugin code in mod.rs. HIGH.

### rantzsoft_physics2d
- rantzsoft_physics2d/src/quadtree.rs — 1235 lines (488 prod / 747 tests, 29 fns). HIGH. Strategy A + sub-split.

## Previously flagged (still open from Phase 3)
- chips/offering.rs — 777 lines. HIGH. Strategy A.
- effect/triggers/impacted.rs — 876 lines. HIGH. Strategy A.
- effect/triggers/impact.rs — 792 lines. HIGH. Strategy A.
- bolt/systems/bolt_wall_collision.rs — 497 lines. MEDIUM. Strategy A.
- effect/core/types.rs — 517 lines. MEDIUM. Strategy B.
