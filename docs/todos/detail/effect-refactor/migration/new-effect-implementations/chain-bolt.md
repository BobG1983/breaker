# Name
ChainBolt

# Enum Variant
- `EffectType::ChainBolt(ChainBoltConfig)`

# Config
`ChainBoltConfig { tether_distance: f32 }`

# Fire
1. Read the source bolt entity's position and velocity.
2. Spawn a new bolt entity at the source position with the negated source velocity (`Vec2::new(-vel.x, -vel.y)`).
3. Mark the spawned bolt as `ExtraBolt`.
4. Create a `DistanceConstraint` from `rantzsoft_physics2d` between the source bolt and the spawned bolt with `max_distance` set to `config.tether_distance`.
5. Fire does NOT enforce the tether -- `DistanceConstraint` from `rantzsoft_physics2d` does.

# Reverse
Not reversible.

# Source Location
`src/effect_v3/effects/chain_bolt/config.rs`

# New Types
None -- uses existing bolt builder and `DistanceConstraint` from `rantzsoft_physics2d`.

# New Systems
None new -- `rantzsoft_physics2d` handles the distance constraint enforcement.
