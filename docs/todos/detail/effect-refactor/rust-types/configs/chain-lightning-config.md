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
- damage_mult: Snapshot multiplier. Computed once at fire time as `source BoltBaseDamage * damage_mult * DamageBoost.aggregate()`. This snapshot value is stored on the spawned chain lightning entity and applied uniformly to every arc hit from this emission — it does not re-read passive stacks at tick time.
- arc_speed: How fast each lightning arc travels between cells in world units per second
