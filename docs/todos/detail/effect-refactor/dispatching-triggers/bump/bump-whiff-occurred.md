# Name
BumpWhiffOccurred

# When it fires
The bolt entered the bump timing window near the breaker, the player provided bump input, but the timing was outside all acceptable windows (too early or too late to count).

# Scope
Global. Fires on every entity that has BoundEffects or StagedEffects.

# Description
A whiff is a failed bump attempt — the player tried but missed the timing. This is distinct from NoBumpOccurred where no input was provided at all.

DO NOT fire Bumped or any timing-graded trigger alongside a whiff — the bump failed.
DO NOT fire BumpOccurred alongside a whiff.
