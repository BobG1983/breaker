# Name
Condition

# Syntax
```rust
enum Condition {
    NodeActive,
    ShieldActive,
    ComboActive(u32),
}
```

# Description
- NodeActive: True while a node is playing or paused. Starts on node enter, ends on node teardown. See [node-active](../ron-syntax/conditions/node-active.md)
- ShieldActive: True while at least one ShieldWall entity exists in the world. See [shield-active](../ron-syntax/conditions/shield-active.md)
- ComboActive: True while the consecutive perfect bump streak is at or above the given count. Ends when a non-perfect bump breaks the streak. See [combo-active](../ron-syntax/conditions/combo-active.md)
