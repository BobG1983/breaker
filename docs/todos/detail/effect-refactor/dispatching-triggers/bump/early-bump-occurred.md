# Name
EarlyBumpOccurred

# When it fires
An early bump happened somewhere in the world.

# Scope
Global. Fires on every entity that has BoundEffects or StagedEffects.

# Description
The global counterpart of EarlyBumped. Fired alongside BumpOccurred in the same frame.

Bump participant context IS populated for global bump triggers — On(Bump(Bolt)) and On(Bump(Breaker)) resolve to the bolt and breaker from the bump event.
