# Name
EffectStack\<DamageBoostConfig\>

# Syntax
```rust
type DamageBoostStack = EffectStack<DamageBoostConfig>;
```

# Description
Stack of active damage boost entries on an entity. Aggregation: multiplicative — product of all multiplier values. Default aggregate (empty stack) is 1.0.

Replaces `ActiveDamageBoosts`.
