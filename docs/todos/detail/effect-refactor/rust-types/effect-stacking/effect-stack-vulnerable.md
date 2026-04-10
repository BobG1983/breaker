# Name
EffectStack\<VulnerableConfig\>

# Syntax
```rust
type VulnerableStack = EffectStack<VulnerableConfig>;
```

# Description
Stack of active vulnerability entries on an entity. Aggregation: multiplicative — product of all multiplier values. Default aggregate (empty stack) is 1.0.

Replaces `ActiveVulnerability`.
