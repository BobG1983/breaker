# Name
TetherBeamConfig

# Syntax
```rust
struct TetherBeamConfig {
    damage_mult: OrderedFloat<f32>,
    chain: bool,
    width: OrderedFloat<f32>,
}
```

# Description
- damage_mult: Multiplier applied to base damage for cells the beam crosses each tick
- chain: false = spawn a new bolt and beam to it; true = connect existing bolts
- width: Half-width of the beam line; cells whose perpendicular distance to the beam is ≤ this value are damaged each tick
