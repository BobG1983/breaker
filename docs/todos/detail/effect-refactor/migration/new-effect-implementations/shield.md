# Name
Shield

# Enum Variant
- `EffectType::Shield(ShieldConfig)`
- `ReversibleEffectType::Shield(ShieldConfig)`

# Config
`ShieldConfig { duration: OrderedFloat<f32>, reflection_cost: OrderedFloat<f32> }`

# Fire
1. Spawn a visible `ShieldWall` entity at the bottom of the playfield.
2. Insert `ShieldDuration` initialized to `duration` (ticks down over time).
3. Insert `ShieldReflectionCost` initialized to `reflection_cost`.
4. Insert `ShieldOwner(entity)` so reverse can find it.
   Insert `CleanupOnExit<NodeState>` as safety net.
5. Fire does NOT handle bolt reflections -- that is the wall collision system's job.
6. Fire does NOT tick the timer -- `tick_shield_duration` does.

# Reverse
1. Find all `ShieldWall` entities with `ShieldOwner` matching the target entity.
2. Despawn them.

# Source Location
`src/effect_v3/effects/shield/config.rs`

# New Types
- `ShieldWall` -- marker component for the shield wall entity.
- `ShieldOwner(Entity)` -- component linking a shield wall back to the entity that spawned it.
- `ShieldDuration(f32)` -- component tracking remaining shield time in seconds.
- `ShieldReflectionCost(f32)` -- component storing the cost in seconds subtracted from duration on each bolt bounce.

# New Systems

## tick_shield_duration
- **What it does**: For each entity with `ShieldWall`, decrement `ShieldDuration` by `dt`. Read bolt-wall collision messages — for each bounce involving this shield wall, subtract `ShieldReflectionCost` from `ShieldDuration`. When `ShieldDuration` reaches 0 or below, despawn the shield wall entity.
- **What it does NOT do**: Does not handle bolt physics. Does not reflect bolts. Does not modify bolt velocity.
- **Schedule**: FixedUpdate, after wall collision systems.
