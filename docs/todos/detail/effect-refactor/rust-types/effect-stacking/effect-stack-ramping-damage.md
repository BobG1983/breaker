# Name
EffectStack\<RampingDamageConfig\>

# Syntax
```rust
type RampingDamageStack = EffectStack<RampingDamageConfig>;
```

# Description
Stack of active ramping damage entries on an entity. Aggregation: additive — sum of all increment values. This is the per-activation increment, not the accumulated total. The accumulated damage is tracked separately and resets each node.

Replaces no existing type — RampingDamage previously used a different accumulation pattern.
