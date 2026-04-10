# Name
EffectTimers

# Struct
```rust
/// Collection of active effect duration timers on an entity.
/// Each entry is a (remaining_time, original_duration) pair.
#[derive(Component)]
pub struct EffectTimers {
    pub timers: Vec<(OrderedFloat<f32>, OrderedFloat<f32>)>,
}
```

# Location
`src/effect/triggers/time/`

# Description
`EffectTimers` holds countdown timers for time-limited effects installed via `Until(TimeExpires(N))` conditions in the effect tree.

- **Added by**: The tree walker inserts this component (or pushes a new entry) when installing an effect with a `TimeExpires` condition. The first element of each tuple is the remaining time; the second is the original duration (for UI display or reset).
- **Tick**: `tick_effect_timers` decrements the remaining time of each entry every frame.
- **Expiry**: When an entry's remaining time reaches zero, the corresponding effect is reversed and the entry is removed from the vec. If the vec becomes empty, the component is removed from the entity.
