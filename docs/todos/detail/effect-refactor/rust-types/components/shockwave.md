# Name
ShockwaveSource, ShockwaveRadius, ShockwaveMaxRadius, ShockwaveSpeed, ShockwaveDamaged, ShockwaveBaseDamage, ShockwaveDamageMultiplier

# Struct
```rust
/// Marker component identifying an entity as a shockwave source.
#[derive(Component)]
pub struct ShockwaveSource;

/// Current expanding radius of the shockwave.
#[derive(Component)]
pub struct ShockwaveRadius(pub f32);

/// Maximum radius the shockwave can reach before despawning.
#[derive(Component)]
pub struct ShockwaveMaxRadius(pub f32);

/// Expansion speed of the shockwave in units per second.
#[derive(Component)]
pub struct ShockwaveSpeed(pub f32);

/// Set of entities already damaged by this shockwave (prevents double-hit).
#[derive(Component)]
pub struct ShockwaveDamaged(pub HashSet<Entity>);

/// Base damage dealt by the shockwave before multipliers.
#[derive(Component)]
pub struct ShockwaveBaseDamage(pub f32);

/// Multiplier applied to shockwave damage (from stacking or upgrades).
#[derive(Component)]
pub struct ShockwaveDamageMultiplier(pub f32);
```

# Location
`src/effect_v3/effects/shockwave/`

# Description
These components form the shockwave effect entity bundle. A shockwave is a radially expanding area-of-effect that damages cells it reaches.

- **Spawned by**: `ShockwaveConfig.fire()` — creates a new entity with all seven components.
- **Tick**: `tick_shockwave` increases `ShockwaveRadius` each frame based on `ShockwaveSpeed`. Cells within `ShockwaveRadius` that are not in `ShockwaveDamaged` take damage and get added to the set.
- **Despawned by**: `despawn_finished_shockwave` — removes the entity when `ShockwaveRadius >= ShockwaveMaxRadius`.
