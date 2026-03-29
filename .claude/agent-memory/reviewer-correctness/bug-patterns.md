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
