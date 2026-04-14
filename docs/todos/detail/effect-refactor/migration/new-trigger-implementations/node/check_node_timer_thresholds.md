# Name
check_node_timer_thresholds

# Reads
Node timer ratio (current elapsed / total duration).
`NodeTimerThresholdRegistry` resource.

# Dispatches
Nothing -- this is a game system, not a bridge. Sends `NodeTimerThresholdCrossed` message.

# Scope
N/A (game system).

# TriggerContext
N/A (game system).

# Source Location
`src/effect_v3/triggers/node/check_thresholds.rs`

# Schedule
FixedUpdate, after node timer tick.

# Behavior
1. Read the current node timer ratio.
2. Read the `NodeTimerThresholdRegistry` resource.
3. For each threshold in `thresholds` where `ratio >= threshold` and threshold is NOT in `fired`:
   a. Send `NodeTimerThresholdCrossed { ratio: threshold.into_inner() }`.
   b. Insert threshold into `fired`.

- Does NOT dispatch triggers -- the `on_node_timer_threshold_occurred` bridge does that.
- Does NOT modify the node timer.
- Does NOT read or write effect trees.
