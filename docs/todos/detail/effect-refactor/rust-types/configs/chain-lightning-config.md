# Name
ChainLightningConfig

# Syntax
```rust
struct ChainLightningConfig {
    arcs: u32,
    range: OrderedFloat<f32>,
    damage_mult: OrderedFloat<f32>,
    arc_speed: OrderedFloat<f32>,
}
```

# Description
- arcs: Number of times the lightning jumps between cells
- range: Maximum distance each arc can jump to find a new target
- damage_mult: Multiplier applied to base damage for each arc hit
- arc_speed: How fast each lightning arc travels between cells in world units per second
