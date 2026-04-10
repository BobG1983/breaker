# Name
NodeTimerThresholdOccurred

# Parameters
`f32` — ratio between 0.0 and 1.0.

# Description
Fires on all entities when the node timer ratio crosses the specified threshold. `NodeTimerThresholdOccurred(0.5)` fires when half the node's time has elapsed. Use for time-pressure effects — desperation boosts that kick in as time runs out, or penalties that escalate as the clock ticks down.
