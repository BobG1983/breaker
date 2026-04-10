# Name
GravityWellSource, GravityWellStrength, GravityWellRadius, GravityWellLifetime, GravityWellOwner

# Struct
```rust
/// Marker identifying an entity as a gravity well source.
#[derive(Component)]
pub struct GravityWellSource;

/// Attractive force magnitude of the gravity well.
#[derive(Component)]
pub struct GravityWellStrength(pub f32);

/// Radius of the gravity well's influence area.
#[derive(Component)]
pub struct GravityWellRadius(pub f32);

/// Remaining lifetime in seconds before the well despawns.
#[derive(Component)]
pub struct GravityWellLifetime(pub f32);

/// Entity that spawned this gravity well (for ownership tracking).
#[derive(Component)]
pub struct GravityWellOwner(pub Entity);
```

# Location
`src/effect/effects/gravity_well/`

# Description
These components form the gravity well effect entity bundle. A gravity well is a point attractor that pulls bolts toward it.

- **Spawned by**: `GravityWellConfig.fire()` creates a new entity at the target position with all five components.
- **Tick**: The gravity well system applies force to bolts within `GravityWellRadius` based on `GravityWellStrength`. `GravityWellLifetime` is decremented each frame.
- **Despawned by**: `despawn_expired_wells` removes the entity when `GravityWellLifetime <= 0.0`.
