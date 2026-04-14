# Name
SpeedBoost

# Enum Variant
- `EffectType::SpeedBoost(SpeedBoostConfig)`
- `ReversibleEffectType::SpeedBoost(SpeedBoostConfig)`

# Config
`SpeedBoostConfig { multiplier: OrderedFloat<f32> }`

# Fire
1. Get or insert `EffectStack<SpeedBoostConfig>` on the target entity.
2. Push `(source, config)` onto the stack.
3. Fire does NOT recalculate velocity -- that is the bolt velocity system's job.

# Reverse
1. Get `EffectStack<SpeedBoostConfig>` on the target entity.
2. Remove the first entry matching `(source, config)`.
3. Reverse does NOT recalculate velocity -- that is the bolt velocity system's job.

# Source Location
`src/effect_v3/effects/speed_boost/config.rs`

# New Types
None beyond `EffectStack<SpeedBoostConfig>`.

# New Systems
None -- downstream systems (bolt velocity) read the stack aggregate.
