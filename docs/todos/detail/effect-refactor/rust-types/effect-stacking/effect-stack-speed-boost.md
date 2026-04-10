# Name
EffectStack\<SpeedBoostConfig\>

# Syntax
```rust
type SpeedBoostStack = EffectStack<SpeedBoostConfig>;
```

# Description
Stack of active speed boost entries on an entity. Aggregation: multiplicative — product of all multiplier values. Default aggregate (empty stack) is 1.0.

Replaces `ActiveSpeedBoosts`.
