# Name
DeathOccurred(EntityKind)

# When it fires
An entity of the specified kind died somewhere in the world.

# Scope
Global. Fires on every entity that has BoundEffects or StagedEffects.

# Description
The global counterpart of Died/Killed. `DeathOccurred(Cell)` fires on all entities when any cell dies. Use this when an entity not involved in the death wants to react.

DO NOT populate death participant context for global triggers — On(Death(...)) inside a DeathOccurred tree would have no participant to resolve.
