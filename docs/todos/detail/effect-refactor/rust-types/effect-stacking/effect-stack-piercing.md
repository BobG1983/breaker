# Name
EffectStack\<PiercingConfig\>

# Syntax
```rust
type PiercingStack = EffectStack<PiercingConfig>;
```

# Description
Stack of active piercing entries on a bolt. Aggregation: additive — sum of all charges values. Default aggregate (empty stack) is 0.

Replaces `ActivePiercings`.
