# Name
EffectStack\<BumpForceConfig\>

# Syntax
```rust
type BumpForceStack = EffectStack<BumpForceConfig>;
```

# Description
Stack of active bump force entries on the breaker. Aggregation: multiplicative — product of all multiplier values. Default aggregate (empty stack) is 1.0.

Replaces `ActiveBumpForces`.
