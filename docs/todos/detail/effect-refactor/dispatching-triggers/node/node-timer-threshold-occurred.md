# Name
NodeTimerThresholdOccurred(f32)

# When it fires
The node timer ratio crosses the specified threshold value (0.0 to 1.0). 0.0 is the start of the node, 1.0 is time expired.

# Scope
Global. Fires on every entity that has BoundEffects or StagedEffects.

# Description
NodeTimerThresholdOccurred is parameterized by a ratio. `NodeTimerThresholdOccurred(0.75)` fires when 75% of the node timer has elapsed. Use this for time-pressure effects like "when the timer is nearly up, apply a penalty."

The threshold fires once per node when the ratio is crossed. It does not fire repeatedly.

No participant context — node lifecycle events have no participants.
DO NOT use On(...) inside a NodeTimerThresholdOccurred tree — there are no participants to resolve.
DO fire exactly once when the ratio crosses the threshold, not on every frame the ratio is above it.
