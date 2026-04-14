# Name
SpawnBolts

# Enum Variant
- `EffectType::SpawnBolts(SpawnBoltsConfig)`

# Config
`SpawnBoltsConfig { count: u32, lifespan: Option<f32>, inherit: bool }`

# Fire
1. Read the source entity's position.
2. Spawn `count` new bolt entities at the source position, each with a random velocity direction.
3. Mark each spawned bolt as `ExtraBolt`.
4. If `lifespan` is `Some(duration)`, attach a lifespan timer to each spawned bolt.
5. If `inherit` is `true`, copy the first primary bolt's `BoundEffects` onto each spawned bolt.
6. Fire does NOT manage bolt lifetime -- `tick_bolt_lifespan` does.
7. Fire does NOT inherit `StagedEffects` -- only `BoundEffects`.

# Reverse
Not reversible.

# Source Location
`src/effect_v3/effects/spawn_bolts/config.rs`

# New Types
None -- uses existing bolt builder and existing `ExtraBolt` marker.

# New Systems
None new -- existing bolt systems handle extra bolts, and existing `tick_bolt_lifespan` handles lifespan timers.
