# MirrorProtocol

Spawn mirrored bolts that inherit the primary bolt's effects (not the breaker's). On perfect bump, spawns bolts at mirrored angles from the primary bolt's trajectory.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `count` | `u32` | Number of mirrored bolts to spawn |
| `inherit_from_primary` | `bool` | If true, copies BoundEffects from the primary bolt entity (not the triggering breaker) |

## Behavior

1. Triggered on `PerfectBump` (breaker-level trigger)
2. Find the primary bolt entity
3. Copy the primary bolt's `BoundEffects` (piercing, speed boosts, damage boosts, etc.)
4. Spawn `count` new bolts at the breaker's position
5. Set their velocities to mirrored angles from the primary bolt's trajectory (symmetrically spread)
6. Attach the copied `BoundEffects` to the spawned bolts

Key difference from `SpawnBolts(inherit: true)`: SpawnBolts inherits from the source entity (which is the breaker for breaker-triggered effects). MirrorProtocol specifically finds the primary bolt and inherits from IT, ensuring piercing/speed/damage effects transfer correctly.

## Reversal

No-op. Spawned bolts have their own lifecycle.

## Ingredients

Reflex x1 + Piercing Shot x2.

## VFX

- On trigger: brief prismatic flash at breaker position
- Spawned bolts emerge at mirrored angles with prismatic birth trails
- Expert players can control the spread via bump angle — three bolts spreading in controlled directions for surgical field coverage
