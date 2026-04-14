# Name
CircuitBreakerCounter

# Struct
```rust
/// Tracks bump counts toward triggering an automatic shockwave.
#[derive(Component)]
pub struct CircuitBreakerCounter {
    /// Bumps remaining until the next shockwave fires.
    pub remaining: u32,
    /// Total bumps required per cycle.
    pub bumps_required: u32,
    /// Number of shockwaves to spawn when the counter reaches zero.
    pub spawn_count: u32,
    /// Whether child bolts inherit this counter.
    pub inherit: bool,
    /// Radius of the triggered shockwave.
    pub shockwave_range: f32,
    /// Expansion speed of the triggered shockwave.
    pub shockwave_speed: f32,
}
```

# Location
`src/effect_v3/effects/circuit_breaker/`

# Description
`CircuitBreakerCounter` is added to the bolt to track bump events toward an automatic shockwave trigger.

- **Added by**: `CircuitBreakerConfig.fire()` inserts the component on first activation. Subsequent fires may update parameters.
- **Tick**: Each bolt-cell bump decrements `remaining`. When `remaining` reaches zero, `spawn_count` shockwaves are spawned at the bump location and `remaining` resets to `bumps_required`.
- **Removed by**: `CircuitBreakerConfig.reverse()` removes the component entirely.
