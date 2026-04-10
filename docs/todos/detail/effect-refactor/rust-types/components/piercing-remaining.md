# Name
PiercingRemaining

# Struct
```rust
/// Number of cells the bolt can pierce through without bouncing.
#[derive(Component)]
pub struct PiercingRemaining(pub u32);
```

# Location
`src/effect/effects/piercing/`

# Description
`PiercingRemaining` is added to the bolt to track how many more cells it can pass through without reflecting.

- **Added by**: `PiercingConfig.fire()` inserts or updates the component on the bolt, setting the remaining pierce count.
- **Decremented by**: The bolt-cell collision system checks for this component. When present and `> 0`, the bolt passes through the cell instead of bouncing, and the count is decremented.
- **Reset**: The count is reset at the start of each node to its configured value.
- **Not removed**: The component persists on the bolt for the duration of the effect. It is managed through value updates rather than insertion/removal.
