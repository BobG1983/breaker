# Name
ShockwaveConfig

# Syntax
```rust
struct ShockwaveConfig {
    base_range: f32,
    range_per_level: f32,
    stacks: u32,
    speed: f32,
}
```

# Description
- base_range: How far the shockwave ring expands before disappearing
- range_per_level: Extra range added per stack beyond the first
- stacks: Current stack count -- effective range is base_range + range_per_level x (stacks - 1)
- speed: How fast the ring expands outward in world units per second
