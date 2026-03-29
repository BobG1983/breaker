---
name: Phase 3 file length findings
description: Files flagged as over-threshold after feature/stat-effects (Phase 3); tracks which need splitting and which were pre-existing
type: project
---

Reviewed after `feature/stat-effects` merged to `develop` (2026-03-28).

## HIGH priority (need splitting soon)

- `breaker-game/src/chips/offering.rs` — 777 lines (160 prod / 617 tests, 24 test fns). Strategy A. Pre-existing, not Phase 3 growth.
- `breaker-game/src/effect/triggers/impacted.rs` — 876 lines (278 prod / 598 tests, 9 test fns). Strategy A. Pre-existing.
- `breaker-game/src/effect/triggers/impact.rs` — 792 lines (278 prod / 514 tests, 9 test fns). Strategy A. Pre-existing.

## MEDIUM priority (schedule for next modification)

- `breaker-game/src/bolt/systems/bolt_wall_collision.rs` — 497 lines (134 prod / 363 tests, 6 test fns). Strategy A. Phase 3 added ~30 lines pushing it over 400. Extract to bolt_wall_collision/ directory.
- `breaker-game/src/effect/core/types.rs` — 517 lines, all production code, 0 tests. Strategy B (concern separation: triggers.rs + nodes.rs). Pre-existing.

## LOW / within limits (watch)

- `breaker-game/src/bolt/systems/bolt_cell_collision/tests/piercing.rs` — 453 lines, pure test file. Already extracted. Phase 3 added ~50 lines. Under 800-line sub-split threshold. Watch; flag at ~600.

## Effect files confirmed clean

All `effect/effects/*.rs` files (attraction, bump_force, chain_bolt, damage_boost, gravity_well, piercing, quick_stop, ramping_damage, shield, shockwave, size_boost, spawn_phantom, speed_boost, etc.) are 22–273 lines. No action needed.
