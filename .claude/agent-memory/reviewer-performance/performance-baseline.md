---
name: Performance Baseline
description: Entity scale expectations, confirmed efficient patterns, fragmentation risks, known hotspots
type: reference
---

## Entity Scale Expectations
- Phase 1-2: ~50 cells, 1 bolt, 1 breaker, 3 walls — most concerns are theoretical
- Phase 3+: upgrades add entity variety but not significantly more count
- Phase 7+ (roguelite meta): may introduce persistent entities across runs

## Confirmed Efficient Patterns
- All hot-path queries use proper `With<>` / `Without<>` filters
- `ActiveBoltFilter`, `CellCollisionFilter`, `WallCollisionFilter`, `BreakerCollisionFilter` in `physics/filters.rs`
- `ServingBoltFilter` vs `ActiveBoltFilter` cleanly separate archetypes
- CCD collision loop is O(bolts × cells) — fine at current scale
- All `breaker_query.single()` calls in physics/bolt systems are outside the bolt loop
- Physics systems gated with `run_if(in_state(PlayingState::Active))`
- `handle_cell_hit` and `track_node_completion` are event-driven (not polling)
- Debug systems guarded by `resource_exists::<DebugOverlays>` and overlay flags

## Known Fragmentation Risks (Watch)
- `RequiredToClear` marker creates two cell archetypes. Fine at 50-cell scale.
- `BumpVisual` added/removed at runtime — 1 entity, negligible.
- `BoltServing` added/removed at launch — 1 entity, negligible.
- Phase 4b.2 chip effect components (`Piercing`, `DamageBoost`, `BoltSpeedBoost`, `ChainHit`, `BoltSizeBoost`, `WidthBoost`, `BreakerSpeedBoost`, `BumpForceBoost`, `TiltControlBoost`) are added once at chip-select time using `Option<&mut Component>` queries in observers. These run once per chip selection (not per frame). Fine at 1-bolt/1-breaker scale. Watch when multi-bolt upgrades arrive.

## Known Hotspots
- bolt_cell_collision (FixedUpdate): O(bolts × cells × MAX_BOUNCES=4). Watch if multi-bolt upgrades added.
- animate_bump_visual (Update): structural change on expiry. Once per bump event — not a concern.

## Deferred Issues (Fine Now, Watch Later)
- update_menu_colors runs every Update frame in MainMenu state unconditionally. Fine for ~3 items.
- update_lives_display runs every Update in Active, iterates all LivesDisplay (currently 1).
- bolt_info_ui / breaker_state_ui: String allocations via format!() every frame. Dev-only.
- update_chip_display: format!() every Update frame in ChipSelect state for the timer countdown text. 1 entity, short-lived state — negligible.
- 9 chip-effect observers each hold a broad `Option<&mut Component>` query across all bolts/breakers. Fine at 1 entity each. Each observer early-returns immediately on wrong effect variant — zero query cost for non-matching events.
