---
name: bolt-message-pattern-map
description: Bolt domain message status (SpawnAdditionalBolt placeholder), ExtraBolt/phantom/lifespan patterns, and DistanceConstraint wiring for ChainBolt
type: reference
---

# Bolt Message & Pattern Map

## SpawnAdditionalBolt Message — Status: PLACEHOLDER

`SpawnAdditionalBolt` is defined in `src/bolt/messages.rs`:
```rust
pub struct SpawnAdditionalBolt {
    pub source_chip: Option<String>,
    pub lifespan: Option<f32>,
    pub inherit: bool,
}
```

The docstring says "Consumed by `handle_spawn_bolt` in the effect domain" but that handler
**does not exist**. The effect handler in `src/effect/effects/spawn_bolts.rs::fire()` is a
placeholder — it just logs, does not write `SpawnAdditionalBolt` or spawn actual bolt entities.

Note: `SpawnChainBolt` was removed from `messages.rs`. The `chain_bolt.rs::fire()` effect
directly spawns full bolt entities with all required components (`ChainBoltMarker`, `Position2D`,
`Velocity2D`, `Aabb2D`, `CollisionLayers`, `BoltBaseSpeed`, etc.) plus a `DistanceConstraint`
entity referencing the chain bolt and the anchor entity. This is the established direct-spawn
pattern (see `reviewer-architecture` memory).

## Phantom Bolt — Status: PLACEHOLDER

`src/effect/effects/spawn_phantom.rs` exists but is NOT the ECS-bolt approach.
It spawns `(PhantomBoltMarker, PhantomTimer, PhantomOwner, Transform)` — no
`Bolt` marker, no physics components. Reads `Transform` (NOT `Position2D`).
Uses `PhantomTimer(f32)` float instead of `BoltLifespan(Timer)`.

This is the OLD placeholder approach. The PLANNED approach (for SpawnPhantom spec work)
would be a real bolt entity with:
- All bolt spawn components (Bolt, Position2D, Velocity2D, Aabb2D, CollisionLayers, ...)
- `ExtraBolt` marker (so it despawns on loss rather than respawning)
- `BoltLifespan(Timer::from_seconds(duration, TimerMode::Once))`
- `PiercingRemaining(u32::MAX)` — or equivalent infinite piercing
- `CleanupOnNodeExit` (NOT `CleanupOnRunEnd` — phantom dies with the node)

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
