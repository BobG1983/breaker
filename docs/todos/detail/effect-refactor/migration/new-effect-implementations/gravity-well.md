# Name
GravityWell

# Enum Variant
- `EffectType::GravityWell(GravityWellConfig)`

# Config
`GravityWellConfig { strength: f32, duration: f32, radius: f32, max: u32 }`

# Fire
1. Read the source entity's position.
2. Query for existing `GravityWellSource` entities with `GravityWellOwner` matching the source entity.
3. If the count of active wells >= `max`, despawn the oldest well (by remaining `GravityWellLifetime`).
4. Spawn a gravity well entity at the source position with:
   - `GravityWellSource` marker
   - `GravityWellStrength(config.strength)`
   - `GravityWellRadius(config.radius)`
   - `GravityWellLifetime(config.duration)`
   - `GravityWellOwner(source_entity)`
   - `CleanupOnExit<NodeState>`
5. Fire does NOT pull bolts -- `tick_gravity_wells` does.

# Reverse
Not reversible.

# Source Location
`src/effect/effects/gravity_well/config.rs`

# New Types
- `GravityWellSource` -- marker component identifying gravity well entities
- `GravityWellStrength(f32)` -- how strongly bolts are pulled toward the well center per tick
- `GravityWellRadius(f32)` -- how far from the well center bolts are affected
- `GravityWellLifetime(f32)` -- remaining lifetime in seconds
- `GravityWellOwner(Entity)` -- the source entity that spawned this well

# New Systems

## tick_gravity_wells
- **What it does**: For each entity with `GravityWellSource`, query for bolt entities within `GravityWellRadius`. For each bolt, apply a force toward the well center scaled by `GravityWellStrength`. Decrement `GravityWellLifetime` by `dt`.
- **What it does NOT do**: Does not despawn wells. Does not spawn wells.
- **Schedule**: FixedUpdate, in `EffectSystems::Tick`, with `run_if(in_state(NodeState::Playing))`.

## despawn_expired_wells
- **What it does**: For each entity with `GravityWellSource`, if `GravityWellLifetime <= 0.0`, despawn the entity.
- **What it does NOT do**: Does not apply forces. Does not modify bolt velocities.
- **Schedule**: FixedUpdate, after `tick_gravity_wells`.
