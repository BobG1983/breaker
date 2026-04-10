# Name
on_node_end_occurred

# Reads
Node state transition (exited Playing state or node cleared).

# Dispatches
`Trigger::NodeEndOccurred`

# Scope
Global.

Walk ALL entities with `NodeEndOccurred` trigger attached.

# TriggerContext
`TriggerContext::None`

# Source Location
`src/effect/bridges/node.rs`

# Schedule
FixedUpdate, on node state exit.

# Behavior
1. Detect that the node has exited the Playing state (state transition or node cleared).
2. Walk ALL entities that have `NodeEndOccurred` in their trigger set.
3. For each match, invoke the tree walker with `TriggerContext::None`.

- Does NOT tear down the node.
- Does NOT despawn cells or clean up node resources.
- Does NOT modify the node timer or threshold registry.
