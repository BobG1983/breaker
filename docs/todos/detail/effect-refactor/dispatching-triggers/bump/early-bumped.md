# Name
EarlyBumped

# When it fires
The bolt contacts the breaker and the player's bump input was before the perfect window but still within the acceptable early window.

# Scope
Local. Fires on the bolt and the breaker that participated in the bump.

On targets resolve as:
- `Bump(Bolt)` → the bolt entity that was bumped
- `Bump(Breaker)` → the breaker entity that did the bumping

# Description
EarlyBumped is the early timing grade. It fires alongside Bumped (any successful bump). An early bump produces both EarlyBumped and Bumped locally, plus EarlyBumpOccurred and BumpOccurred globally.

DO fire both EarlyBumped and Bumped.
DO NOT fire PerfectBumped or LateBumped alongside EarlyBumped — timing grades are exclusive.
