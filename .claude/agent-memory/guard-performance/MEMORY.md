# guard-performance Memory

## Entity Scale Expectations
- Phase 1-2: ~50 cells, 1 bolt, 1 breaker, 3 walls — most performance concerns are theoretical at this scale
- Phase 3+: upgrades add entity variety but not significantly more count
- Phase 7+ (roguelite meta): may introduce persistent entities across runs

## Confirmed Efficient Patterns
- All hot-path queries use proper `With<>` / `Without<>` filters — no broad unfiltered scans
- `ActiveBoltFilter`, `CellCollisionFilter`, `WallCollisionFilter`, `BreakerCollisionFilter` in `physics/filters.rs` provide correct archetype separation
- `ServingBoltFilter` vs `ActiveBoltFilter` cleanly separate archetypes (BoltServing marker added/removed at launch)
- CCD collision loop (bolt_cell_collision) is O(bolts × cells) — fine at current scale (~50 cells, 1 bolt)
- All `breaker_query.single()` calls in physics/bolt systems are outside the bolt loop — correct pattern
- Physics systems gated with `run_if(in_state(PlayingState::Active))` — not running in menus/transitions
- `handle_cell_hit` is event-driven (reads `BoltHitCell` messages) — not polling every frame
- `track_node_completion` is event-driven (reads `CellDestroyed`) — not querying remaining cells each frame
- Debug systems guarded by `resource_exists::<DebugOverlays>` and overlay flags — no perf impact in release

## Known Fragmentation Risks (Watch)
- `RequiredToClear` is a marker component added selectively at spawn; creates two cell archetypes
  (Cell+RequiredToClear vs Cell-only). At current 50-cell scale this is fine. CellCollisionFilter
  doesn't filter on RequiredToClear so both archetypes are visited by physics — correct.
- `BumpVisual` is added/removed at runtime on the breaker entity (animates bump pop).
  This is 1 entity affected, so archetype churn is negligible.
- `BoltServing` added/removed at launch on the bolt entity — 1 entity, negligible.

## Deferred Issues (Fine Now, Watch Later)
- update_menu_colors runs every Update frame in MainMenu state, unconditionally rewriting TextColor
  for ~3 menu items. Fine now. If menu items grow significantly, add `run_if(resource_changed::<MainMenuSelection>())`
- update_lives_display runs every Update in PlayingState::Active, iterates all LivesDisplay entities
  (currently 1). Fine now. Change-detection guard warranted if HUD entities multiply.
- bolt_info_ui / breaker_state_ui (dev feature only): String allocations via format!() every frame.
  Dev-only, not a production concern.

## Known Hotspots
- bolt_cell_collision (FixedUpdate): O(bolts × cells × MAX_BOUNCES=4). At 50 cells and 1 bolt this
  is trivially cheap. Worth watching if multi-bolt upgrades are added in Phase 3+.
- animate_bump_visual (Update): commands.entity().remove::<BumpVisual>() on expiry causes structural
  change. Happens at most once per bump event — not a concern.

## Session History
See [ephemeral/](ephemeral/) — not committed.
