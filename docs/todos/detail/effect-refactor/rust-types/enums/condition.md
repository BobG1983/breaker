# Name
Condition

# Derives
`Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize`

# Syntax
```rust
enum Condition {
    NodeActive,
    ShieldActive,
    ComboActive(u32),
}
```

# Description
- NodeActive: True while the current node is in the `Playing` state. Starts on `OnEnter(NodeState::Playing)`, ends on exit from `Playing` (i.e. when teardown begins). See [node-active](../ron-syntax/conditions/node-active.md)
- ShieldActive: True while at least one ShieldWall entity exists in the world. See [shield-active](../ron-syntax/conditions/shield-active.md)
- ComboActive: True while the consecutive perfect bump streak is at or above the given count. Ends when a non-perfect bump breaks the streak. See [combo-active](../ron-syntax/conditions/combo-active.md)
