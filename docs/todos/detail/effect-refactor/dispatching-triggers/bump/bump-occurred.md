# Name
BumpOccurred

# When it fires
Any successful bump happened somewhere in the world (perfect, early, or late).

# Scope
Global. Fires on every entity that has BoundEffects or StagedEffects.

# Description
The global counterpart of Bumped. Always fired alongside exactly one timing-graded global variant (PerfectBumpOccurred, EarlyBumpOccurred, or LateBumpOccurred).

DO NOT populate bump participant context for global triggers.
