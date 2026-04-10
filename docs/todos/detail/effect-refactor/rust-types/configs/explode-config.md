# Name
ExplodeConfig

# Syntax
```rust
struct ExplodeConfig {
    range: OrderedFloat<f32>,
    damage: OrderedFloat<f32>,
}
```

# Description
- range: Radius of the explosion in world units
- damage: Flat damage dealt to every cell within range (not modified by damage boosts)
