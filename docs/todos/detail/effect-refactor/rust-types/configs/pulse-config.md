# Name
PulseConfig

# Syntax
```rust
struct PulseConfig {
    base_range: OrderedFloat<f32>,
    range_per_level: OrderedFloat<f32>,
    stacks: u32,
    speed: OrderedFloat<f32>,
    interval: OrderedFloat<f32>,
}
```

# Description
- base_range: Radius of each pulse shockwave
- range_per_level: Extra range per stack
- stacks: Current stack count
- speed: Expansion speed of each pulse ring
- interval: Seconds between each pulse emission
