# Name
on_node_timer_threshold_occurred

# Reads
`NodeTimerThresholdCrossed` message.

# Dispatches
`Trigger::NodeTimerThresholdOccurred(ratio)`

# Scope
Global.

Walk ALL entities with `NodeTimerThresholdOccurred(ratio)` trigger attached, where `ratio` matches the crossed threshold.

# TriggerContext
`TriggerContext::None`

# Source Location
`src/effect_v3/triggers/node/bridges.rs`

# Schedule
FixedUpdate, after `check_node_timer_thresholds`.

# Behavior
1. Read each `NodeTimerThresholdCrossed { ratio }` message.
2. Walk ALL entities that have `NodeTimerThresholdOccurred(ratio)` in their trigger set, matching on the specific ratio value.
3. For each match, invoke the tree walker with `TriggerContext::None`.

- Does NOT check the timer -- the `check_node_timer_thresholds` game system does that.
- Does NOT modify the node timer or threshold registry.
- Does NOT interact with node state.
