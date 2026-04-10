# Name
EntropyCounter

# Struct
```rust
/// Tracks bump counts toward randomly firing an effect from a weighted pool.
#[derive(Component)]
pub struct EntropyCounter {
    /// Current bump count since last trigger.
    pub count: u32,
    /// Maximum number of effects that can fire per node.
    pub max_effects: u32,
    /// Weighted pool of effects to choose from. Each entry is (weight, effect_type).
    pub pool: Vec<(OrderedFloat<f32>, Box<EffectType>)>,
}
```

# Location
`src/effect/effects/entropy_engine/`

# Description
`EntropyCounter` is added to the bolt to implement the entropy engine mechanic -- a random effect trigger based on bump accumulation.

- **Added by**: `EntropyEngineConfig.fire()` inserts the component on first activation with the configured pool and max.
- **Tick**: Each bolt-cell bump increments `count`. When the threshold is reached, a weighted random selection from `pool` fires the chosen effect. `count` resets after each trigger.
- **Reset**: `reset_entropy_counter` resets `count` to zero at the start of each node, preventing carry-over between levels.
- **Removed by**: `EntropyEngineConfig.reverse()` removes the component entirely.
