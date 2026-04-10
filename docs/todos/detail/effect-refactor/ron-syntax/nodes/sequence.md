# Name
Sequence

# Parameters
- A list of Terminals to execute in order

# Description
Sequence executes multiple terminals left to right. Sequence([Fire(SpeedBoost(1.5)), Fire(DamageBoost(2.0))]) means "first apply speed boost, then apply damage boost" in that specific order.

Use Sequence when order matters — for example, if one effect should be applied before another because it affects the other's behavior. If order doesn't matter, use separate Stamp entries instead.

A Sequence is considered reversible if all its children are reversible. When reversed (inside During or Until), the children are reversed in reverse order — last applied, first removed.

Each child in a Sequence must be a Terminal (Fire, Stamp, or Route).
