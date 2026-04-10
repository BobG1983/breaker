# Name
EntropyConfig

# Syntax
```rust
struct EntropyConfig {
    max_effects: u32,
    pool: Vec<(f32, Box<EffectType>)>,
}
```

# Description
- max_effects: Cap on how many effects fire per activation (counter can't exceed this)
- pool: Weighted list of effects to randomly select from -- each entry is (weight, effect). See [EffectType](../enums/effect-type.md)
