# Name
Vulnerable

# Enum Variant
- `EffectType::Vulnerable(VulnerableConfig)`
- `ReversibleEffectType::Vulnerable(VulnerableConfig)`

# Config
`VulnerableConfig { multiplier: OrderedFloat<f32> }`

# Fire
1. Get or insert `EffectStack<VulnerableConfig>` on the target entity.
2. Push `(source, config)` onto the stack.
3. Fire does NOT change incoming damage -- that is apply_damage's job.
4. Typically stamped on cells, not bolts.

# Reverse
1. Get `EffectStack<VulnerableConfig>` on the target entity.
2. Remove the first entry matching `(source, config)`.
3. Reverse does NOT change incoming damage -- that is apply_damage's job.

# Source Location
`src/effect/effects/vulnerable/config.rs`

# New Types
None beyond `EffectStack<VulnerableConfig>`.

# New Systems
None -- downstream system (apply_damage) reads the stack aggregate when applying damage to the entity.
