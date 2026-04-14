# Name
TetherBeam

# Enum Variant
- `EffectType::TetherBeam(TetherBeamConfig)`

# Config
`TetherBeamConfig { damage_mult: OrderedFloat<f32>, chain: bool, width: OrderedFloat<f32> }`

# Fire
1. Read the source bolt entity's position.
2. If `chain` is `false`:
   a. Spawn a new bolt entity at the source position with a random velocity direction.
   b. Mark the spawned bolt as `ExtraBolt`.
   c. Spawn a tether beam entity with `TetherBeamSource` tracking the source bolt and the new bolt, and `TetherBeamDamage(damage_mult.0)` (the raw multiplier — the tick system applies it to base damage at tick time), and `TetherBeamWidth(width.0)`.
3. If `chain` is `true`:
   a. Find the nearest other bolt entity to the source bolt.
   b. If found, spawn a tether beam entity with `TetherBeamSource` tracking the source bolt and the nearest bolt, and `TetherBeamDamage(damage_mult.0)`, and `TetherBeamWidth(width.0)`.
   c. If no other bolt is found, do nothing.
4. All spawned tether beam entities get `CleanupOnExit<NodeState>` as safety net.
5. Fire does NOT tick the beam -- `tick_tether_beam_damage` does.

# Reverse
Not reversible.

# Source Location
`src/effect_v3/effects/tether_beam/config.rs`

# New Types
- `TetherBeamSource` -- component tracking the two bolt endpoint entities (bolt_a: Entity, bolt_b: Entity)
- `TetherBeamDamage(f32)` -- the damage multiplier; the tick system applies `TetherBeamDamage * BoltBaseDamage` per tick
- `TetherBeamWidth(f32)` -- half-width of the beam line; cells whose perpendicular distance to the beam is <= this value are damaged each tick

# New Systems

## tick_tether_beam_damage
- **What it does**: For each entity with `TetherBeamSource`, read positions of both endpoint bolts. Calculate the beam line segment between them. Query for cells within `TetherBeamWidth` perpendicular distance of the line. Send `DamageDealt<Cell>` for each intersecting cell with damage computed as `TetherBeamDamage * BoltBaseDamage`.
- **What it does NOT do**: Does not despawn beams. Does not check if endpoints still exist (cleanup system does that).
- **Schedule**: FixedUpdate, in `EffectV3Systems::Tick`, with `run_if(in_state(NodeState::Playing))`.

## cleanup_tether_beams
- **What it does**: For each entity with `TetherBeamSource`, check if either endpoint bolt entity has been despawned. If either endpoint is gone, despawn the beam entity.
- **What it does NOT do**: Does not deal damage. Does not modify beam endpoints.
- **Schedule**: FixedUpdate, after `tick_tether_beam_damage`.
