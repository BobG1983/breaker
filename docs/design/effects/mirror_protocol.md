# MirrorProtocol

Spawn a mirrored bolt from any bolt on impact. The mirror position and velocity depend on which side of the collider was hit — creating a symmetric "reflection" that goes the opposite direction on the appropriate axis.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `inherit` | `bool` | If true, copies BoundEffects from the bolt entity |

## Prerequisites

Bolts must have a `LastImpact` component updated by collision systems on rebound (not pierce-through):

```
LastImpact { position: Vec2, side: ImpactSide }
ImpactSide: Top, Bottom, Left, Right
```

Stamped by `bolt_breaker_collision`, `bolt_cell_collision`, `bolt_wall_collision` on rebound.

## Behavior

1. Target is the **bolt entity** (trigger-agnostic — works with PerfectBumped, CellDestroyed, any trigger)
2. If entity is not a Bolt → noop
3. If `LastImpact` is absent or missing → noop
4. Read `LastImpact { position, side }`, current `Position2D`, and `Velocity2D` from the bolt
5. Compute mirror based on impact side:
   - **Top or Bottom** (horizontal surface): flip X — `mirror_pos = (2 * impact.x - bolt.x, bolt.y)`, `mirror_vel = (-vel.x, vel.y)`
   - **Left or Right** (vertical surface): flip Y — `mirror_pos = (bolt.x, 2 * impact.y - bolt.y)`, `mirror_vel = (vel.x, -vel.y)`
6. If `inherit: true`, clone the bolt's `BoundEffects`
7. Spawn 1 bolt at mirror position with mirror velocity

Key difference from `SpawnBolts`: MirrorProtocol spawns at a geometrically mirrored position based on the bolt's last impact, with side-dependent axis mirroring. SpawnBolts spawns at the source entity's position with random velocity spread.

## Reversal

No-op. Spawned bolts have their own lifecycle.

## Ingredients

Reflex x1 + Piercing Shot x2.

## VFX

- On trigger: brief prismatic flash at the bolt's impact point
- Mirrored bolt emerges from the flash with prismatic birth trail
- The mirror direction (horizontal or vertical) is visually readable from the flash orientation
