# Name
RampingDamage

# Enum Variant
- `EffectType::RampingDamage(RampingDamageConfig)`
- `ReversibleEffectType::RampingDamage(RampingDamageConfig)`

# Config
`RampingDamageConfig { increment: OrderedFloat<f32> }`

# Fire
1. Get or insert `EffectStack<RampingDamageConfig>` on the target entity.
2. Push `(source, config)` onto the stack.
3. If `RampingDamageAccumulator` is not present on the entity, insert `RampingDamageAccumulator(OrderedFloat(0.0))`.
4. Fire does NOT increment the accumulator -- that happens when the gating trigger fires again.
5. Fire does NOT change damage dealt -- the accumulator value is added to base damage before the damage multiplier: `(BoltBaseDamage + RampingDamageAccumulator) * EffectStack<DamageBoostConfig>.aggregate()`.

Note: aggregation is additive (sum of increments), not multiplicative.

# Reverse
1. Get `EffectStack<RampingDamageConfig>` on the target entity.
2. Remove the first entry matching `(source, config)`.
3. If the stack is empty, remove `RampingDamageAccumulator` from the entity.
4. Reverse does NOT change damage dealt -- that is the downstream system's job.

# Source Location
`src/effect_v3/effects/ramping_damage/config.rs`

# New Types
- `RampingDamageAccumulator(OrderedFloat<f32>)` -- component that tracks accumulated bonus damage from repeated trigger activations. When the gating trigger fires again, the accumulator is incremented by the stack aggregate (sum of all increments). The accumulator value is added to damage by the downstream system. Resets to 0.0 at node start.

# New Systems

## reset_ramping_damage
- **What it does**: For each entity with `RampingDamageAccumulator`, set the value to `OrderedFloat(0.0)`.
- **What it does NOT do**: Does not remove the accumulator. Does not modify the EffectStack. Does not change damage dealt.
- **Schedule**: OnEnter(NodeState::Loading), runs when a new node starts loading. Registered in `EffectV3Systems::Reset`.
