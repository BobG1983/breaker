# Name
LateBumped

# When it fires
The bolt contacts the breaker and the player's bump input was after the perfect window but still within the acceptable late window.

# Scope
Local. Fires on the bolt and the breaker that participated in the bump.

On targets resolve as:
- `Bump(Bolt)` → the bolt entity that was bumped
- `Bump(Breaker)` → the breaker entity that did the bumping

# Description
LateBumped is the late timing grade. It fires alongside Bumped (any successful bump). A late bump produces both LateBumped and Bumped locally, plus LateBumpOccurred and BumpOccurred globally.

DO fire both LateBumped and Bumped.
DO NOT fire PerfectBumped or EarlyBumped alongside LateBumped — timing grades are exclusive.
