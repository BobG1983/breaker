# Name
DeathOccurred(EntityKind)

# When it fires
An entity of the specified kind died somewhere in the world.

# Scope
Global. Fires on every entity that has BoundEffects or StagedEffects.

# TriggerContext
`TriggerContext::Death { victim, killer }` — same context as the Died/Killed triggers that fire on the participants.

All global triggers populate their participant context. On(Death(Killer)) inside a DeathOccurred tree resolves to the killer entity.

# Description
The global counterpart of Died/Killed. `DeathOccurred(Cell)` fires on all entities when any cell dies. Use this when an entity not involved in the death wants to react.
