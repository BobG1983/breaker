# Name
Bumped

# When it fires
The bolt contacts the breaker and the player provided bump input within any acceptable timing window (perfect, early, or late).

# Scope
Local. Fires on the bolt and the breaker that participated in the bump.

On targets resolve as:
- `Bump(Bolt)` → the bolt entity that was bumped
- `Bump(Breaker)` → the breaker entity that did the bumping

# Description
Bumped is the catch-all successful bump trigger. It always fires alongside exactly one timing-graded trigger (PerfectBumped, EarlyBumped, or LateBumped). Use Bumped when the effect should fire on any successful bump regardless of timing quality.

DO always fire Bumped alongside the timing-graded variant.
DO NOT fire Bumped when the bump was a whiff or no input was provided — those are BumpWhiffOccurred and NoBumpOccurred.
