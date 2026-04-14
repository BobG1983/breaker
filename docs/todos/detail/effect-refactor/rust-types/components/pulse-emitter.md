# Name
PulseEmitter

# Struct
```rust
/// Periodically emits expanding pulse shockwaves from the bolt.
#[derive(Component)]
pub struct PulseEmitter {
    /// Base range of each pulse.
    pub base_range: f32,
    /// Additional range per stack level.
    pub range_per_level: f32,
    /// Current stack count (affects range).
    pub stacks: u32,
    /// Expansion speed of emitted pulses.
    pub speed: f32,
    /// Interval in seconds between pulse emissions.
    pub interval: f32,
    /// Current countdown timer until next pulse.
    pub timer: f32,
}
```

# Location
`src/effect_v3/effects/pulse/`

# Description
`PulseEmitter` is added to the bolt to periodically spawn expanding pulse waves that damage nearby cells.

- **Added by**: `PulseConfig.fire()` inserts the component with configured parameters. Additional fires increment `stacks`, increasing the effective range.
- **Tick**: `tick_pulse` decrements `timer` each frame. When the timer reaches zero, a pulse shockwave is spawned at the bolt's position with range `base_range + range_per_level * stacks`, and the timer resets to `interval`.
- **Removed by**: `PulseConfig.reverse()` removes the component, stopping all pulse emissions.
