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

## Phase 4 effect systems: shockwave/pulse use Transform (not Position2D) for center

`apply_shockwave_damage` and `apply_pulse_damage` read `transform.translation.truncate()`
as the circle center for quadtree queries. These entities have Transform set at spawn and
never move, so this is functionally correct. But shockwave/ring entities don't have
GlobalPosition2D or Spatial2D — they are purely Transform-based. This is intentional
(they're not `Position2D`-tracked spatial entities).

## Phase 5 effect fire(): Transform vs Position2D for bolt position

`chain_lightning::fire()` — FIXED in rework: now uses `Position2D` directly.

`piercing_beam::fire()` — STILL uses `Position2D -> Transform fallback` (line 37-41). The
Transform fallback is wrong — should be `Position2D -> Vec2::ZERO` only. OPEN.

Impact: ~6px positional error at typical bolt speed (400px/s at 64Hz). Minor but incorrect.

## dispatch_chip_effects: effects dispatched even on max-stack add_chip failure

`dispatch_chip_effects` (chips/systems/dispatch_chip_effects/system.rs) logs a warning when
`add_chip` returns false (chip at max stacks), but does NOT `continue` to skip effect dispatch.
The `for root_effect in &effects` loop runs unconditionally, causing double-application of all
effect trees on the target entities.

Fix: add `continue;` after the warning so effects are not re-dispatched when max stacks is hit.

**Status**: OPEN — still present as of branch feature/source-chip-shield-absorption review.

## apply_pending_bolt_effects: silently drops effects if bolt lacks BoundEffects

`apply_pending_bolt_effects` (scenario-runner/src/lifecycle/systems.rs) queries
`&mut BoundEffects` on tagged bolt entities. If no prior system (e.g. dispatch_breaker_effects)
has inserted `BoundEffects` on the bolt, the query matches zero entities and returns early.
Effects in `PendingBoltEffects` are permanently lost because the `Local<bool>` guard means
the system only successfully applies once (and `pending.0.clear()` is never called).
`spawn_breaker` does NOT insert `BoundEffects` on bolts — confirmed by reading spawn_breaker/system.rs.

Contrast with `apply_pending_cell_effects` and `apply_pending_wall_effects` which correctly
use `commands.entity(entity).insert_if_new((BoundEffects::default(), StagedEffects::default()))`
before extending.

Fix: insert `BoundEffects`+`StagedEffects` via commands before extending, like the cell/wall
variants do. Cannot use `&mut BoundEffects` query directly; needs the insert_if_new + deferred
world callback pattern.

**Status**: OPEN — still present as of branch feature/source-chip-shield-absorption review.

## bypass_menu_to_playing: Target::Breaker initial_effects always dropped

`bypass_menu_to_playing` (scenario-runner/src/lifecycle/systems.rs) runs OnEnter(MainMenu).
It dispatches Target::Breaker initial_effects directly to breaker_query (Query<&mut BoundEffects, With<Breaker>>).
But no Breaker entity exists at MainMenu time (breaker spawns OnEnter(Playing)). Query returns
zero results and breaker-targeted initial_effects are silently dropped with no warning.

Cell, bolt, and wall effects are correctly deferred via PendingCellEffects/PendingBoltEffects/
PendingWallEffects. Breaker effects have no deferred path.

Fix: introduce PendingBreakerEffects resource and apply it in a deferred system like the other
pending-effect systems, after tag_game_entities.

**Status**: OPEN — found in branch feature/source-chip-shield-absorption review.

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
