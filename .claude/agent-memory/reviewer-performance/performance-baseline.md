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
- ~~`interpolate_transform`~~ DELETED 2026-03-24 (spatial/physics extraction). Replaced by `derive_transform` (AfterFixedMainLoop) which uses `With<DrawLayer>` filter. Interpolation via `InterpolateTransform2D` marker + rantzsoft_spatial2d pipeline.
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
- Chip effect components (`Piercing`, `DamageBoost`, `BoltSpeedBoost` (Amp flat-speed chip), `ChainHit`, `BoltSizeBoost`, `WidthBoost`, `BreakerSpeedBoost`, `BumpForceBoost`, `TiltControlBoost`) added once at chip-select time via observers. Fine at 1-bolt/1-breaker scale.

## Known Hotspots
- `bolt_cell_collision` (FixedUpdate): O(bolts × cells × MAX_BOUNCES=4). Watch if multi-bolt upgrades added.
- `compute_globals` HashMap allocation: 1 HashMap::new() per RunFixedMainLoop call. Trivial at <100 entities. Watch if entity counts grow significantly (16K cell grids would make this O(16K) allocation every visual frame).
- `propagate_position/rotation/scale` still registered alongside `derive_transform`: both write Transform on the same entities every frame. propagate_* output overwrites derive_transform output — redundant work. The old systems should be removed once derive_transform is validated as the replacement. At current scale (50-60 entities) the redundant work is 3 extra system traversals costing ~microseconds, not a real hitch.
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

### bolt/systems/bolt_cell_collision.rs — 3 MessageWriter params (moved from physics/ 2026-03-24)
- Added `wall_hit_writer: MessageWriter<BoltHitWall>` alongside existing `hit_writer` and `damage_writer`.
- In Bevy 0.18 all MessageWriter params share the same deferred command buffer as the system's Commands — no additional per-writer overhead, no new parallelism conflict.
- System was already serialized by its mutable bolt_query + Commands. Third writer adds zero scheduling cost.

### behaviors/bridges.rs — bridge_breaker_impact, bridge_wall_impact (was bolt/behaviors/bridges.rs)
- Structurally identical to bridge_cell_impact (already clean).
- MessageReader drains early-exit on no events; armed_query: Query<&mut ArmedTriggers> hits 0–1 entities.
- All three impact bridges access &mut ArmedTriggers — cannot run in parallel with each other. Correct and expected; they are all ordered after(BoltSystems::BreakerCollision).
- No new archetype fragmentation: ArmedTriggers is already tracked as 1-entity add/remove.

### behaviors/bridges.rs — bridge_bump double evaluation (was bolt/behaviors/bridges.rs)
- Iterates active.0 twice per bump message: once for grade-specific trigger, once for BumpSuccess.
- Max 3 chains × 2 passes × 1 pure match each = negligible. evaluate() is a pure enum pattern match.
- Not a hot-path concern at any foreseeable chip stack cap. Correct design, not double-work.

### behaviors/evaluate.rs — ImpactTarget 3-arm explicit match vs wildcard (was bolt/behaviors/evaluate.rs)
- Explicit (CellImpact, OnImpact(Cell, inner)) | (BreakerImpact, OnImpact(Breaker, inner)) | (WallImpact, OnImpact(Wall, inner)) arms compile identically to a wildcard after optimization.
- The explicit arms are a correctness win (prevent cross-target misfires). Zero performance difference.

### Deferred Item Update
- resolve_armed Vec allocation (bridges.rs:289-298): confirmed still deferred. Moderate concern only when multi-bolt upgrades arrive (Phase 7+). At 1 bolt and typical <4 armed chains the per-bounce Vec::new() is negligible.

## Confirmed-Clean New Systems (reviewed 2026-03-20)

