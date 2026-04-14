# Name
SecondWindWall, SecondWindOwner

# Struct
```rust
/// Marker identifying an entity as a second-wind wall.
#[derive(Component)]
pub struct SecondWindWall;

/// Entity that owns this second-wind wall (for cleanup on reverse).
#[derive(Component)]
pub struct SecondWindOwner(pub Entity);
```

# Location
`src/effect_v3/effects/second_wind/`

# Description
These components form the second-wind effect entity. A second-wind wall is a one-use safety net that catches the first bolt that would otherwise be lost.

- **Spawned by**: `SecondWindConfig.fire()` creates a new entity with both components positioned at the death zone boundary.
- **Consumed by**: On the first bolt bounce against the wall, the second-wind wall self-destructs via `Die` -- it is a single-use effect.
- **Reverse**: `SecondWindConfig.reverse()` finds and despawns all second-wind entities matching the `SecondWindOwner`.
