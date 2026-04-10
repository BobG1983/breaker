# Name
SizeBoost

# Enum Variant
- `EffectType::SizeBoost(SizeBoostConfig)`
- `ReversibleEffectType::SizeBoost(SizeBoostConfig)`

# Config
`SizeBoostConfig { multiplier: OrderedFloat<f32> }`

# Fire
1. Get or insert `EffectStack<SizeBoostConfig>` on the target entity.
2. Push `(source, config)` onto the stack.
3. Fire does NOT change Scale2D -- that is sync_bolt_scale's job.

# Reverse
1. Get `EffectStack<SizeBoostConfig>` on the target entity.
2. Remove the first entry matching `(source, config)`.
3. Reverse does NOT change Scale2D -- that is sync_bolt_scale's job.

# Source Location
`src/effect/effects/size_boost/config.rs`

# New Types
None beyond `EffectStack<SizeBoostConfig>`.

# New Systems
None -- downstream system (sync_bolt_scale) reads the stack aggregate.
