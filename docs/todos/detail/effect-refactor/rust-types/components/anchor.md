# Name
AnchorActive, AnchorTimer, AnchorPlanted

# Struct
```rust
/// Marks the breaker as having an active anchor effect and holds its parameters.
#[derive(Component)]
pub struct AnchorActive {
    /// Multiplier applied to bump force while anchored.
    pub bump_force_multiplier: f32,
    /// Multiplier applied during the perfect-hit window.
    pub perfect_window_multiplier: f32,
    /// Delay in seconds before the anchor plants after activation.
    pub plant_delay: f32,
}

/// Countdown timer tracking how long until the anchor plants.
#[derive(Component)]
pub struct AnchorTimer(pub f32);

/// Marker indicating the anchor has planted (breaker is now stationary).
#[derive(Component)]
pub struct AnchorPlanted;
```

# Location
`src/effect_v3/effects/anchor/`

# Description
These three components work together to implement the anchor effect on the breaker entity.

- **Added by**: `AnchorConfig.fire()` inserts `AnchorActive` and `AnchorTimer` onto the breaker. `AnchorPlanted` is not added at fire time.
- **Tick**: `tick_anchor` decrements `AnchorTimer` each frame. When the timer reaches zero, it inserts `AnchorPlanted` onto the breaker, locking it in place.
- **Removed by**: `AnchorConfig.reverse()` removes all three components (`AnchorActive`, `AnchorTimer`, `AnchorPlanted`) from the breaker, fully deactivating the effect.
