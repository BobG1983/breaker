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

**Status**: OPEN — filed in 2026-03-30 review of full-verification-fixes branch.

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

## Missing cross-domain ordering: EffectSystems::Recalculate before consumer systems

Confirmed in Phase 3 review. The bolt/breaker consumer systems (`prepare_bolt_velocity`,
`bolt_cell_collision`, `bolt_breaker_collision`, `move_breaker`) read `Effective*`
components but have NO `.after(EffectSystems::Recalculate)` constraint. When an
effect fires via Bridge and Recalculate updates the Effective* value, the consumer
systems may have already run that frame — producing a 1-frame stale value.

**Reproduces as**: bolt speed/damage/size does not immediately reflect the new
multiplier in the same frame the effect fires.

**Location**: `bolt/plugin.rs` (PrepareVelocity, CellCollision, BreakerCollision sets)
and `breaker/plugin.rs` (Move set). Fix: add `.after(EffectSystems::Recalculate)`
to those system registrations.

**Status**: OPEN — filed in Phase 3 review.
