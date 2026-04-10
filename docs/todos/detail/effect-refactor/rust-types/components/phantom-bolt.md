# Name
PhantomBolt, PhantomLifetime, PhantomOwner

# Struct
```rust
/// Marker identifying an entity as a phantom bolt.
#[derive(Component)]
pub struct PhantomBolt;

/// Remaining lifetime in seconds before the phantom bolt despawns.
#[derive(Component)]
pub struct PhantomLifetime(pub f32);

/// Entity that spawned this phantom bolt (for ownership tracking).
#[derive(Component)]
pub struct PhantomOwner(pub Entity);
```

# Location
`src/effect/effects/spawn_phantom/`

# Description
These components form the phantom bolt effect entity. A phantom bolt is a temporary secondary bolt spawned from the primary bolt's position.

- **Spawned by**: `SpawnPhantomConfig.fire()` creates a new bolt entity with all three components plus standard bolt components, launched from the source bolt's position.
- **Tick**: `tick_phantom_lifetime` decrements `PhantomLifetime` each frame.
- **Despawned by**: `tick_phantom_lifetime` removes the entity when `PhantomLifetime <= 0.0`.
