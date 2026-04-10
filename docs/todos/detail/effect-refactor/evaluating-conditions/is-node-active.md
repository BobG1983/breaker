# Name
is_node_active

# Signature
```rust
fn is_node_active(world: &World) -> bool;
```

# Source Location
`src/effect/systems/conditions/node_active.rs`

# Description
Returns true when a node is currently playing. Reads the `State<NodeState>` resource from the world. Returns true when the state is `NodeState::Playing`, false otherwise.

This condition becomes true on node start and false on node end/teardown. During(NodeActive, ...) entries fire their scoped effects when play begins and reverse them when play ends.

This evaluator does NOT:
- Check whether cells remain. Node completion is a separate concern.
- Modify any state. Pure read-only query.
