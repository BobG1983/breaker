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
- `ActiveBoltFilter`, `CollisionFilterCell`, `CollisionFilterWall`, `CollisionFilterBreaker` in `physics/filters.rs`
- `ServingFilter` vs `ActiveFilter` cleanly separate bolt archetypes
- CCD collision loop is O(bolts × cells × MAX_BOUNCES=4). Fine at current scale.
- All `breaker_query.single()` / `bolt_query.single()` calls in physics systems are outside the bolt loop
- Physics systems gated with `run_if(in_state(PlayingState::Active))`
- `handle_cell_hit` and `check_lock_release` are event-driven (not polling)
- Debug systems guarded by overlay flags (early return if not active)
- `tick_cell_regen` query uses `With<Cell>` — correct filter
- `interpolate_transform` runs PostUpdate, uses `With<InterpolateTransform>` to opt-in — minimal entities
- `animate_tilt_visual`, `width_boost_visual`, `animate_bump_visual` run Update, `With<Breaker>` filtered — 1 entity
- `bolt_lost` uses `Local<Vec<LostBoltEntry>>` for scratch storage — zero allocs after warmup
- `bolt_cell_collision` uses `Local<Vec<Entity>>` (pierced_this_frame) — zero allocs after warmup

## Archetype Fragmentation (Watch)
- `RequiredToClear` marker: two cell archetypes. Fine at 50-cell scale.
- `Locked` marker: adds a third cell archetype. Fine at 50-cell scale.
- `CellRegen` component: cells with regen form a fourth archetype (subset query). Fine at current scale.
- `LockAdjacents(Vec<Entity>)`: heap allocation per locked-cell entity; each adjacency check is a Vec scan (fine at ~50 cells with few lock cells).
- `BumpVisual` added/removed at runtime — 1 entity, negligible.
- `BoltServing` added/removed at launch — 1 entity, negligible.
- `Locked` added/removed at runtime (via `check_lock_release`) — rare structural change, fine.
- Chip effect components (`Piercing`, `DamageBoost`, `BoltSpeedBoost`, `ChainHit`, `BoltSizeBoost`, `WidthBoost`, `BreakerSpeedBoost`, `BumpForceBoost`, `TiltControlBoost`) added once at chip-select time via observers. Fine at 1-bolt/1-breaker scale.

## Known Hotspots
- `bolt_cell_collision` (FixedUpdate): O(bolts × cells × MAX_BOUNCES=4). Watch if multi-bolt upgrades added.
- `pierced_this_frame.contains()`: linear scan O(n) per cell check in CCD inner loop. Bounded by MAX_BOUNCES=4 — negligible at current scale.
- `check_lock_release`: runs every FixedUpdate, polls all `With<Locked>` cells unconditionally (not event-triggered after message drain). Fine at <10 locked cells; becomes polling overhead at scale.
- `despawned.contains()` in `handle_cell_hit`: linear scan O(n). Bounded by MAX_BOUNCES×bolts hits per frame — negligible.
- `ActiveBehaviors::consequences_for()` / `has_trigger()`: linear scans through `Vec<(Trigger, Consequence)>` — typically <10 entries; fine forever.
- `animate_bump_visual`: structural change (remove::<BumpVisual>) on expiry. Once per bump event.

## Deferred Issues (Fine Now, Watch Later)
- `update_menu_colors` runs every Update frame in MainMenu state unconditionally. Fine for ~3 items.
- `bolt_info_ui` / `breaker_state_ui`: String allocations via format!() every frame. Dev-only (feature-gated).
- `update_chip_display`: format!() every Update frame in ChipSelect state for the timer countdown text. 1 entity, short-lived state — negligible.
- `update_timer_display`: format!() every Update frame in Active state. 1 entity — negligible.
- 9 chip-effect observers each hold a broad `Option<&mut Component>` query. Fine at 1 entity each; each early-returns on wrong variant — zero query cost for non-matching events.
- `check_lock_release` drains `CellDestroyed` reader unconditionally then re-checks all locked cells via entity liveness. Not purely event-driven; acceptable at <10 locked cells.
- `LockAdjacents(Vec<Entity>)`: allocates per locked-cell entity. Could be a fixed-size array. Not worth changing now.
- `width_boost_visual` (Update, 1 entity): writes Transform::scale every frame unconditionally. Negligible at 1 entity.
- `spawn_additional_bolt`: allocates Mesh + ColorMaterial per bolt spawn (event-driven, not hot-path). Watch if multi-bolt stacking becomes common in Phase 7+.
- `animate_fade_out` query (FadeOut + TextColor): no marker filter. Steady-state entity count is 0-3. Negligible.

## Confirmed-Clean New Systems (reviewed 2026-03-19)
- chips domain (9 effect observers): all event-driven, early-return on wrong variant, 1-2 entities each
- `apply_chip_effect`: gated in_state(GameState::ChipSelect), correct
- `spawn_bump_grade_text`, `spawn_whiff_text`, `spawn_bolt_lost_text`: event-driven, single() outside loop
- `check_lock_release`: destroyed_count guard prevents scan when nothing destroyed
- ExtraBolt / ServingFilter separation: correct, already in baseline
- Dash system (breaker): runs in_state(PlayingState::Active), With<Breaker> filtered — 1 entity
