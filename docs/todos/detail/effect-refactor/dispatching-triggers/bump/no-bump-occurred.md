# Name
NoBumpOccurred

# When it fires
The bolt contacted the breaker and the bump timing window expired without any bump input from the player.

# Scope
Global. Fires on every entity that has BoundEffects or StagedEffects.

# Description
NoBumpOccurred means the player didn't try to bump at all. The bolt hit the breaker passively. This is distinct from BumpWhiffOccurred where the player tried but mistimed.

DO NOT fire Bumped or any timing-graded trigger — there was no bump attempt.
DO NOT fire BumpWhiffOccurred — there was no input at all.
