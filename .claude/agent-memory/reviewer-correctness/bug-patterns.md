---
name: Recurring bug patterns
description: Bug categories that appear repeatedly in this codebase — check these first on every review
type: project
---

## Insert-without-increment on first fire() — FIXED in Phase 1 cleanup

In effect `fire()` functions that accumulate state, the original bug inserted with
`accumulated: 0.0` on first call, missing the first trigger contribution. This was
confirmed in `ramping_damage.rs` Phase 1 original implementation.

**Current status**: FIXED. `ramping_damage::fire()` now inserts with
`accumulated: damage_per_trigger` on first call. `multi_call_accumulation_is_linear`
confirms: 4 calls at 0.5 yield 2.0 (not 1.5).

Watch for this pattern in other effect `fire()` functions that weren't reviewed.

## ccd_sweep_breaker fallback Aabb2D center semantics

`Aabb2D::new(center, half_extents)` stores `center` as-is in the `center` field.
`ray_intersect` uses `self.center` as world-space center. When constructing the
fallback expanded AABB in `ccd_sweep_breaker`, passing `breaker_pos` as center is
correct IF `breaker_pos` is already world-space. Confirmed correct — no bug.

## query_circle_filtered vs query_aabb_filtered in bolt_wall_collision

Old notes referred to `query_circle_filtered` — current code uses `query_aabb_filtered`.
The stale memory was corrected. Always verify which query function is actually used.

## gravity_well::fire() infinite loop when max == 0

`gravity_well::fire()` does `while owned.len() >= max as usize` to cap active wells.
When `max == 0`, `owned.len() >= 0` is always true. After draining `owned`, `owned.first()`
returns `None` → the inner `if let` never executes → infinite loop.

`spawn_phantom::fire()` has `if max_active == 0 { return; }` guard. `gravity_well` does not.

The RON file (`gravity_well.evolution.ron`) always uses `max: 2`, so this is not triggered in
production today. But it is a confirmed logic bug.

**Status**: FIXED in scenario-coverage branch — `fire()` now has `if max == 0 { return; }` guard at line 35-37.

## Phase 4 effect systems: shockwave/pulse migrated to Position2D (FIXED in full-verification-fixes)

`apply_shockwave_damage`, `apply_pulse_damage`, and `process_explode_requests` previously read
`transform.translation.truncate()` as the center. These were migrated to query `Position2D`
directly. Spawned shockwave/explode/ring entities now have `Position2D(position)` set at
fire-time. They still do NOT have `GlobalPosition2D` or `Spatial2D` — they are ephemeral
effect entities that do not move after spawn.

## Phase 5 effect fire(): Transform vs Position2D for bolt position

`chain_lightning::fire()` — FIXED in rework: now uses `Position2D` directly.

`piercing_beam::fire()` — FIXED in full-verification-fixes branch: now uses `entity_position()`
helper which returns `Position2D -> Vec2::ZERO` (no Transform fallback). Confirmed by tests in
`fire_tests.rs` (`fire_without_position2d_falls_back_to_zero_not_transform`, etc.).

## dispatch_chip_effects: effects dispatched even on max-stack add_chip failure — FIXED

`dispatch_chip_effects` now has `continue;` after the `add_chip` max-stacks warning
(system.rs line 57-59). The `for root_effect in &effects` loop is skipped when max stacks hit.

**Status**: FIXED — confirmed in code. Test: `chip_at_max_stacks_does_not_dispatch_effects`.

## apply_pending_bolt_effects: silently drops effects if bolt lacks BoundEffects — FIXED

`apply_pending_bolt_effects` (scenario-runner lifecycle/systems/pending_effects.rs) now uses
`commands.entity(entity).insert_if_new((BoundEffects::default(), StagedEffects::default()))`
before extending — matching the cell/wall variants.

**Status**: FIXED — confirmed in code.

## bypass_menu_to_playing: Target::Breaker initial_effects always dropped — FIXED

`PendingBreakerEffects` resource introduced in scenario-runner `lifecycle/systems/types.rs`.
`apply_pending_breaker_effects` registered in FixedUpdate after `tag_game_entities`.
`bypass_menu_to_playing` uses `commands.insert_resource(PendingBreakerEffects(...))`.

**Status**: FIXED — confirmed in code. `menu_bypass.rs` handles all 4 target types via Pending*Effects.

## TransferCommand silently drops non-Do children when BoundEffects/StagedEffects absent

`TransferCommand::apply` (effect/commands.rs) uses `entity_ref.get_mut::<BoundEffects>()` guarded
by `if let Some(...)`. If the entity lacks `BoundEffects` and `permanent: true`, non-`Do` children
(When/Once/Until nodes) are silently dropped. Same for `StagedEffects` when `permanent: false`.

