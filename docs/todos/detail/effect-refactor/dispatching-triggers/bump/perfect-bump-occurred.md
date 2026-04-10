# Name
PerfectBumpOccurred

# When it fires
A perfect bump happened somewhere in the world.

# Scope
Global. Fires on every entity that has BoundEffects or StagedEffects.

# Description
The global counterpart of PerfectBumped. Fires on all entities so that effects not directly involved in the bump can react. For example, a cell with "on any perfect bump, become vulnerable" would use this trigger.

Fired alongside BumpOccurred (global catch-all) in the same frame.

Bump participant context IS populated for global bump triggers — On(Bump(Bolt)) and On(Bump(Breaker)) resolve to the bolt and breaker from the bump event.
