---
name: AabbMatchesEntityDimensions — breaker Aabb2D never updated on EntityScale change
description: Game bug confirmed in boss_arena_chaos; Aabb2D.half_extents not updated when EntityScale is applied to breaker or bolt
type: project
---

## Bug: Aabb2D.half_extents stale after EntityScale applied

**First detected:** 2026-03-30, boss_arena_chaos scenario (BossArena layout, entity_scale: 0.7)
**Invariant:** AabbMatchesEntityDimensions
**Violation count:** 19997/20000 frames (fires every frame from frame 0)

## Root cause

The checker at `breaker-scenario-runner/src/invariants/checkers/check_aabb_matches_entity_dimensions/checker.rs`
expects:
- Breaker: `aabb.half_extents == Vec2::new(BreakerWidth.half_width() * EntityScale, BreakerHeight.half_height() * EntityScale)`
- Bolt: `aabb.half_extents == Vec2::splat(BoltRadius.0)` (no EntityScale expected for bolt)

The breaker is spawned in `breaker-game/src/breaker/systems/spawn_breaker/system.rs` with:
`Aabb2D::new(Vec2::ZERO, Vec2::new(config.width / 2.0, config.height / 2.0))`

Then `apply_entity_scale_to_breaker` (`breaker-game/src/breaker/systems/apply_entity_scale_to_breaker.rs`)
inserts `EntityScale(layout.0.entity_scale)` on the breaker entity — but there is NO system that
subsequently updates `Aabb2D.half_extents` to reflect the new scale.

Similarly, `apply_entity_scale_to_bolt` inserts `EntityScale` on bolts, but the bolt's `Aabb2D`
is also never updated (though the bolt checker does NOT multiply by EntityScale, so bolt violations
would require a different trigger).

## Systems involved

- Spawn: `breaker-game/src/breaker/systems/spawn_breaker/system.rs` line 63-66
- Scale applied: `breaker-game/src/breaker/systems/apply_entity_scale_to_breaker.rs`
- Visual update: `breaker-game/src/breaker/systems/width_boost_visual.rs` — updates Scale2D but NOT Aabb2D
- Bolt visual: `breaker-game/src/bolt/systems/bolt_scale_visual.rs` — updates Scale2D but NOT Aabb2D
- Checker: `breaker-scenario-runner/src/invariants/checkers/check_aabb_matches_entity_dimensions/checker.rs`

## Missing system

A system that runs in FixedUpdate (or OnEnter(Playing) after apply_entity_scale_to_breaker) that
updates `Aabb2D.half_extents` to match the breaker's effective physical size:
`Vec2::new(BreakerWidth.half_width() * EntityScale, BreakerHeight.half_height() * EntityScale)`

This system should also run when `BreakerWidth` changes (width boost effects).

For bolts, the checker expects `half_extents == BoltRadius` (no EntityScale), so the bolt Aabb2D
does not need to be scaled — but the question of whether bolts should have scaled AABBs for
correct collision is a separate design question (see bolt_wall_collision.rs line 55: it computes
`r = bolt_radius.0 * bolt_scale` independently at runtime).

## Affected layouts

Any layout with `entity_scale != 1.0`. BossArena uses `entity_scale: 0.7`.
