# Name
SpawnPhantom

# Enum Variant
- `EffectType::SpawnPhantom(SpawnPhantomConfig)`

# Config
`SpawnPhantomConfig { duration: f32, max_active: u32 }`

# Fire
1. Read the source entity's position.
2. Query for existing `PhantomBolt` entities with `PhantomOwner` matching the source entity.
3. If the count of active phantoms >= `max_active`, despawn the oldest phantom (by remaining `PhantomLifetime`).
4. Spawn a phantom bolt entity at the source position with:
   - `PhantomBolt` marker
   - `PhantomLifetime(config.duration)`
   - `PhantomOwner(source_entity)`
   - Infinite piercing (passes through all cells without bouncing)
   - A distinct visual (phantom appearance)
   - `CleanupOnExit<NodeState>`
5. Fire does NOT manage phantom lifetime -- `tick_phantom_lifetime` does.

# Reverse
Not reversible.

# Source Location
`src/effect/effects/phantom_bolt/config.rs`

# New Types
- `PhantomBolt` -- marker component identifying phantom bolt entities
- `PhantomLifetime(f32)` -- remaining lifetime in seconds
- `PhantomOwner(Entity)` -- the source entity that spawned this phantom

# New Systems

## tick_phantom_lifetime
- **What it does**: For each entity with `PhantomBolt`, decrement `PhantomLifetime` by `dt`. If `PhantomLifetime <= 0.0`, despawn the entity.
- **What it does NOT do**: Does not spawn phantoms. Does not cap active phantom count (fire does that).
- **Schedule**: FixedUpdate, in `EffectSystems::Tick`, with `run_if(in_state(NodeState::Playing))`.
