# Name
on_node_start_occurred

# Reads
Node state transition (entered Playing state or equivalent).

# Dispatches
`Trigger::NodeStartOccurred`

# Scope
Global.

Walk ALL entities with `NodeStartOccurred` trigger attached.

# TriggerContext
`TriggerContext::None`

# Source Location
`src/effect/bridges/node.rs`

# Schedule
FixedUpdate, on node state enter.

# Behavior
1. Detect that the node has entered the Playing state (state transition).
2. Walk ALL entities with BoundEffects/StagedEffects with `NodeStartOccurred` and `TriggerContext::None`.

- Does NOT spawn cells or set up the node.
- Does NOT modify the node timer.
- Does NOT reset the NodeTimerThresholdRegistry — that is `check_node_timer_thresholds`' responsibility on node start.
