# Name
PulseConfig

# Syntax
```rust
struct PulseConfig {
    base_range: f32,
    range_per_level: f32,
    stacks: u32,
    speed: f32,
    interval: f32,
}
```

# Description
- base_range: Radius of each pulse shockwave
- range_per_level: Extra range per stack
- stacks: Current stack count
- speed: Expansion speed of each pulse ring
- interval: Seconds between each pulse emission
