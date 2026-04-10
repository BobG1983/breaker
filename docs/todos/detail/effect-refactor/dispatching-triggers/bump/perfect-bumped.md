# Name
PerfectBumped

# When it fires
The bolt contacts the breaker and the player's bump input was within the perfect timing window.

# Scope
Local. Fires on the bolt and the breaker that participated in the bump.

On targets resolve as:
- `Bump(Bolt)` → the bolt entity that was bumped
- `Bump(Breaker)` → the breaker entity that did the bumping

# Description
PerfectBumped is the highest-grade bump timing. It fires alongside Bumped (which fires on any successful bump regardless of timing grade). A perfect bump produces both PerfectBumped and Bumped on the same entities in the same frame, plus PerfectBumpOccurred and BumpOccurred globally.

DO fire PerfectBumped before the global variants so local effects resolve before global ones see the event.
DO fire both PerfectBumped and Bumped — they are not mutually exclusive.
DO NOT fire EarlyBumped or LateBumped alongside PerfectBumped — timing grades are exclusive.
