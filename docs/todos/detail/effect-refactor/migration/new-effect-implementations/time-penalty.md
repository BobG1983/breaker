# Name
TimePenalty

# Enum Variant
- `EffectType::TimePenalty(TimePenaltyConfig)`
- NOT in `ReversibleEffectType`

# Config
`TimePenaltyConfig { seconds: OrderedFloat<f32> }`

# Fire
1. Read the `NodeTimer` resource.
2. Subtract `seconds` from the remaining time.
3. Clamp to 0 -- do not go negative.
4. Fire does NOT check if the timer reached zero -- the node timer system handles that.
5. Fire does NOT end the node.

# Reverse
Not applicable -- TimePenalty is not reversible.

# Source Location
`src/effect/configs/time_penalty.rs`

# New Types
None -- reads existing `NodeTimer` resource.

# New Systems
None.