### behaviors/effects/shockwave.rs — handle_shockwave observer (was bolt/behaviors/effects/shockwave.rs)
- Fires only on EffectFired (was OverclockEffectFired — renamed in refactor/unify-behaviors). Event-driven, not polling. Early-returns for non-Shockwave variants.
- Cell query (updated 2026-03-20 session 7): `ShockwaveCellQuery = (Entity, &Transform, Has<Locked>)` with `With<Cell>` filter — shockwave no longer needs &mut CellHealth or RequiredToClear since it writes DamageCell messages instead. Smaller tuple, no mutable component access. Clean.
- Has<Locked> used correctly for skip logic (cheaper than Without<Locked> filter here because locked cells also need to be iterated past; no archetype penalty).
- Archetype fragmentation: no new fragmentation vs prior baseline — Locked is existing archetype component. No new concern.
- No allocations inside the observer body. Clean.

### behaviors/bridges.rs — bridge systems + resolve_armed (was bolt/behaviors/bridges.rs)
- `resolve_armed` allocates `Vec::new()` per call and swaps it into `armed.0`. This is NOT a hot-path: fires only when an EffectFired (was OverclockEffectFired) chain resolves (rare game event, not per-frame). Acceptable.
- `bridge_cell_destroyed` / `bridge_bolt_lost` guards: `reader.read().count() == 0` early-exit — prevents work when no events. But NOTE: calling `.count()` drains the iterator; the events are consumed. Fine because the only needed info is "did any arrive."
- `bridge_cell_destroyed` / `bridge_bolt_lost` declare `armed_query: Query<(Entity, &mut ArmedTriggers)>` — mutable access even in function signature passed immutably to `evaluate_armed_all`. The `&mut ArmedTriggers` is warranted (resolve_armed mutates armed.0). Correct.
- `arm_bolt` inserts `ArmedTriggers(vec![remaining])` — single allocation at arm-time. Event-driven, not hot-path.
- Observer registration via `add_observer(handle_shockwave)`: standard Bevy 0.18 global observer. Per-call overhead is observer dispatch + query, not per-frame. No concern.

### Archetype Note
- ArmedTriggers added/removed at runtime per bolt. With 1 bolt this is negligible. If multi-bolt upgrades arrive (Phase 7+), watch for archetype churn if bolts frequently arm/disarm mid-run.

### Intentional Pattern: resolve_armed swap idiom
- `drain(..)` into `new_armed` then assign back to `armed.0` is the standard "process and rebuild" Vec pattern. At typical scale (≤3 armed chains per bolt, ≤3 active overclock chips), this is negligible. Not worth changing.

## Confirmed-Clean New Systems (reviewed 2026-03-23, feature/wave-3-offerings-transitions, memorable moments)

### run/systems/ — detection systems (detect_mass_destruction, detect_combo_and_pinball, detect_close_save, detect_nail_biter, detect_first_evolution, track_node_cleared_stats)

SCHEDULING — 10+ systems in FixedUpdate gated with run_if(in_state(PlayingState::Active)). All passive message-reader pattern (drain, early-exit when no messages). Scheduling overhead is negligible: Bevy 0.18 run_if short-circuits at resource check, system body is skipped. Clean.

DETECT_MASS_DESTRUCTION — Vec<f32> (cell_destroyed_times) in HighlightTracker. Bounded per node: at most ~50-100 cells per typical node. Vec is cleared at node exit (reset_highlight_tracker). No unbounded growth. `retain()` runs every FixedUpdate unconditionally even when no messages arrived; retain on an empty Vec is O(1). Clean.

DETECT_COMBO_AND_PINBALL — 3 MessageReaders via SystemParam. Pure counter increments; no queries or allocations. Clean.

DETECT_CLOSE_SAVE — bolt_query.get(msg.bolt) inside loop. Correct pattern: called per BumpPerformed message (rare event, not per-frame). With<Bolt> + Without<BoltServing> filter produces clean 2-archetype separation (ActiveBoltFilter pattern). Clean.

