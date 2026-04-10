# Name
SecondWind

# Enum Variant
- `EffectType::SecondWind(SecondWindConfig)`
- `ReversibleEffectType::SecondWind(SecondWindConfig)`

# Config
`SecondWindConfig {}` (empty struct)

# Fire
1. Spawn an invisible one-shot wall entity at the bottom of the playfield.
2. Insert `SecondWindWall` marker component on the wall.
3. Insert `SecondWindOwner(entity)` so reverse can find it.
   Insert `CleanupOnExit<NodeState>` as safety net.
4. The wall has no visual and is consumed after one bolt bounce -- the wall collision system sends the bounce, then the wall self-destructs via `Fire(Die)`.
5. Fire does NOT handle the bounce -- that is the wall collision system's job.
6. Fire does NOT make the wall visible.

# Reverse
1. Find all `SecondWindWall` entities with `SecondWindOwner` matching the target entity.
2. Despawn them.

# Source Location
`src/effect/effects/second_wind/config.rs`

# New Types
- `SecondWindWall` -- marker component for the one-shot safety net wall.
- `SecondWindOwner(Entity)` -- component linking a second-wind wall back to the entity that spawned it.

# New Systems
None -- wall collision handles the bounce, die handles the despawn.
