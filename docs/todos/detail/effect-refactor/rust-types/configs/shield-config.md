# Name
ShieldConfig

# Syntax
```rust
struct ShieldConfig {
    duration: OrderedFloat<f32>,
    reflection_cost: OrderedFloat<f32>,
}
```

# Description
- duration: How long the shield wall lasts in seconds
- reflection_cost: Seconds subtracted from the shield's remaining time each time a bolt bounces off it
