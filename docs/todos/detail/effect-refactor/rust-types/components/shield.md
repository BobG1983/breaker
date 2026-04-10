# Name
ShieldWall, ShieldOwner, ShieldDuration, ShieldReflectionCost

# Struct
```rust
/// Marker identifying an entity as a shield wall.
#[derive(Component)]
pub struct ShieldWall;

/// Entity that owns this shield (for cleanup on reverse).
#[derive(Component)]
pub struct ShieldOwner(pub Entity);

/// Remaining duration in seconds before the shield despawns.
#[derive(Component)]
pub struct ShieldDuration(pub f32);

/// Duration cost consumed from the shield when a bolt reflects off it.
#[derive(Component)]
pub struct ShieldReflectionCost(pub f32);
```

# Location
`src/effect/effects/shield/`

# Description
These components form the shield effect entity bundle. A shield is a temporary barrier that reflects bolts and protects the lower playfield.

- **Spawned by**: `ShieldConfig.fire()` creates a new shield entity with all four components positioned above the death zone.
- **Tick**: `tick_shield_duration` decrements `ShieldDuration` each frame. When a bolt reflects off the shield, `ShieldReflectionCost` is subtracted from `ShieldDuration`.
- **Despawned by**: `tick_shield_duration` removes the entity when `ShieldDuration <= 0.0`.
- **Reverse**: `ShieldConfig.reverse()` finds and despawns all shield entities matching the `ShieldOwner`.
