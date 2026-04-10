# Name
AnchorConfig

# Syntax
```rust
struct AnchorConfig {
    bump_force_multiplier: OrderedFloat<f32>,
    perfect_window_multiplier: OrderedFloat<f32>,
    plant_delay: OrderedFloat<f32>,
}
```

# Description
- bump_force_multiplier: How much the bump force is multiplied when planted
- perfect_window_multiplier: How much wider the perfect timing window becomes when planted
- plant_delay: Seconds the breaker must stand still before planting
