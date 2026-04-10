# Name
TetherBeamConfig

# Syntax
```rust
struct TetherBeamConfig {
    damage_mult: OrderedFloat<f32>,
    chain: bool,
}
```

# Description
- damage_mult: Multiplier applied to base damage for cells the beam crosses each tick
- chain: false = spawn a new bolt and beam to it; true = connect existing bolts
