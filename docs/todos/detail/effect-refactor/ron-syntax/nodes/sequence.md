# Name
Sequence

# Parameters
- A list of Terminals to execute in order

# Description
Sequence executes multiple terminals left to right. Sequence([Fire(SpeedBoost(SpeedBoostConfig(multiplier: 1.5))), Fire(DamageBoost(DamageBoostConfig(multiplier: 2.0)))]) means "first apply speed boost, then apply damage boost" in that specific order.

Use Sequence when order matters — for example, if one effect should be applied before another because it affects the other's behavior. If order doesn't matter, use separate Stamp entries instead.

Each child in a Sequence must be a Terminal (Fire or Route).

## Inside During/Until (scoped context)

When Sequence appears as a direct child of During or Until, its children are bare reversible effects — not Terminals. The Sequence holds ReversibleEffectType values directly, not Fire-wrapped.

During(NodeActive, Sequence([SpeedBoost(SpeedBoostConfig(multiplier: 1.5)), DamageBoost(DamageBoostConfig(multiplier: 2.0))])) — note no Fire wrapper inside the scoped Sequence.

When reversed, children are reversed in reverse order — last applied, first removed.
