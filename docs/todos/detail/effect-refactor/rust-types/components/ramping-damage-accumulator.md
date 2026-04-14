# Name
RampingDamageAccumulator

# Struct
```rust
/// Accumulated bonus damage from consecutive hits without missing.
#[derive(Component)]
pub struct RampingDamageAccumulator(pub OrderedFloat<f32>);
```

# Location
`src/effect_v3/effects/ramping_damage/`

# Description
`RampingDamageAccumulator` is added to the bolt to track bonus damage that increases with consecutive cell hits.

- **Added by**: `RampingDamageConfig.fire()` inserts the component if absent (initialized to zero). Does not overwrite if already present.
- **Tick**: Each bolt-cell bump adds to the accumulator based on the ramping damage configuration. The accumulated value is added to the bolt's effective damage on each hit.
- **Reset**: `reset_ramping_damage` resets the accumulator to zero at the start of each node to prevent carry-over.
- **Removed by**: `RampingDamageConfig.reverse()` removes the component only if the effect stack is empty (no remaining ramping damage sources).
