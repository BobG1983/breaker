# Name
DamageBoost

# Enum Variant
- `EffectType::DamageBoost(DamageBoostConfig)`
- `ReversibleEffectType::DamageBoost(DamageBoostConfig)`

# Config
`DamageBoostConfig { multiplier: OrderedFloat<f32> }`

# Fire
1. Get or insert `EffectStack<DamageBoostConfig>` on the target entity.
2. Push `(source, config)` onto the stack.
3. Fire does NOT change damage dealt -- that is the collision system's job.

# Reverse
1. Get `EffectStack<DamageBoostConfig>` on the target entity.
2. Remove the first entry matching `(source, config)`.
3. Reverse does NOT change damage dealt -- that is the collision system's job.

# Source Location
`src/effect_v3/effects/damage_boost/config.rs`

# New Types
None beyond `EffectStack<DamageBoostConfig>`.

# New Systems
None -- downstream system (collision) reads the stack aggregate when calculating damage.
