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

## gravity_well::fire() infinite loop when max == 0 — FIXED

`gravity_well::fire()` was previously missing the `max == 0` guard. Now has both
`if world.get_entity(entity).is_err() { return; }` AND `if max == 0 { return; }` guards
at lines 47-53. `spawn_phantom::fire()` also has the despawned-entity guard at line 46.

**Status**: FIXED — confirmed on feature/chip-evolution-ecosystem branch 2026-03-31.

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

## TetherChainActive resource lifecycle — RESOLVED as of 2026-03-31

Previously flagged as OPEN: resource could leak across node boundaries.

Analysis on feature/chip-evolution-ecosystem branch: `reverse(chain=true)` correctly calls
`world.remove_resource::<TetherChainActive>()` when the chip effect expires. `cleanup_tether_chain_resource`
runs `OnExit(GameState::Playing)` as a safety net. `TetherChainBeam` entities have `CleanupOnNodeExit`.

Within a run, the resource intentionally persists across nodes so `maintain_tether_chain` can
rebuild beams each time bolt count changes. The primary bolt's `BoundEffects` is `CleanupOnRunEnd`
so chain mode re-fires on each node start, which is correct behavior.

**Status**: RESOLVED — no leak. The open concern was based on incomplete analysis of the `reverse()` path.

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

## circuit_breaker::fire() u32 underflow when bumps_required == 0 — OPEN latent bug

`circuit_breaker/effect.rs:73`: `let remaining = config.bumps_required - 1` where `bumps_required`
is `u32`. If `bumps_required == 0`, this panics in debug or wraps to `u32::MAX` in release.
Current production RON (`circuit_breaker.evolution.ron`) uses `bumps_required: 3` so no current
trigger. No guard against 0 exists. No test covers this case.

**Status**: OPEN — latent. Not triggered by current data. Needs `bumps_required == 0` guard.
Location: `breaker-game/src/effect/effects/circuit_breaker/effect.rs:73`

## MirrorProtocol::fire() wastes RNG call with dead random velocity — latent design issue

`mirror_protocol/effect.rs:73-78`: generates `random_velocity` via `rng.random_range(0..TAU)` and
passes it to bolt builder, but line 85 immediately overwrites with `mirror_vel`. The random call
advances RNG state for no observable effect. Other systems sharing `GameRng` will see different
values if MirrorProtocol fires. Not a correctness bug for the mirror bolt itself, but affects
RNG determinism for other game systems.

**Status**: OPEN design issue — the extra RNG call has no gameplay effect on the mirror bolt.

## BoltBuilder::build() silently drops OptionalBoltData — latent API hazard, not active bug

The four `build()` terminal impls in `breaker-game/src/bolt/builder.rs` (lines 376, 408, 440, 472)
include role+cleanup+serving markers in the returned tuple but silently drop: `bolt_params`,
`spawned_by`, `lifespan`, `with_effects`, `inherited_effects`.

**This is NOT an active bug**: grep confirms `build()` has zero callers in production code.
All production sites use `spawn()` which processes `OptionalBoltData` via `spawn_inner()`.

Tests in builder.rs section E test `build()` but only test paths where `optional.bolt_params`
is either None (no-config path tested in `build_without_from_config_has_no_bolt_params`) or set
via `config()` (which does populate `bolt_params`). No test calls `.with_lifespan(x).build()`
to verify lifespan is (or isn't) on the entity — this test gap is acceptable since `build()`
is unused in production.

**Status**: Latent hazard — safe to leave as-is unless `build()` gains production callers.

## .definition() builder omits BoltRespawnOffsetY and BoltRespawnAngleSpread — LATENT BUG

`spawn_inner` (core.rs:390-397) inserts `BoltAngleSpread` + `BoltSpawnOffsetY` from `definition_params`
but does NOT insert `BoltRespawnOffsetY` or `BoltRespawnAngleSpread`. The `LostBoltData` query
(queries.rs:90-92) requires BOTH `BoltRespawnOffsetY` and `BoltRespawnAngleSpread` as non-optional
components. Bolts built via `.definition()` would be silently excluded from `bolt_lost` query
and never detected as lost.

**Current status**: LATENT — `.definition()` is not yet wired into `spawn_bolt`. All production
bolt spawns use `.config()`. `BASE_BOLT_DAMAGE` comment in resources.rs:9 confirms wiring is
deferred to Wave 6. If wave 6 wires `spawn_bolt` to use `.definition()`, this will become an
active bug: primary bolt falls below playfield but no BoltLost is ever emitted.

**Location**: `breaker-game/src/bolt/builder/core.rs:390-396` (spawn_inner, definition_params block)
**Also affected**: `breaker-game/src/bolt/queries.rs:90-92` (LostBoltData requires BoltRespawnOffsetY, BoltRespawnAngleSpread)

**Fix**: Add `BoltRespawnOffsetY` and `BoltRespawnAngleSpread` to the `definition_params` insertion block,
using values from the definition or falling back to constants (e.g., `DEFAULT_BOLT_SPAWN_OFFSET_Y`
and `DEFAULT_BOLT_ANGLE_SPREAD`).
