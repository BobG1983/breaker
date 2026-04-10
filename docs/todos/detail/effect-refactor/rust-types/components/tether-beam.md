# Name
TetherBeamSource, TetherBeamDamage

# Struct
```rust
/// Identifies a tether beam entity and the two bolts it connects.
#[derive(Component)]
pub struct TetherBeamSource {
    /// First bolt endpoint.
    pub bolt_a: Entity,
    /// Second bolt endpoint.
    pub bolt_b: Entity,
}

/// Damage dealt per tick to cells that cross the tether beam.
#[derive(Component)]
pub struct TetherBeamDamage(pub f32);
```

# Location
`src/effect/effects/tether_beam/`

# Description
These components form the tether beam effect entity. A tether beam is a damage-dealing line between two bolts that hurts cells passing through it.

- **Spawned by**: `TetherBeamConfig.fire()` creates a new entity with both components, linking two bolt entities.
- **Tick**: The tether beam system checks for cells intersecting the line between `bolt_a` and `bolt_b` each frame and applies `TetherBeamDamage` to them.
- **Despawned by**: `cleanup_tether_beams` removes the entity when either endpoint entity (`bolt_a` or `bolt_b`) no longer exists.
