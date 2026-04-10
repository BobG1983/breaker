# Name
ImpactOccurred(EntityKind)

# When it fires
A collision involving an entity of the specified kind happened somewhere in the world.

# Scope
Global. Fires on every entity that has BoundEffects or StagedEffects.

# Description
The global counterpart of Impacted. `ImpactOccurred(Cell)` fires on all entities when any collision involving a cell happened. Use this when an entity not involved in the collision wants to react.

DO NOT populate impact participant context for global triggers — On(Impact(...)) inside an ImpactOccurred tree would have no participant to resolve.
