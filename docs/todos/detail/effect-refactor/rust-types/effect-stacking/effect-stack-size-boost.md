# Name
EffectStack\<SizeBoostConfig\>

# Syntax
```rust
type SizeBoostStack = EffectStack<SizeBoostConfig>;
```

# Description
Stack of active size boost entries on an entity. Aggregation: multiplicative — product of all multiplier values. Default aggregate (empty stack) is 1.0.

Replaces `ActiveSizeBoosts`.