DETECT_NAIL_BITER — bolt_query.iter() inside NodeCleared handler. Called once per NodeCleared (rare event). Iterates 1-2 bolts. Clean. bolt_query identical archetype filter to detect_close_save; shares archetype cache.

TRACK_NODE_CLEARED_STATS — 8 conditions checked, all against scalar fields in HighlightTracker / NodeTimer. No queries, no allocations. NodeCleared fires once per node. Clean.

HIGHLIGHTS VEC DEDUP — stats.highlights.iter().any(|h| h.kind == kind) is O(cap) = O(5). Not a scan concern.

SPAWN_HIGHLIGHT_TEXT — NOT REGISTERED in any plugin. Function is exported from systems mod but absent from RunPlugin::build. Text popups will never appear in-game. This is a correctness bug, not a performance concern, but noted here because it means the FadeOut entity accumulation concern (entity leak) is moot — no entities are spawned.

ANIMATE_FADE_OUT — runs Update, PlayingState::Active guard. Query: FadeOut + TextColor, no marker filter. Since spawn_highlight_text is unregistered, zero entities match. Fine.

CONFIRM-EFFICIENT PATTERNS:
- detect_close_save + detect_nail_biter both use (With<Bolt>, Without<BoltServing>) — matches existing ActiveBoltFilter archetype convention. Confirmed clean.
- All detection systems are message-reader pattern: drain → early-exit if empty → process. Zero CPU cost in steady state with no messages.
- No allocations in any hot path. HighlightTracker fields are all primitive scalars or the one Vec<f32> (bounded, cleared per node).
- reset_highlight_tracker runs OnEnter(GameState::Playing) — correct placement, not FixedUpdate.

OPEN ISSUE (correctness, not performance): spawn_highlight_text is exported but not registered in RunPlugin::build. Highlights are detected and HighlightTriggered is emitted correctly, but no text popup is ever spawned.

## Confirmed-Clean New Systems (reviewed 2026-03-23, feature/wave-3-offerings-transitions, spatial2d refactor)

### rantzsoft_spatial2d — compute_globals, derive_transform, apply_velocity, save_previous

COMPUTE_GLOBALS — Per-frame HashMap allocation is the headline concern. HashMap is allocated every RunFixedMainLoop, not per-frame in the game-loop sense, but it runs every visual frame (AfterFixedMainLoop). At current scale: ~50 cells (root) + ~3 orbit children (shield cells) = ~53 entities. HashMap growth is trivial. The while-made_progress loop is O(depth × entity_count); depth = 1 for shield/orbit pattern, so ~2 passes total. This is a Minor/Watch item — see Known Hotspots section. Not critical at current entity counts.

DERIVE_TRANSFORM — Runs AfterFixedMainLoop, filtered by DrawLayer presence. Same entity count as above. Optional field access (Option<&InterpolateTransform2D>, Option<&PreviousPosition>, etc.) adds branch overhead per entity but these are cheap boolean checks. 0 allocations. Clean at current scale.

SAVE_PREVIOUS — Now has 4 separate queries (pos, rot, scale, vel) all filtered With<InterpolateTransform2D>. All 4 share the same archetype filter — Bevy 0.18 caches archetype matches per query. At 1 bolt with InterpolateTransform2D, each query iterates 1 entity. Velocity query runs on entities with both Velocity2D and InterpolateTransform2D — that's exactly the bolt. Confirmed efficient.

APPLY_VELOCITY — Clean. Filtered With<ApplyVelocity> marker. No allocations, O(N) where N = entities with marker (0 or 1 today). Correct FixedUpdate placement.

DOUBLE-WORK (compute_globals + propagate_position/rotation/scale both running) — CONFIRMED ISSUE. All five systems (compute_globals, derive_transform, propagate_position, propagate_rotation, propagate_scale) run every AfterFixedMainLoop in a chain. derive_transform writes Transform from Global*; propagate_position/rotation/scale ALSO write Transform from local Position2D/Rotation2D/Scale2D. Both write the same Transform component on the same entities, with propagate_* overwriting derive_transform's output each frame. This is redundant work — flagged as Moderate in reviews.

