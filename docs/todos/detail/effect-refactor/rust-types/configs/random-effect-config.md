# Name
RandomEffectConfig

# Syntax
```rust
struct RandomEffectConfig {
    pool: Vec<(OrderedFloat<f32>, Box<EffectType>)>,
}
```

# Description
- pool: Weighted list of effects to randomly select from -- each entry is (weight, effect). Fires exactly one from the pool per activation. See [EffectType](../enums/effect-type.md)
