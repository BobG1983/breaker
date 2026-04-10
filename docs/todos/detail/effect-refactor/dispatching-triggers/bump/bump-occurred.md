# Name
BumpOccurred

# When it fires
Any successful bump happened somewhere in the world (perfect, early, or late).

# Scope
Global. Fires on every entity that has BoundEffects or StagedEffects.

# Description
The global counterpart of Bumped. Always fired alongside exactly one timing-graded global variant (PerfectBumpOccurred, EarlyBumpOccurred, or LateBumpOccurred).

Bump participant context IS populated for global bump triggers — On(Bump(Bolt)) and On(Bump(Breaker)) resolve to the bolt and breaker from the bump event.
