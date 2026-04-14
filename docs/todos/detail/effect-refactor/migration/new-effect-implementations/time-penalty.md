# Name
TimePenalty

# Enum Variant
- `EffectType::TimePenalty(TimePenaltyConfig)`
- NOT in `ReversibleEffectType`

# Config
`TimePenaltyConfig { seconds: OrderedFloat<f32> }`

# Fire
1. Send `ApplyTimePenalty { seconds: config.seconds.0 }` message.
2. The run domain's handler for `ApplyTimePenalty` subtracts `seconds` from the `NodeTimer` remaining time and clamps to 0.
3. Fire does NOT read or modify the NodeTimer directly — it sends a message and lets the run domain handle it.
4. Fire does NOT check if the timer reached zero — the node timer system handles that.
5. Fire does NOT end the node.

# Reverse
Not applicable — TimePenalty is not reversible.

# Source Location
`src/effect_v3/effects/time_penalty/config.rs`

# New Types
None — sends existing `ApplyTimePenalty` message.

# New Systems
None.