ANIMATE_SHOCKWAVE material mutation — `materials.get_mut(handle.id())` runs in Update every frame the shockwave exists. Shockwave is a short-lived entity (seconds), so this is brief hot-path material mutation. Each frame causes a dirty flag in Bevy's asset system, triggering re-upload to GPU. The shockwave is 1 entity; negligible at current scale. Watch if multiple simultaneous shockwaves become common.

SHOCKWAVE MESH/MATERIAL SPAWN — meshes.add(Annulus) + materials.add(ColorMaterial) allocated per shockwave trigger in handle_shockwave observer. Event-driven (not per-frame). 1 shockwave at a time in current design. Accepted.

## Confirmed-Clean New Systems (reviewed 2026-03-24, spatial/physics extraction)

### rantzsoft_physics2d — maintain_quadtree, enforce_distance_constraints

MAINTAIN_QUADTREE — FixedUpdate, Changed<GlobalPosition2D> filter prevents per-frame full scan for static entities. Bolt triggers Changed every frame = 1 entity updated per frame (remove + insert). `changed_pos.get(entity)` inside the changed_layers loop is an O(N_changed_layers) HashMap lookup — at current scale (CollisionLayers never change after spawn) the inner body is never entered. Clean. The Added<Aabb2D> / is_added() double-insert guard is correct.

ENFORCE_DISTANCE_CONSTRAINTS — FixedUpdate, iterates all DistanceConstraint entities. Currently 0 constraints in gameplay. If tether mechanic added: 1 constraint = 1 get_many_mut call. Clean at any foreseeable constraint count.

PLUGIN SCHEDULING — both systems in FixedUpdate with named system sets (MaintainQuadtree, EnforceDistanceConstraints). Game collision systems correctly ordered .after(PhysicsSystems::MaintainQuadtree). Clean.

### Archetype note for physics2d
- Aabb2D + CollisionLayers added at spawn, never removed in normal gameplay. Zero runtime archetype churn from physics components.

## Confirmed-Clean New Systems (reviewed 2026-03-21, session on feature/overclock-trigger-chain)

### chips/definition.rs — 7 new TriggerChain variants (branch: refactor/unify-behaviors — NOW FULLY WIRED)
- 4 new leaf variants: LoseLife (0 payload), SpawnBolt (0 payload), TimePenalty { seconds: f32 } (4B), SpeedBoost { target: SpeedBoostTarget, multiplier: f32 } (was BoltSpeedBoost { multiplier: f32 } — renamed and expanded in refactor/unify-behaviors).
- 3 new trigger wrappers: OnEarlyBump(Box<Self>), OnLateBump(Box<Self>), OnBumpWhiff(Box<Self>) — all now wired in TriggerKind + evaluate() + bridge_bump_whiff.
- Enum size impact: NONE. Discriminant expands trivially (still 1 byte). Largest variant is Shockwave/MultiBolt/Shield at 12 bytes — unchanged by new variants. Box<Self> wrappers are all 8 bytes. No size regression.
- ECS impact: TriggerChain is NOT a component or resource by itself. It lives inside ActiveChains(Vec<TriggerChain>) (Res) and ArmedTriggers(Vec<TriggerChain>) (Component). Neither archetype fragmentation nor query cost is affected by adding enum variants.
- Hot-path impact: evaluate() is a pure pattern match called O(active_chains) times per bridge event (typically <5 chains). All variants are now wired; no dead-type overhead.
- All 4 leaf variants now have registered effect observers: handle_life_lost, handle_time_penalty, handle_spawn_bolt, handle_speed_boost (was handle_bolt_speed_boost) in behaviors/effects/.
