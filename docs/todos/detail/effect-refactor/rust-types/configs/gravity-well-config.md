# Name
GravityWellConfig

# Syntax
```rust
struct GravityWellConfig {
    strength: OrderedFloat<f32>,
    duration: OrderedFloat<f32>,
    radius: OrderedFloat<f32>,
    max: u32,
}
```

# Description
- strength: How strongly bolts are pulled toward the well center per tick
- duration: How long the well exists before despawning
- radius: How far from the well center bolts are affected
- max: Maximum active wells per owner entity -- oldest removed when exceeded