**Status**: FIXED in full-verification-fixes branch. `ensure_effect_components()` helper now inserts
both `BoundEffects` and `StagedEffects` as defaults before the `get_mut` calls. Tests in
`commands.rs` section II cover all absent-component combinations.

## check_aabb_matches_entity_dimensions: false positive for breakers in non-1.0 EntityScale layouts

`check_aabb_matches_entity_dimensions` computes `expected = width.half_width() * scale` for breakers
and `expected = Vec2::splat(BoltRadius.0)` for bolts (no scale applied to bolt check).

The stored `Aabb2D` on both bolt and breaker entities is NEVER updated when `EntityScale` changes.
`apply_entity_scale_to_bolt` and `apply_entity_scale_to_breaker` only insert `EntityScale` on the
entity — neither touches `Aabb2D`. Physics systems compute live AABB from `BoltRadius * scale` /
`BreakerWidth * scale` directly; they do not use the stored component.

Result: for breakers in layouts with `entity_scale != 1.0` (e.g., `boss_arena.node.ron` has `entity_scale: 0.7`),
the checker fires false-positive `AabbMatchesEntityDimensions` violations because
`stored_half_extents (60.0, 10.0) != expected (42.0, 7.0)`.

For bolts, the checker uses `Vec2::splat(radius.0)` without multiplying by scale — so bolt checks
are always scale-1.0 semantics. This means bolt invariant is wrong for scaled layouts too.

**Status**: OPEN — confirmed on scenario-coverage branch review 2026-03-30.
Scenarios using Corridor (entity_scale=1.0) are not affected. Affects any scenario that adds
`AabbMatchesEntityDimensions` invariant to a non-1.0-scale layout.

## gravity_well and spawn_phantom fire(): missing despawned-entity guard — OPEN as of 2026-03-30

All 5 stat-boost `fire()` functions (speed_boost, damage_boost, size_boost, bump_force, piercing)
have `if world.get_entity(entity).is_err() { return; }` as the first guard.

`gravity_well/effect.rs::fire()` and `spawn_phantom/effect.rs::fire()` do NOT have this guard.
If fired on a despawned entity: spawns a ghost well/phantom with `owner: dead_entity`, pollutes
`GravityWellSpawnCounter`/`PhantomSpawnCounter` with a dead entity key, and the ghost takes up
a FIFO slot that can never be reclaimed for the same owner.

**Status**: OPEN — filed as regression spec hints in Wave 1 review 2026-03-30.

## TetherChainActive resource leaks across node boundaries — OPEN as of 2026-03-30

`TetherChainActive` is a `Resource` inserted by `fire_chain`. It is only removed by
`reverse(chain=true)`. On node exit, `cleanup_entities::<CleanupOnNodeExit>` despawns the
`TetherChainBeam` entities (they have `CleanupOnNodeExit`) but does NOT remove the resource.
If a node ends while chain mode is active (cleared, timer expired, etc.), `TetherChainActive`
persists into the next node. On the next node, `maintain_tether_chain` runs immediately
(guarded only by `resource_exists::<TetherChainActive>`), sees `bolt_count != last_bolt_count`,
and spawns spurious chain beams on a node that never fired chain mode.

**Fix needed**: Remove `TetherChainActive` in an `OnExit(GameState::Playing)` system,
or add `CleanupOnNodeExit` marker to a dedicated entity that removes the resource when despawned,
or remove it alongside the chain beams in the same cleanup path.

**Status**: OPEN — filed as regression spec hint in Wave 3 review 2026-03-30.

## Missing cross-domain ordering: EffectSystems::Recalculate — RESOLVED by cache removal

The Phase 3 issue about `Effective*` components needing `.after(EffectSystems::Recalculate)` is
no longer applicable. The cache-removal refactor (scenario-coverage branch) eliminated all
`Effective*` components and the `recalculate_*` systems entirely. Consumer systems now read
`Active*` directly each frame — no stale-cache risk exists. **Do NOT re-flag this ordering issue.**

## ActiveQuickStops: fire() is no-op when component absent; no consumer reads multiplier — OPEN

`quick_stop::fire()` silently no-ops if `ActiveQuickStops` is absent (unlike all other stat boosts
which lazy-init). Neither `move_breaker` nor `dash::handle_braking` queries `ActiveQuickStops` to
scale their deceleration. `MovementQuery` and `DashQuery` do not include `ActiveQuickStops`.
The `QuickStop` effect fires but its multiplier is never applied to actual deceleration.

**Status**: OPEN — confirmed in cache-removal refactor review 2026-03-30.
Location: `effect/effects/quick_stop.rs` (fire fn), `breaker/queries.rs` (MovementQuery),
`breaker/systems/move_breaker.rs`, `breaker/systems/dash/system.rs`.
