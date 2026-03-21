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

## Confirmed-Clean New Systems (reviewed 2026-03-20, session 3)

### physics/systems/bolt_cell_collision.rs — 3 MessageWriter params
- Added `wall_hit_writer: MessageWriter<BoltHitWall>` alongside existing `hit_writer` and `damage_writer`.
- In Bevy 0.18 all MessageWriter params share the same deferred command buffer as the system's Commands — no additional per-writer overhead, no new parallelism conflict.
- System was already serialized by its mutable bolt_query + Commands. Third writer adds zero scheduling cost.

### bolt/behaviors/bridges.rs — bridge_overclock_breaker_impact, bridge_overclock_wall_impact
- Structurally identical to bridge_overclock_cell_impact (already clean).
- MessageReader drains early-exit on no events; armed_query: Query<&mut ArmedTriggers> hits 0–1 entities.
- All three impact bridges access &mut ArmedTriggers — cannot run in parallel with each other. Correct and expected; they are all ordered after(PhysicsSystems::BreakerCollision).
- No new archetype fragmentation: ArmedTriggers is already tracked as 1-entity add/remove.

### bolt/behaviors/bridges.rs — bridge_overclock_bump double evaluation
- Iterates active.0 twice per bump message: once for grade-specific trigger, once for BumpSuccess.
- Max 3 chains × 2 passes × 1 pure match each = negligible. evaluate() is a pure enum pattern match.
- Not a hot-path concern at any foreseeable chip stack cap. Correct design, not double-work.

### bolt/behaviors/evaluate.rs — ImpactTarget 3-arm explicit match vs wildcard
- Explicit (CellImpact, OnImpact(Cell, inner)) | (BreakerImpact, OnImpact(Breaker, inner)) | (WallImpact, OnImpact(Wall, inner)) arms compile identically to a wildcard after optimization.
- The explicit arms are a correctness win (prevent cross-target misfires). Zero performance difference.

### Deferred Item Update
- resolve_armed Vec allocation (bridges.rs:289-298): confirmed still deferred. Moderate concern only when multi-bolt upgrades arrive (Phase 7+). At 1 bolt and typical <4 armed chains the per-bounce Vec::new() is negligible.

## Confirmed-Clean New Systems (reviewed 2026-03-20)

### bolt/behaviors/effects/shockwave.rs — handle_shockwave observer
- Fires only on OverclockEffectFired (event-driven, not polling). Early-returns for non-Shockwave variants.
- Cell query (updated 2026-03-20 session 7): `ShockwaveCellQuery = (Entity, &Transform, Has<Locked>)` with `With<Cell>` filter — shockwave no longer needs &mut CellHealth or RequiredToClear since it writes DamageCell messages instead. Smaller tuple, no mutable component access. Clean.
- Has<Locked> used correctly for skip logic (cheaper than Without<Locked> filter here because locked cells also need to be iterated past; no archetype penalty).
- Archetype fragmentation: no new fragmentation vs prior baseline — Locked is existing archetype component. No new concern.
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

## Confirmed-Clean New Systems (reviewed 2026-03-21, session on feature/overclock-trigger-chain)

### chips/definition.rs — 7 new TriggerChain variants (branch: refactor/unify-behaviors)
- 4 new leaf variants: LoseLife (0 payload), SpawnBolt (0 payload), TimePenalty { seconds: f32 } (4B), BoltSpeedBoost { multiplier: f32 } (4B).
- 3 new trigger wrappers: OnEarlyBump(Box<Self>), OnLateBump(Box<Self>), OnBumpWhiff(Box<Self>).
- Enum size impact: NONE. Discriminant expands trivially (still 1 byte). Largest variant is Shockwave/MultiBolt/Shield at 12 bytes — unchanged by new variants. Box<Self> wrappers are all 8 bytes. No size regression.
- ECS impact: TriggerChain is NOT a component or resource by itself. It lives inside ActiveOverclocks(Vec<TriggerChain>) (Res) and ArmedTriggers(Vec<TriggerChain>) (Component). Neither archetype fragmentation nor query cost is affected by adding enum variants.
- Hot-path impact: evaluate() is a pure pattern match called O(active_chains) times per bridge event (typically <5 chains). Adding 7 variants to the match adds zero branches to existing trigger kinds — the `else { NoMatch }` fallback absorbs them cheaply.
- The 3 new trigger wrappers (OnEarlyBump, OnLateBump, OnBumpWhiff) have NO corresponding OverclockTriggerKind variants or bridge systems yet — they are dead types until wired. Zero runtime cost for now.
- The 4 new leaf variants (LoseLife, TimePenalty, SpawnBolt, BoltSpeedBoost) have NO effect handler observers yet — they are dead types until wired. Zero runtime cost for now.
- bridge_overclock_bump handles BumpGrade::Early|Late → None for grade-specific trigger (no Early/LateBump kind yet). BumpSuccess fires for all non-whiff grades. Correct and intentional pending wiring.
