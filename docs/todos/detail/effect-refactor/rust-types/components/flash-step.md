# Name
FlashStepActive

# Struct
```rust
/// Marker indicating the breaker has flash-step movement enabled.
#[derive(Component)]
pub struct FlashStepActive;
```

# Location
`src/effect_v3/effects/flash_step/`

# Description
`FlashStepActive` is a marker component added to the breaker to enable flash-step movement (instant teleport-style repositioning instead of smooth sliding).

- **Added by**: `FlashStepConfig.fire()` inserts the marker onto the breaker.
- **Read by**: The breaker movement system checks for this component to switch from normal movement to flash-step behavior.
- **Removed by**: `FlashStepConfig.reverse()` removes the marker, restoring normal breaker movement.
