# Name
on_node_end_occurred

# Reads
Node state transition via `OnExit(NodeState::Playing)`.

# Dispatches
`Trigger::NodeEndOccurred`

# Scope
Global.

Walk ALL entities with BoundEffects/StagedEffects.

# TriggerContext
`TriggerContext::None`

# Source Location
`src/effect/triggers/node/bridges.rs`

# Schedule
`OnExit(NodeState::Playing)` — runs once when the node transitions out of Playing. This is NOT a FixedUpdate system — it fires as a one-shot state transition hook.

# Behavior
1. Walk ALL entities with BoundEffects/StagedEffects with `NodeEndOccurred` and `TriggerContext::None`.

- Does NOT tear down the node.
- Does NOT despawn cells or clean up node resources.
- Does NOT modify the node timer or threshold registry.
