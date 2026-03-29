---
name: bolt-message-pattern-map
description: Bolt domain message inventory, ExtraBolt/phantom/lifespan patterns, and DistanceConstraint wiring for ChainBolt
type: reference
---

# Bolt Message & Pattern Map

## Bolt Domain Messages (src/bolt/messages.rs)

Current messages as of feature/runtime-effects:
- `BoltSpawned` — sent after bolt entity spawns; consumed by node spawn coordinator
- `BoltImpactBreaker` — bolt hits breaker; consumed by `grade_bump`
- `BoltImpactCell` — bolt hits cell; consumed by chips, cells, audio
- `BoltLost` — bolt falls below breaker; consumed by breaker plugin (penalty)
- `BoltImpactWall` — bolt reflects off wall; consumed by overclock wall impact bridge
- `RequestBoltDestroyed` — extra bolt falls off screen; consumed by `bridge_bolt_death` and `cleanup_destroyed_bolts`

`SpawnAdditionalBolt` was removed from `messages.rs`. `SpawnChainBolt` was also removed.
Effect `fire()` functions spawn bolt entities directly (direct-spawn pattern).

## Phantom Bolt — Status: REAL (feature/runtime-effects)

`src/effect/effects/spawn_phantom/effect.rs` uses the real bolt approach via `spawn_extra_bolt`:
- Calls `spawn_extra_bolt(world, spawn_pos)` which spawns a full bolt entity with all physics components
- Then inserts `(PhantomBoltMarker, PhantomOwner(entity), BoltLifespan(Timer), PiercingRemaining(u32::MAX))`
- Uses `Position2D` (not `Transform`) for spawn position
- `PhantomTimer` component is gone — replaced by `BoltLifespan`

The old placeholder components (`PhantomTimer`, `Transform`-based spawn) are fully removed from the codebase.

## Extra Bolt Pattern (from ExtraBolt marker)

`ExtraBolt` component marks bolt-like entities that:
- Are despawned on loss (via `RequestBoltDestroyed`) NOT respawned
- Have `CleanupOnNodeExit` or handled by bolt_lost
- Used by `LostQuery` to differentiate behavior

## BoltLifespan — Already Implemented

`BoltLifespan(Timer)` is a real component in `src/bolt/components.rs`.
`tick_bolt_lifespan` system in `FixedUpdate` ticks it and writes `RequestBoltDestroyed`
when `just_finished()`. This is a complete, tested, working lifespan system.

## DistanceConstraint Wiring

Component in `rantzsoft_physics2d::constraint::DistanceConstraint`:
```rust
pub struct DistanceConstraint {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub max_distance: f32,
}
```

The constraint is placed on a **third entity** (not on either constrained entity).
`enforce_distance_constraints` queries `Query<&DistanceConstraint>` separately from
`Query<(&mut Position2D, &mut Velocity2D)>` via `get_many_mut`.

System runs in `FixedUpdate` in set `PhysicsSystems::EnforceDistanceConstraints`
(registered by `RantzPhysics2dPlugin`, after `PhysicsSystems::MaintainQuadtree`).

`bolt_lost` runs `.after(PhysicsSystems::EnforceDistanceConstraints)` — the constraint
solver runs first, then loss detection.

Solver behavior:
- If distance <= max_distance: no-op (slack)
- If distance > max_distance (taut): symmetric position correction (each moves half the overshoot)
- Velocity redistribution along constraint axis unless both entities are actively converging
- Perpendicular velocity preserved unchanged
- Missing entity: gracefully skipped (`get_many_mut` returns Err)

For ChainBolt: spawn a DistanceConstraint entity referencing chain_bolt_entity and anchor_entity.
Both constrained entities need `Position2D` and `Velocity2D`.
