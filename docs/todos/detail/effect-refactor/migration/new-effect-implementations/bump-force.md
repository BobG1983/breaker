# Name
BumpForce

# Enum Variant
- `EffectType::BumpForce(BumpForceConfig)`
- `ReversibleEffectType::BumpForce(BumpForceConfig)`

# Config
`BumpForceConfig { multiplier: OrderedFloat<f32> }`

# Fire
1. Get or insert `EffectStack<BumpForceConfig>` on the target entity.
2. Push `(source, config)` onto the stack.
3. Fire does NOT change bump velocity -- that is grade_bump's job.

# Reverse
1. Get `EffectStack<BumpForceConfig>` on the target entity.
2. Remove the first entry matching `(source, config)`.
3. Reverse does NOT change bump velocity -- that is grade_bump's job.

# Source Location
`src/effect/configs/bump_force.rs`

# New Types
None beyond `EffectStack<BumpForceConfig>`.

# New Systems
None -- downstream system (grade_bump) reads the stack aggregate when calculating bump velocity.
