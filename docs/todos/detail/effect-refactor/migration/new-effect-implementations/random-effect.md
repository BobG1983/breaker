# Name
RandomEffect

# Enum Variant
- `EffectType::RandomEffect(RandomEffectConfig)`
- NOT in `ReversibleEffectType`

# Config
`RandomEffectConfig { pool: Vec<(OrderedFloat<f32>, Box<EffectType>)> }`

# Fire
1. Select exactly one effect from the pool using weighted random selection. The weight of each entry determines its probability relative to the total weight.
2. Call the selected effect's `config.fire(entity, source, world)`.
3. If the pool is empty, do nothing.
4. Fire does NOT fire multiple effects.
5. Fire does NOT modify the pool.
6. Fire does NOT track which effect was selected.
7. Fire delegates to the selected effect's `Fireable` impl -- if the selected effect is a SpeedBoost, `SpeedBoostConfig.fire` runs. If it's a Shockwave, `ShockwaveConfig.fire` runs. The meta effect is transparent.

# Reverse
Not applicable -- RandomEffect is not reversible. The effect it fired may or may not be reversible, but RandomEffect itself has no reverse because you'd need to know which effect was selected, and that's not tracked.

# Source Location
`src/effect/effects/random_effect/config.rs`

# New Types
None.

# New Systems
None -- fully resolved in fire via delegation.
