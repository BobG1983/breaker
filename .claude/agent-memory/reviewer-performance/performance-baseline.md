---
name: Performance Baseline
description: Entity scale expectations, confirmed efficient patterns, fragmentation risks, known hotspots
type: reference
---

## Entity Scale Expectations
- Phase 1-2: ~50 cells, 1 bolt, 1 breaker, 3 walls — most concerns are theoretical
- Phase 3+: upgrades add entity variety but not significantly more count
- Grid layouts now support up to 128×128 = 16,384 entities (design-time maximum, not typical gameplay)
- Typical gameplay grids expected to remain small (3–10 cols, 2–8 rows); 128×128 is a pathological upper bound
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

## Confirmed-Clean New Systems (reviewed 2026-03-20, session 2)

### run/node/systems/spawn_cells_from_layout.rs — compute_grid_scale + spawn_cells_from_grid
- `compute_grid_scale` is a pure function: O(1), called once per spawn event (not per entity). Confirmed clean.
- Scaled dimension arithmetic (cell_width, cell_height, step_x, step_y, start_x, start_y) computed once before the loop. No per-entity divisions or scaling math. Clean.
- Per-entity `materials.add(ColorMaterial::from_color(...))` is intentional: damage visual system mutates each cell's material independently. Accepted design decision; not a bug.
- Shared `rect_mesh` handle is cloned per entity (`rect_mesh.clone()` on the handle — a cheap `Arc`-like clone, not a mesh copy). One mesh allocation per spawn call regardless of cell count. Efficient.
- Loop is O(rows × cols), skips '.' and unrecognized aliases early. No allocations inside the loop body beyond the `commands.spawn(...)` command itself.
- At 16K entities the spawn itself is a one-time O(N) operation at level load. Not a hot path; acceptable.
- CCD collision inner loop is O(bolts × cells × MAX_BOUNCES=4). At 16K cells this becomes a real concern if typical grids reach that scale. Flagged as Moderate watch item. At current typical scale (50 cells) it remains clean.
- `u16::try_from(col_idx).unwrap_or(u16::MAX)` / `u16::try_from(row_idx)` pattern: safe saturation for extreme grids. Accepted.

## Confirmed-Clean New Systems (reviewed 2026-03-20)

### bolt/behaviors/effects/shockwave.rs — handle_shockwave observer
- Fires only on OverclockEffectFired (event-driven, not polling). Early-returns for non-Shockwave variants.
- Cell query: `Query<(Entity, &Transform, &mut CellHealth, Has<RequiredToClear>, Has<Locked>), With<Cell>>` — correct filter.
- Has<Locked> used correctly for skip logic (cheaper than Without<Locked> filter here because locked cells also need to be iterated past; no archetype penalty).
- Archetype fragmentation: Locked + RequiredToClear adds to known cell archetypes (already tracked in baseline). No new concern.
- No allocations inside the observer body. Clean.

### bolt/behaviors/bridges.rs — bridge systems + resolve_armed
- `resolve_armed` allocates `Vec::new()` per call and swaps it into `armed.0`. This is NOT a hot-path: fires only when an OverclockEffectFired chain resolves (rare game event, not per-frame). Acceptable.
- `bridge_overclock_cell_destroyed` / `bridge_overclock_bolt_lost` guards: `reader.read().count() == 0` early-exit — prevents work when no events. But NOTE: calling `.count()` drains the iterator; the events are consumed. Fine because the only needed info is "did any arrive."
- `bridge_overclock_cell_destroyed` / `bridge_overclock_bolt_lost` declare `armed_query: Query<(Entity, &mut ArmedTriggers)>` — mutable access even in function signature passed immutably to `evaluate_armed_all`. The `&mut ArmedTriggers` is warranted (resolve_armed mutates armed.0). Correct.
- `arm_bolt` inserts `ArmedTriggers(vec![remaining])` — single allocation at arm-time. Event-driven, not hot-path.
- Observer registration via `add_observer(handle_shockwave)`: standard Bevy 0.18 global observer. Per-call overhead is observer dispatch + query, not per-frame. No concern.

### Archetype Note
- ArmedTriggers added/removed at runtime per bolt. With 1 bolt this is negligible. If multi-bolt upgrades arrive (Phase 7+), watch for archetype churn if bolts frequently arm/disarm mid-run.

### Intentional Pattern: resolve_armed swap idiom
- `drain(..)` into `new_armed` then assign back to `armed.0` is the standard "process and rebuild" Vec pattern. At typical scale (≤3 armed chains per bolt, ≤3 active overclock chips), this is negligible. Not worth changing.
