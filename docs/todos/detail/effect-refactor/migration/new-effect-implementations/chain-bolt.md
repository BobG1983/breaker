# Name
ChainBolt

# Enum Variant
- `EffectType::ChainBolt(ChainBoltConfig)`

# Config
`ChainBoltConfig { tether_distance: f32 }`

# Fire
1. Read the source bolt entity's position.
2. Spawn a new bolt entity at the source position with a random velocity direction.
3. Mark the spawned bolt as `ExtraBolt`.
4. Create a `DistanceConstraint` from `rantzsoft_physics2d` between the source bolt and the spawned bolt with `max_distance` set to `config.tether_distance`.
5. Fire does NOT enforce the tether -- `DistanceConstraint` from `rantzsoft_physics2d` does.

# Reverse
Not reversible.

# Source Location
`src/effect/configs/chain_bolt.rs`

# New Types
None -- uses existing bolt builder and `DistanceConstraint` from `rantzsoft_physics2d`.

# New Systems
None new -- `rantzsoft_physics2d` handles the distance constraint enforcement.
