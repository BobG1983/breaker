# Name
EffectStack\<QuickStopConfig\>

# Syntax
```rust
type QuickStopStack = EffectStack<QuickStopConfig>;
```

# Description
Stack of active quick stop entries on the breaker. Aggregation: multiplicative — product of all multiplier values. Default aggregate (empty stack) is 1.0.

Replaces `ActiveQuickStops`.
