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

## Known Hotspots
- bolt_cell_collision (FixedUpdate): O(bolts × cells × MAX_BOUNCES=4). Watch if multi-bolt upgrades added.
- animate_bump_visual (Update): structural change on expiry. Once per bump event — not a concern.

## Deferred Issues (Fine Now, Watch Later)
- update_menu_colors runs every Update frame in MainMenu state unconditionally. Fine for ~3 items.
- update_lives_display runs every Update in Active, iterates all LivesDisplay (currently 1).
- bolt_info_ui / breaker_state_ui: String allocations via format!() every frame. Dev-only.
