# Name
on_node_start_occurred

# Reads
Node state transition via `OnEnter(NodeState::Playing)`.

# Dispatches
`Trigger::NodeStartOccurred`

# Scope
Global.

Walk ALL entities with BoundEffects/StagedEffects.

# TriggerContext
`TriggerContext::None`

# Source Location
`src/effect_v3/triggers/node/bridges.rs`

# Schedule
`OnEnter(NodeState::Playing)` — runs once when the node transitions to Playing. This is NOT a FixedUpdate system — it fires as a one-shot state transition hook.

# Behavior
1. Walk ALL entities with BoundEffects/StagedEffects with `NodeStartOccurred` and `TriggerContext::None`.

- Does NOT spawn cells or set up the node.
- Does NOT modify the node timer.
- Does NOT reset the NodeTimerThresholdRegistry — that is a separate reset system's responsibility.
