# Name
QuickStop

# Enum Variant
- `EffectType::QuickStop(QuickStopConfig)`
- `ReversibleEffectType::QuickStop(QuickStopConfig)`

# Config
`QuickStopConfig { multiplier: OrderedFloat<f32> }`

# Fire
1. Get or insert `EffectStack<QuickStopConfig>` on the target entity.
2. Push `(source, config)` onto the stack.
3. Fire does NOT change deceleration rate -- that is the breaker movement system's job.

# Reverse
1. Get `EffectStack<QuickStopConfig>` on the target entity.
2. Remove the first entry matching `(source, config)`.
3. Reverse does NOT change deceleration rate -- that is the breaker movement system's job.

# Source Location
`src/effect/configs/quick_stop.rs`

# New Types
None beyond `EffectStack<QuickStopConfig>`.

# New Systems
None -- downstream system (breaker movement) reads the stack aggregate for deceleration rate.
